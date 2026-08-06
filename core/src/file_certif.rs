use crate::ToolError;
use directories::ProjectDirs;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::net::IpAddr;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug)]
pub struct SkipServerVerification;

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
        ToolError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("key generation failed: {e}"),
        ))
    })?;

    let cert = params.self_signed(&key_pair).map_err(|e| {
        ToolError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("self-sign failed: {e}"),
        ))
    })?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    fs::write(&cert_file, cert_pem.as_bytes()).await?;
    fs::write(&key_file, key_pem.as_bytes()).await?;

    Ok((cert_pem, key_pem))
}

fn data_file() -> Result<(PathBuf, PathBuf), ToolError> {
    let proj_dirs =
        ProjectDirs::from("com", "Tiligre Open Space", "Toole").ok_or(ToolError::AppDirError)?;
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;

    // ← CORRECTION : créer aussi le sous-dossier certs
    let cert_dir = data_dir.join("certs");
    std::fs::create_dir_all(&cert_dir)?;

    Ok((cert_dir.join("key.pem"), cert_dir.join("cert.pem")))
}
