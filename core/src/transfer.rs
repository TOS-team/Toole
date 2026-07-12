use crate::{ToolError, UI};
use directories::ProjectDirs;
use local_ip_address::local_ip;
use quinn::{ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Error as IoError, ErrorKind};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
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
const TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RETRIES: u8 = 3;
const PORT: u16 = 58200;
const CERT_PATH: &str = "certs/cert.pem";
const KEY_PATH: &str = "certs/key.pem";

// ------------------------------------------------------------
// En gros ici je prepare la configuration  des endpoints pour le Sender et le Recever
// ------------------------------------------------------------

async fn make_server_endpoint() -> Result<Endpoint, ToolError> {
    let (cert_pem, key_pem) = certificat().await?;
    let mut cert_reader = std::io::BufReader::new(cert_pem.as_bytes());
    let mut key_reader = std::io::BufReader::new(key_pem.as_bytes());

    let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    let key = match rustls_pemfile::private_key(&mut key_reader)? {
        Some(v) => v,
        None => return Err(ToolError::ParseKeyError),
    };

    let server_config = ServerConfig::with_single_cert(certs, key)?;

    // On ecoute sur toutes les interfaces, port fixe (58200)
    let bind_addr: SocketAddr = format!("0.0.0.0:{PORT}").parse().map_err(io_err)?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

// Verificateur qui accepte n'importe quel certificat presente par le pair.
// Necessaire car chaque pair genere son propre certificat auto-signe (pas de
// CA commune). Le chiffrement TLS 1.3 reste entier : seule la verification
// d'IDENTITE du pair est desactivee, ce qui est acceptable ici car l'IP du
// destinataire est choisie explicitement par l'utilisateur (pairing direct),
// pas decouverte publiquement. Quand on vas passer a une version TCP  on vas changer sa 
#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn make_client_endpoint() -> Result<Endpoint, ToolError> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(io_err)?,
    ));

    let bind_addr: SocketAddr = "0.0.0.0:0".parse().map_err(io_err)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

fn io_err<E: std::fmt::Display>(e: E) -> ToolError {
    IoError::new(ErrorKind::Other, e.to_string()).into()
}

// ------------------------------------------------------------
// Receveur
// ------------------------------------------------------------

pub async fn start_receiver(
    ui: &dyn UI,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let endpoint = make_server_endpoint().await?;
    ui.log(&format!("Recepteur en ecoute sur le port {PORT}"));

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        // On attend soit une connexion entrante, soit un tick de 500ms pour
        // rester reactif au signal d'arret (evite de bloquer indefiniment
        // sur endpoint.accept()).
        let incoming = tokio::select! {
            conn = endpoint.accept() => conn,
            _ = tokio::time::sleep(Duration::from_millis(500)) => continue,
        };

        let Some(connecting) = incoming else {
            break; // endpoint ferme cote local
        };

        let dest_dir = dest_dir.clone();
        let stop = stop.clone();

        tokio::spawn(async move {
            match connecting.await {
                Ok(connection) => {
                    if let Err(e) = handle_incoming_connection(connection, dest_dir, stop).await {
                        eprintln!("Erreur de connexion receveur: {e}");
                    }
                }
                Err(e) => eprintln!("Handshake QUIC echoue😭:  {e}"),
            }
        });
    }

    endpoint.close(0u32.into(), b"arret");
    Ok(())
}

