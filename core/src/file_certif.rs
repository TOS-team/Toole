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

use tokio::fs;


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
