use crate::{ToolError, UI};
use local_ip_address::local_ip;
use quinn::{Endpoint, ServerConfig};
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncReadExt, BufReader};
use directories::ProjectDirs;

struct Metadata {
    rel_path: String,
    size: u64,
    sha256: String,
    is_dir: bool,
}

pub struct CompleteMsg {
    pub sha256: String,
}

const CHUNK_SIZE: usize = 1_048_576; // 1 Mo
const ACK: u8 = 0x01;
const REJECT: u8 = 0x00;
const CHUNK: u8 = 0x02;
const TIMEOUT: Duration = Duration::from_secs(10);
const PORT: u16 = 58200;
const CERT_PATH: &str = "certs/cert.pem";
const KEY_PATH: &str = "certs/key.pem";

// Transfert de fichiers par QUIC (via Quinn).
//
// Pourquoi QUIC plutot que TCP ?
//   - Multiplexage natif : plusieurs fichiers en parallele sur une seule connexion
//   - Pas de Head-of-Line blocking
//   - Chiffrement TLS 1.3 integré (aucune config manuelle)
//   - Controle de congestion, perte de paquets, renvoi automatique
//
// Principe :
//   1. Connexion QUIC etablie entre deux pairs (client/serveur)
//   2. Chaque fichier = un bi-directional stream QUIC dedie
//   3. Les streams circulent en parallele sur la meme connexion
//   4. Les dossiers sont transmis recursivement (un stream par fichier)
//
// Protocole applicatif (par stream) :
//   1. Sender → Receiver : Metadata JSON (rel_path, size, sha256, is_dir) + \n
//   2. Receiver → Sender : Ack 0x01
//   3. Sender → Receiver : Chunk (chunk_index u32 big-endian + data)
//   4. Receiver → Sender : Ack chunk_index u32 big-endian
//   5. Repeter 3-4 jusqu'au dernier chunk
//   6. Sender → Receiver : Complete JSON { sha256 } + \n
//   7. Receiver → Sender : FinalAck 0x01 (OK) ou 0x00 (REJET)
//
// Metadata JSON :
//   { "rel_path": "photos/vacances/img1.jpg", "size": 104857600,
//     "sha256": "e3b0c44...", "is_dir": false }
//
// Taille de chunk : 1 048 576 octets (1 Mo)
// Timeout renvoi : 10s, 3 tentatives max par chunk
// SHA-256 calcule progressivement cote Receiver, compare a la fin
//
// Connexion : QUIC sur le port 58200
// - Chaque pair demarre un serveur QUIC au lancement
// - L'envoi ouvre une connexion sortante vers l'IP du destinataire
// - Handshake TLS 1.3 automatique via Quinn
//
// Pour un dossier :
//   - Le sender parcourt recursivement l'arborescence
//   - Ouvre un stream QUIC dedie pour chaque fichier
//   - rel_path conserve la structure (ex: "photos/vacances/img1.jpg")
//   - Les streams sont ouverts en parallele (multiplexage QUIC)
//   - Les dossiers vides sont signales par is_dir=true sans chunks
//
// Gestion des erreurs :
//   - Si aucun Ack recu dans les 10s, renvoi du chunk (max 3 fois)
//   - Si SHA-256 mismatch, FinalAck 0x00 + fichier supprime cote receiver
//   - Annulation : fermeture du stream, le receiver ignore les residus
//
// start_sender(ui, paths: Vec<PathBuf>, peer_addr, stop)
// start_receiver(ui, dest_dir, stop)
// ici je genere le certificat auto-signé pour le sender

fn config_endpoint(peer_addr: SocketAddr,)->Result<Endpoint,ToolError>{
    let (cert_pem, key_pem) = certificat()?;
    let mut cert_reader = std::io::BufReader::new(cert_pem.as_bytes());
    let mut key_reader = std::io::BufReader::new(key_pem.as_bytes());

    let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    let key = match rustls_pemfile::private_key(&mut key_reader)? {
        Some(v) => v,
        None => return Err(ToolError::ParseKeyError),
    };

    let cert = certs[0].clone();

    let server_config = ServerConfig::with_single_cert(vec![cert], key)?;
    let endpoint = Endpoint::server(server_config, peer_addr)?;
    Ok(endpoint)
}

pub async fn start_sender(
    ui: &dyn UI,
    paths: Vec<PathBuf>,
    peer_addr: SocketAddr,
    stop: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    
    let endpoint =config_endpoint(peer_addr)?;

    while let Some(conn) = endpoint.accept().await {
        let connection = conn.await?;
    }

    Ok(())
}

pub fn certificat() -> Result<(String, String), ToolError> {

    let (key_file,cert_file) = data_file()?;

    if key_file.exists() && cert_file.exists() {
        // Lecture directe des fichiers texte
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

    // Sauvegarde physique pour les prochains démarrages
    std::fs::write(cert_file, cert_pem.as_bytes())?;
    std::fs::write(key_file, key_pem.as_bytes())?;

    // Retourne le tuple contenant (certificat, clé)
    Ok((cert_pem, key_pem))
}

fn data_file()-> Result<(PathBuf,PathBuf), ToolError>{
    let proj_dirs = match ProjectDirs::from("com","Tiligre Open Space","Toole")  {
        Some(value)=>value,
        None=>return Err(ToolError::AppDireError)
    };
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    Ok((data_dir.join(KEY_PATH),data_dir.join(CERT_PATH)))
}

