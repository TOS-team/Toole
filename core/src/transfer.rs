use crate::file_certif::{certificat, SkipServerVerification};
use crate::{ToolError, UI};
use quinn::{ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use serde::{Deserialize, Serialize};
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub transfer_id: String,
    pub rel_path: String,
    pub size: u64,
    pub is_dir: bool,
}

/// en-tête de lot envoyé par l'émetteur sur le premier flux de la connexion :
/// le récepteur connaît le transfer_id et le total du lot à l'avance, afin
/// d'afficher la même progression globale que l'émetteur dès le premier fichier
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchHeader {
    pub transfer_id: String,
    pub total_bytes: u64,
}

const CHUNK_SIZE: usize = 1_048_576; // 1 Mo
pub const ACK: u8 = 0x01;
const COMPLETE: u8 = 0x02;
const PORT: u16 = 58200;

/// limite les emissions de progression UI a ~20/s : le webview n'a pas besoin
/// de 190 evenements/s et chaque IPC coûte cher en boucle serree
struct UiThrottle {
    last: Instant,
}

impl UiThrottle {
    fn new() -> Self {
        UiThrottle {
            last: Instant::now(),
        }
    }

    /// true si au moins 50ms se sont ecoulees depuis la derniere emission
    fn ready(&mut self) -> bool {
        if self.last.elapsed() >= Duration::from_millis(50) {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}

pub fn io_err<E: std::fmt::Display>(e: E) -> ToolError {
    IoError::other(e.to_string()).into()
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
/// initial_rtt garde au defaut quinn (333ms) : une valeur optimiste fait partir le
/// pacing trop vite et provoque des pertes puis l'effondrement de Cubic sur WiFi
fn transport_config() -> quinn::TransportConfig {
    use quinn::VarInt;
    let mut t = quinn::TransportConfig::default();
t.stream_receive_window(VarInt::from(8u32 * 1024 * 1024));
        t.receive_window(VarInt::from(32u32 * 1024 * 1024));
        t.send_window(32 * 1024 * 1024);
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
    // je mémorise si un flux a échoué en cours de route (ex: annulation côté
    // émetteur) : dans ce cas je signalerai une erreur, pas une réception
    let had_error = Arc::new(AtomicBool::new(false));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let mut header_received = false;
    loop {
        if stop.load(Ordering::Relaxed) {
            connection.close(0u32.into(), b"arret utilisateur");
            return Ok(());
        }

        match tokio::time::timeout(Duration::from_secs(1), connection.accept_bi()).await {
            Ok(Ok((mut send, mut recv))) => {
                // le premier flux est l'en-tête de lot : il apporte transfer_id
                // et total d'avance, pour que la progression du récepteur ait
                // le même dénominateur que celle de l'émetteur dès le départ
                if !header_received {
                    header_received = true;
                    let header: BatchHeader = read_json_line(&mut recv).await?;
                    send.write_all(&[ACK]).await?;
                    send.finish()?;
                    {
                        let mut tid = transfer_id.lock().unwrap_or_else(|e| e.into_inner());
                        if tid.is_none() {
                            *tid = Some(header.transfer_id.clone());
                        }
                    }
                    total.store(header.total_bytes, Ordering::Relaxed);
                    ui.show_progress_bar(&header.transfer_id);
                    continue;
                }

                let dest_dir = dest_dir.clone();
                let files = files.clone();
                let bytes = bytes.clone();
                let ui = ui.clone();
                let transfer_id = transfer_id.clone();
                let total = total.clone();
                let had_error = had_error.clone();
                handles.push(tokio::spawn(async move {
                    if let Err(e) =
                        receive_one(send, recv, &dest_dir, files, bytes, ui, transfer_id, total)
                            .await
                    {
                        // je marque l'échec pour que la connexion soit
                        // signalée en erreur (pas comme un transfert reçu)
                        had_error.store(true, Ordering::Relaxed);
                        eprintln!("Erreur reception fichier: {e}");
                    }
                }));
            }
            Ok(Err(quinn::ConnectionError::ApplicationClosed(_)))
            | Ok(Err(quinn::ConnectionError::LocallyClosed)) => {
                // j'attends la fin des tâches de flux : elles posent
                // `had_error` si un fichier a échoué, et elles terminent vite
                // une fois la connexion fermée
                for h in handles {
                    let _ = h.await;
                }
                // si un flux a échoué (ex: annulation côté émetteur), je
                // signale une erreur plutôt qu'une réception
                return if had_error.load(Ordering::Relaxed) {
                    Err(io_err("connexion fermee avec un flux en echec"))
                } else {
                    Ok(())
                };
            }
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

    // l'id du transfert vient de l'en-tête de lot ; en secours (protocole
    // ancien) je le prends du metadata, sans redéclencher la barre de
    // progression qui a été signalée par l'en-tête
    {
        let mut tid = transfer_id.lock().unwrap_or_else(|e| e.into_inner());
        if tid.is_none() {
            *tid = Some(metadata.transfer_id.clone());
        }
    }
    let tid = transfer_id.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default();

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
    let mut received: u64 = 0;
    let mut throttle = UiThrottle::new();

    while received < metadata.size {
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await?;

        out_file.write_all(&data).await?;
        received += len as u64;

        // progression cote receveur, aluminee sur celle de l'emetteur
        let done = bytes.fetch_add(len as u64, Ordering::Relaxed) + len as u64;
        if throttle.ready() || received == metadata.size {
            ui.update_progress_bar(&tid, done, total.load(Ordering::Relaxed));
            ui.file_progress_bar(&tid, &name, received, metadata.size);
        }
    }

    out_file.flush().await?;

    // Marqueur de complétion, puis ack final
    let mut complete = [0u8; 1];
    recv.read_exact(&mut complete).await?;
    if complete[0] != COMPLETE {
        return Err(io_err(
            "fin de fichier inattendue (protocole desynchronise)",
        ));
    }
    send.write_all(&[ACK]).await?;

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

/// envoie un fichier ou dossier dans la connexion QUIC établie : les paquets
/// sont fiables grâce à QUIC (pas de retry applicatif), je copie le contenu en
/// blocs et je signale la progression via l'interface UI.
#[allow(clippy::too_many_arguments)]
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
            is_dir: true,
        };
        write_json_line(&mut send, &metadata).await?;
        let mut ack = [0u8; 1];
        recv.read_exact(&mut ack).await?;
        send.finish()?;
        return Ok(());
    }

    let file_name = Path::new(&rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.clone());

    let size = fs::metadata(&abs_path).await?.len();
    let metadata = Metadata {
        transfer_id: transfer_id.clone(),
        rel_path,
        size,
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
    let mut file_sent: u64 = 0;
    let mut throttle = UiThrottle::new();

    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = send.reset(0u32.into());
            return Err(io_err("transfert annule par l'utilisateur"));
        }

        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        send.write_all(&(n as u32).to_be_bytes()).await?;
        send.write_all(&buf[..n]).await?;

        // Progression globale cumulee (lot) + per-fichier
        file_sent += n as u64;
        let total_sent = bytes_sent_counter.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
        if throttle.ready() || file_sent == size {
            ui.update_progress_bar(&transfer_id, total_sent, total_bytes);
            ui.file_progress_bar(&transfer_id, &file_name, file_sent, size);
        }
    }

    // Marqueur de complétion
    send.write_all(&[COMPLETE]).await?;

    // Ack final
    let mut final_ack = [0u8; 1];
    recv.read_exact(&mut final_ack).await?;
    send.finish()?;

    if final_ack[0] != ACK {
        return Err(io_err("le receveur a rejete le fichier"));
    }

    Ok(())
}

pub async fn write_json_line<T: Serialize>(send: &mut SendStream, value: &T) -> Result<(), ToolError> {
    let mut encoded = serde_json::to_vec(value).map_err(io_err)?;
    encoded.push(b'\n');
    send.write_all(&encoded).await?;
    Ok(())
}
