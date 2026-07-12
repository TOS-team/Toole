use quinn::{ConnectError, ConnectionError, ReadExactError, WriteError};
use thiserror::Error;


// Liste des erreurs specifiques à Toolé
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("erreur IO: {0}")]
    IoError(#[from] std::io::Error),

    #[error("operation annulee")]
    Canceled,

    #[error("transfert refuse par le pair")]
    TransfertError,

    #[error("erreur de certification: {0}")]
    CertificateError(#[from] rcgen::Error),

    #[error("erreur de l'adresse ip: {0}")]
    IpAddressError(#[from] local_ip_address::Error),

    #[error("erreur de formatage: {0}")]
    FormatError(#[from] pem::PemError),

    #[error("erreur de parsage de la cle")]
    ParseKeyError,

    #[error("erreur de configuration de la connexion: {0}")]
    ConfigQuicError(#[from] quinn::rustls::Error),

    #[error("erreur de creation des dossiers")]
    AppDirError,

    #[error("erreur de connexion: {0}")]
    ConnectionError(#[from] ConnectionError),

    #[error("erreur d'ecriture sur le stream: {0}")]
    WriteError(#[from] WriteError),

    #[error("erreur de lecture sur le stream: {0}")]
    ReadExactError(#[from] ReadExactError),

    #[error("erreur d'etablissement de connexion: {0}")]
    ConnectError(#[from] ConnectError),

    #[error("erreur de (de)serialisation JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("hash du fichier recu ne correspond pas au hash attendu")]
    HashMismatch,

    #[error("ack incoherent: attendu {expected}, recu {got}")]
    AckMismatch { expected: u32, got: u32 },

    #[error("aucun ack recu pour le chunk {chunk_index} apres {attempts} tentatives")]
    AckTimeout { chunk_index: u32, attempts: u8 },

    #[error("trame inattendue recue (protocole desynchronise)")]
    UnexpectedFrame,

    #[error("{0}")]
    Protocol(String),

    #[error("fermeture du stream: {0}")]
    CloseStream(quinn::ClosedStream)

}

impl From<quinn::ClosedStream> for ToolError {
    fn from(value: quinn::ClosedStream) -> Self {
        ToolError::CloseStream(value)
    }
}