use crate::file_certif::{certificat, SkipServerVerification};
use crate::{ToolError, UI};
use quinn::{ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Error as IoError, ErrorKind};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;

pub struct Transfer {
    pub cancel_handle: Mutex<HashMap<String, JoinHandle<()>>>,
}

impl Transfer {
    pub fn new() -> Self {
        Transfer {
            cancel_handle: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for Transfer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
    transfer_id: String,
    rel_path: String,
    size: u64,
    sha256: String,
    is_dir: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompleteMsg {
    pub sha256: String,
}

const CHUNK_SIZE: usize = 1_048_576; // 1 Mo
const ACK: u8 = 0x01;
const REJECT: u8 = 0x00;
const CHUNK: u8 = 0x02;
const PORT: u16 = 58200;

pub fn io_err<E: std::fmt::Display>(e: E) -> ToolError {
    IoError::new(ErrorKind::Other, e.to_string()).into()
}

pub async fn make_server_endpoint() -> Result<Endpoint, ToolError> {
    let (cert_pem, key_pem) = certificat().await?;
    let mut cert_reader = std::io::BufReader::new(cert_pem.as_bytes());
    let mut key_reader = std::io::BufReader::new(key_pem.as_bytes());

    let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    let key = match rustls_pemfile::private_key(&mut key_reader)? {
        Some(v) => v,
        None => return Err(ToolError::ParseKeyError),
    };

    let server_config = ServerConfig::with_single_cert(certs, key)?;
    let mut server_config = server_config;
    server_config.transport_config(Arc::new(transport_config()));
    let bind_addr: SocketAddr = format!("0.0.0.0:{PORT}").parse().map_err(io_err)?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

/// fenetres QUIC larges (defaut quinn = 1,25 Mo par stream, ca plafonne le debit)
fn transport_config() -> quinn::TransportConfig {
    use quinn::VarInt;
    let mut t = quinn::TransportConfig::default();
    t.stream_receive_window(VarInt::from(8u32 * 1024 * 1024));
    t.receive_window(VarInt::from(32u32 * 1024 * 1024));
    t.send_window(32 * 1024 * 1024);
    t.initial_rtt(std::time::Duration::from_millis(10));
    t
}

pub fn make_client_endpoint() -> Result<Endpoint, ToolError> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(io_err)?,
    ));
    let mut client_config = client_config;
    client_config.transport_config(Arc::new(transport_config()));

    let bind_addr: SocketAddr = "0.0.0.0:0".parse().map_err(io_err)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// reçoit un lot de fichiers sur cette connexion et remonte la liste des fichiers recus
#[allow(clippy::too_many_arguments)]
pub async fn handle_incoming_connection(
    connection: Connection,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
    files: Arc<Mutex<Vec<String>>>,
    bytes: Arc<AtomicU64>,
    ui: Arc<dyn UI>,
    transfer_id: Arc<Mutex<Option<String>>>,
    total: Arc<AtomicU64>,
) -> Result<(), ToolError> {
    loop {
        if stop.load(Ordering::Relaxed) {
            connection.close(0u32.into(), b"arret utilisateur");
            return Ok(());
        }

        match tokio::time::timeout(Duration::from_secs(1), connection.accept_bi()).await {
            Ok(Ok((send, recv))) => {
                let dest_dir = dest_dir.clone();
                let files = files.clone();
                let bytes = bytes.clone();
                let ui = ui.clone();
                let transfer_id = transfer_id.clone();
                let total = total.clone();
                tokio::spawn(async move {
                    if let Err(e) = receive_one(send, recv, &dest_dir, files, bytes, ui, transfer_id, total).await {
                        eprintln!("Erreur reception fichier: {e}");
                    }
                });
            }
            Ok(Err(quinn::ConnectionError::ApplicationClosed(_)))
            | Ok(Err(quinn::ConnectionError::LocallyClosed)) => return Ok(()),
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => continue, // timeout : on revérifie `stop`
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn receive_one(
    mut send: SendStream,
    mut recv: RecvStream,
    dest_dir: &Path,
    files: Arc<Mutex<Vec<String>>>,
    bytes: Arc<AtomicU64>,
    ui: Arc<dyn UI>,
    transfer_id: Arc<Mutex<Option<String>>>,
    total: Arc<AtomicU64>,
) -> Result<(), ToolError> {
    let metadata = read_json_line::<Metadata>(&mut recv).await?;
    let full_path = dest_dir.join(&metadata.rel_path);
    let name = Path::new(&metadata.rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| metadata.rel_path.clone());

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    total.fetch_add(metadata.size, Ordering::Relaxed);

    // l'id du transfert vient de l'emetteur pour que les deux appareils
    // affichent la meme progression sous le meme id
    {
        let mut tid = transfer_id.lock().unwrap();
        if tid.is_none() {
            *tid = Some(metadata.transfer_id.clone());
            ui.show_progress_bar(&metadata.transfer_id);
        }
    }
    let tid = transfer_id.lock().unwrap().clone().unwrap_or_default();

    if metadata.is_dir {
        fs::create_dir_all(&full_path).await?;
        send.write_all(&[ACK]).await?;
        send.finish()?;
        files.lock().unwrap().push(name);
        return Ok(());
    }

    // Ack métadonnées
    send.write_all(&[ACK]).await?;

    let mut out_file = fs::File::create(&full_path).await?;
    let mut hasher = Sha256::new();
    let mut received: u64 = 0;

    while received < metadata.size {
        let mut marker = [0u8; 1];
        recv.read_exact(&mut marker).await?;
        if marker[0] != CHUNK {
            return Err(io_err("trame inattendue (protocole desynchronise)"));
        }

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await?;

        hasher.update(&data);
        out_file.write_all(&data).await?;
        received += len as u64;

        // progression cote receveur, aluminee sur celle de l'emetteur
        let done = bytes.fetch_add(len as u64, Ordering::Relaxed) + len as u64;
        ui.update_progress_bar(&tid, done, total.load(Ordering::Relaxed));
    }

    out_file.flush().await?;

    // Message de complétion
    let complete = read_json_line::<CompleteMsg>(&mut recv).await?;
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash == complete.sha256 && actual_hash == metadata.sha256 {
        send.write_all(&[ACK]).await?;
    } else {
        send.write_all(&[REJECT]).await?;
        let _ = fs::remove_file(&full_path).await;
        return Err(io_err("hash invalide, fichier rejete"));
    }

    send.finish()?;
    files.lock().unwrap().push(name);
    Ok(())
}

pub async fn read_json_line<T: for<'de> Deserialize<'de>>(
    recv: &mut RecvStream,
) -> Result<T, ToolError> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        recv.read_exact(&mut byte).await?;
        if byte[0] == b'\n' {
            break;
        }
        buf.push(byte[0]);
    }

    serde_json::from_slice(&buf).map_err(io_err)
}

pub fn collect_entries<'a>(
    root: &'a Path,
    current: &'a Path,
    out: &'a mut Vec<(PathBuf, String, bool)>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + 'a>> {
    Box::pin(async move {
        let metadata = fs::metadata(current).await?;
        let base = root.parent().unwrap_or(root);

        if metadata.is_file() {
            let rel = current
                .strip_prefix(base)
                .unwrap_or(current)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((current.to_path_buf(), rel, false));
            return Ok(());
        }

        if metadata.is_dir() {
            let mut read_dir = fs::read_dir(current).await?;
            let mut has_children = false;

            while let Some(entry) = read_dir.next_entry().await? {
                has_children = true;
                collect_entries(root, &entry.path(), out).await?;
            }

            if !has_children {
                let rel = current
                    .strip_prefix(base)
                    .unwrap_or(current)
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((current.to_path_buf(), rel, true));
            }
        }

        Ok(())
    })
}
/// ← CORRECTION : retry applicatif supprimé — QUIC gère déjà la fiabilité
pub async fn send_entry(
    connection: Connection,
    abs_path: PathBuf,
    rel_path: String,
    is_dir: bool,
    stop: Arc<AtomicBool>,
    ui: Arc<dyn UI>,
    transfer_id: String,
    total_bytes: u64,
    bytes_sent_counter: Arc<AtomicU64>,
) -> Result<(), ToolError> {
    let (mut send, mut recv) = connection.open_bi().await?;

    if is_dir {
        let metadata = Metadata {
            transfer_id: transfer_id.clone(),
            rel_path,
            size: 0,
            sha256: String::new(),
            is_dir: true,
        };
        write_json_line(&mut send, &metadata).await?;
        let mut ack = [0u8; 1];
        recv.read_exact(&mut ack).await?;
        send.finish()?;
        return Ok(());
    }

    let sha256 = hash_file(&abs_path).await?;
    let size = fs::metadata(&abs_path).await?.len();
    let metadata = Metadata {
        transfer_id: transfer_id.clone(),
        rel_path,
        size,
        sha256: sha256.clone(),
        is_dir: false,
    };
    write_json_line(&mut send, &metadata).await?;

    // Ack métadonnées
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack).await?;
    if ack[0] != ACK {
        return Err(io_err("metadonnees rejetees par le receveur"));
    }

    // Chunks — pas d'ack par chunk : QUIC assure la fiabilite, on pipeline
    let mut file = fs::File::open(&abs_path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = send.reset(0u32.into());
            return Err(io_err("transfert annule par l'utilisateur"));
        }

        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        send.write_all(&[CHUNK]).await?;
        send.write_all(&(n as u32).to_be_bytes()).await?;
        send.write_all(&buf[..n]).await?;

        // Progression locale, sans aller-retour
        let total_sent = bytes_sent_counter.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
        ui.update_progress_bar(&transfer_id, total_sent, total_bytes);
    }

    // Message de complétion
    let complete = CompleteMsg { sha256 };
    write_json_line(&mut send, &complete).await?;

    // Ack final
    let mut final_ack = [0u8; 1];
    recv.read_exact(&mut final_ack).await?;
    send.finish()?;

    if final_ack[0] != ACK {
        return Err(io_err("le receveur a rejete le fichier (hash invalide)"));
    }

    Ok(())
}

async fn write_json_line<T: Serialize>(send: &mut SendStream, value: &T) -> Result<(), ToolError> {
    let mut encoded = serde_json::to_vec(value).map_err(io_err)?;
    encoded.push(b'\n');
    send.write_all(&encoded).await?;
    Ok(())
}

async fn hash_file(path: &Path) -> Result<String, ToolError> {
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}
