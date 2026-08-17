use crate::ToolError;
use directories::ProjectDirs;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use tokio::fs;

/// vérificateur de certificat par empreinte épinglée (TOFU) : au premier
/// contact je n'attends aucune empreinte (expected = None) et j'accepte pour
/// la mémoriser ensuite ; aux contacts suivants je refuse tout certificat dont
/// l'empreinte diffère de celle épinglée pour ce pair (identité changée =
/// attaque de l'homme du milieu probable)
#[derive(Debug)]
pub struct PinnedServerVerifier {
    pub expected: Option<String>,
}

impl rustls::client::danger::ServerCertVerifier for PinnedServerVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let fp = fingerprint(end_entity.as_ref());
        if let Some(expected) = &self.expected {
            if *expected != fp {
                return Err(rustls::Error::General(format!(
                    "empreinte du pair ({fp}) differente de celle epinguee ({expected}) : attaque MITM ?"
                )));
            }
        }
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

/// empreinte SHA-256 (hex) du certificat : elle identifie la clé du pair tant
/// que le certificat persisté est réutilisé (c'est le cas : certificat() relit
/// cert.pem au démarrage). Je hache le DER complet, pas le SPKI, pour rester
/// sur la donnée déjà exposée par le handshake rustls
pub fn fingerprint(cert_der: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cert_der);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// empreinte du certificat présenté par le pair sur une connexion QUIC
/// établie : le handshake ayant réussi, le cert est celui que le pair a servi
pub fn peer_fingerprint(connection: &quinn::Connection) -> Option<String> {
    let identity = connection.peer_identity()?;
    let certs = identity
        .downcast::<Vec<rustls::pki_types::CertificateDer<'static>>>()
        .ok()?;
    let cert = certs.first()?;
    Some(fingerprint(cert.as_ref()))
}

/// empreinte épinglée pour un device_id, si elle existe (None = premier
/// contact). Un fichier absent ou illisible retombe sur None : au pire on
/// re-pinne, c'est dégradé mais jamais bloquant
pub fn pin_for(device_id: &str) -> Option<String> {
    let raw = std::fs::read_to_string(pins_path().ok()?).ok()?;
    let pins: HashMap<String, String> = serde_json::from_str(&raw).ok()?;
    pins.get(device_id).cloned()
}

/// j'épingle (ou je ré-épingle) l'empreinte d'un device_id dans le fichier
/// d'empreintes persisté
pub fn save_pin(device_id: &str, fingerprint: &str) -> Result<(), ToolError> {
    let path = pins_path()?;
    let mut pins: HashMap<String, String> = match std::fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => HashMap::new(),
    };
    pins.insert(device_id.to_string(), fingerprint.to_string());
    let raw = serde_json::to_string_pretty(&pins)?;
    std::fs::write(path, raw)?;
    Ok(())
}

fn pins_path() -> Result<PathBuf, ToolError> {
    Ok(data_dir()?.join("pins.json"))
}

pub async fn certificat() -> Result<(String, String), ToolError> {
    let (key_file, cert_file) = data_file()?;

    if key_file.exists() && cert_file.exists() {
        let cert_pem = fs::read_to_string(&cert_file).await?;
        let key_pem = fs::read_to_string(&key_file).await?;
        return Ok((cert_pem, key_pem));
    }

    let my_local_ip = local_ip_address::local_ip().unwrap_or(IpAddr::from([127, 0, 0, 1]));

    let mut params: CertificateParams = Default::default();
    params.not_before = date_time_ymd(2026, 1, 1);
    params.not_after = date_time_ymd(2036, 1, 1);

    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Toolé");
    params
        .distinguished_name
        .push(DnType::CommonName, "Toolé P2P Server");

    params.subject_alt_names = vec![
        SanType::DnsName("localhost".try_into().map_err(|_| {
            ToolError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid DNS name",
            ))
        })?),
        SanType::IpAddress(my_local_ip),
    ];

    let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).map_err(|e| {
        ToolError::IoError(std::io::Error::other(format!("key generation failed: {e}")))
    })?;

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| ToolError::IoError(std::io::Error::other(format!("self-sign failed: {e}"))))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    fs::write(&cert_file, cert_pem.as_bytes()).await?;
    fs::write(&key_file, key_pem.as_bytes()).await?;

    // je restreins l'accès à la clé privée (lecture seule pour l'utilisateur
    // courant) : un autre compte local ne doit pas pouvoir la récupérer
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600)).await;
    }

    Ok((cert_pem, key_pem))
}

fn data_dir() -> Result<PathBuf, ToolError> {
    let proj_dirs =
        ProjectDirs::from("com", "Tiligre Open Space", "Toole").ok_or(ToolError::AppDirError)?;
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    Ok(data_dir.to_path_buf())
}

fn data_file() -> Result<(PathBuf, PathBuf), ToolError> {
    let data_dir = data_dir()?;

    // je créé aussi le sous-dossier certs
    let cert_dir = data_dir.join("certs");
    std::fs::create_dir_all(&cert_dir)?;

    Ok((cert_dir.join("key.pem"), cert_dir.join("cert.pem")))
}