async fn handle_incoming_connection(
    connection: Connection,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    loop {
        if stop.load(Ordering::Relaxed) {
            connection.close(0u32.into(), b"arret utilisateur");
            return Ok(());
        }

        // Chaque fichier/dossier arrive sur un stream bidirectionnel dedie.
        match connection.accept_bi().await {
            Ok((send, recv)) => {
                let dest_dir = dest_dir.clone();
                tokio::spawn(async move {
                    if let Err(e) = receive_one(send, recv, &dest_dir).await {
                        eprintln!("Erreur reception fichier: {e}");
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_))
            | Err(quinn::ConnectionError::LocallyClosed) => return Ok(()),
            Err(e) => return Err(e.into()),
        }
    }
}

async fn receive_one(
    mut send: SendStream,
    mut recv: RecvStream,
    dest_dir: &Path,
) -> Result<(), ToolError> {
    // je prepare les  Metadonnees 
    let metadata = read_json_line::<Metadata>(&mut recv).await?;
    let full_path = dest_dir.join(&metadata.rel_path);

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    if metadata.is_dir {
        fs::create_dir_all(&full_path).await?;
        send.write_all(&[ACK]).await?;
        send.finish()?;
        return Ok(());
    }

    //  Ack des metadonnees 
    send.write_all(&[ACK]).await?;

    //   Chunks + hachage progressif 
    let mut out_file = fs::File::create(&full_path).await?;
    let mut hasher = Sha256::new();
    let mut received: u64 = 0;

    while received < metadata.size {
        let mut marker = [0u8; 1];
        recv.read_exact(&mut marker).await?;
        if marker[0] != CHUNK {
            return Err(io_err("trame inattendue (protocole desynchronise)"));
        }

        let mut idx_buf = [0u8; 4];
        recv.read_exact(&mut idx_buf).await?;

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await?;

        hasher.update(&data);
        out_file.write_all(&data).await?;
        received += len as u64;

        // Ack du chunk : le sender s'en sert pour son mecanisme de retry
        send.write_all(&idx_buf).await?;
    }

    out_file.flush().await?;

    //  6. Message de completion 
    let complete = read_json_line::<CompleteMsg>(&mut recv).await?;
    let actual_hash = hex::encode(hasher.finalize());

    //  7. Ack final 
    if actual_hash == complete.sha256 && actual_hash == metadata.sha256 {
        send.write_all(&[ACK]).await?;
    } else {
        send.write_all(&[REJECT]).await?;
        let _ = fs::remove_file(&full_path).await; // fichier corrompu, on ne le garde pas
    }

    send.finish()?;
    Ok(())
}

async fn read_json_line<T: for<'de> Deserialize<'de>>(
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

// ---------------------------------------------------------
// Emetteur en gors le sender
// ---------------------------------------------------------

pub async fn start_sender(
    ui: &dyn UI,
    paths: Vec<PathBuf>,
    peer_addr: SocketAddr,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let endpoint = make_client_endpoint()?;

    // "localhost" : nom SNI attendu par le certificat auto-signe (voir SanType::DnsName
    // dans certificat()). La verification etant desactivee (SkipServerVerification),
    // ce nom n'a pas besoin de correspondre a une vraie resolution DNS.
    let connecting = endpoint.connect(peer_addr, "localhost").map_err(io_err)?;
    let connection = connecting.await?;
    ui.log(&format!("Connecte a {peer_addr}"));

    // Parcours recursif prealable : on liste toutes les entrees (fichiers et
    // dossiers vides) avant d'ouvrir les streams, pour pouvoir les envoyer
    // en parallele.
    let mut entries = Vec::new();
    for path in &paths {
        collect_entries(path, path, &mut entries).await?;
    }

    let mut handles = Vec::new();
    for (abs_path, rel_path, is_dir) in entries {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let connection = connection.clone();
        let stop = stop.clone();
        handles.push(tokio::spawn(async move {
            send_entry(connection, abs_path, rel_path, is_dir, stop).await
        }));
    }

    for handle in handles {
        if let Err(e) = handle.await.map_err(io_err)? {
            eprintln!("Erreur d'envoi: {e}");
        }
    }

    connection.close(0u32.into(), b"transfert termine");
    endpoint.wait_idle().await;
    Ok(())
}

// En gros ici on fait un  Parcours recursif manuel (pas de dependance a walkdir) : accumule
// (chemin absolu, chemin relatif, is_dir) pour chaque fichier et chaque
// dossier vide rencontre.
fn collect_entries<'a>(
    root: &'a Path,
    current: &'a Path,
    out: &'a mut Vec<(PathBuf, String, bool)>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + 'a>> {
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

async fn send_entry(
    connection: Connection,
    abs_path: PathBuf,
    rel_path: String,
    is_dir: bool,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let (mut send, mut recv) = connection.open_bi().await?;

    if is_dir {
        let metadata = Metadata {
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

    // Hash prealable : necessaire pour l'annoncer dans les metadonnees et
    // pour la verification finale cote receveur.
    let sha256 = hash_file(&abs_path).await?;
    let size = fs::metadata(&abs_path).await?.len();

    let metadata = Metadata {
        rel_path,
        size,
        sha256: sha256.clone(),
        is_dir: false,
    };
    write_json_line(&mut send, &metadata).await?;

    //  Ack des metadonnees 
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack).await?;
    if ack[0] != ACK {
        return Err(io_err("metadonnees rejetees par le receveur"));
    }

    //  Chunks avec retry sur timeout 
    let mut file = fs::File::open(&abs_path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut chunk_index: u32 = 0;

    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = send.reset(0u32.into());
            return Err(io_err("transfert annule par l'utilisateur"));
        }

        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        let mut attempts = 0u8;
        loop {
            send.write_all(&[CHUNK]).await?;
            send.write_all(&chunk_index.to_be_bytes()).await?;
            send.write_all(&(n as u32).to_be_bytes()).await?;
            send.write_all(&buf[..n]).await?;

            let mut ack_buf = [0u8; 4];
            match tokio::time::timeout(TIMEOUT, recv.read_exact(&mut ack_buf)).await {
                Ok(Ok(())) => {
                    let acked = u32::from_be_bytes(ack_buf);
                    if acked != chunk_index {
                        return Err(io_err(format!(
                            "ack incoherent: attendu {chunk_index}, recu {acked}"
                        )));
                    }
                    break; // chunk confirme, on passe au suivant
                }
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => {
                    attempts += 1;
                    if attempts >= MAX_RETRIES {
                        return Err(io_err(format!(
                            "aucun ack recu pour le chunk {chunk_index} apres {MAX_RETRIES} tentatives"
                        )));
                    }
                    // sinon on reboucle et on renvoie le meme chunk
                }
            }
        }

        chunk_index += 1;
    }

    //  6. Message de completion 
    let complete = CompleteMsg { sha256 };
    write_json_line(&mut send, &complete).await?;

    //  7. Ack final 
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

// ---------------------------------------------------------
// Certificats
// ---------------------------------------------------------

pub async fn certificat() -> Result<(String, String), ToolError> {
    let (key_file, cert_file) = data_file()?;

    if key_file.exists() && cert_file.exists() {
        let cert_pem = std::fs::read_to_string(cert_file)?;
        let key_pem = std::fs::read_to_string(key_file)?;
        return Ok((cert_pem, key_pem));
    }

    let my_local_ip = local_ip()?;
    let mut params: CertificateParams = Default::default();
    params.not_before = date_time_ymd(2026, 1, 1);
    params.not_after = date_time_ymd(4096, 1, 1);

    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Toole");
    params
        .distinguished_name
        .push(DnType::CommonName, "Serveur QUIC");

    params.subject_alt_names = vec![
        SanType::DnsName("localhost".try_into()?),
        SanType::IpAddress(my_local_ip),
    ];

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    fs::write(&cert_file, cert_pem.as_bytes()).await?;
    fs::write(&key_file, key_pem.as_bytes()).await?;

    Ok((cert_pem, key_pem))
}

fn data_file() -> Result<(PathBuf, PathBuf), ToolError> {
    let proj_dirs = match ProjectDirs::from("com", "Tiligre Open Space", "Toole") {
        Some(value) => value,
        None => return Err(ToolError::AppDirError),
    };
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    Ok((data_dir.join(KEY_PATH), data_dir.join(CERT_PATH)))
} 