use std::fmt;

// ici c'est la liste des error specifique à toolé
#[derive(Debug)]
pub enum ToolError {
    IoError(std::io::Error),
    Canceled,
    TransfertError,
    CertificateError(rcgen::Error),
    IpaddresError(local_ip_address::Error),
    FormatError(pem::PemError),
    ParseKeyError,
    ConfigQuicError(quinn::rustls::Error),
    AppDireError
}

//là je definit ToolError comme une erreur standard
impl std::error::Error for ToolError {}

// J'affiche maintenant ToolError pour l'utilisateur
impl fmt::Display  for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::IoError(e) => write!(f, "IO error: {}", e),
            ToolError::Canceled => write!(f, "Operation canceled"),
            ToolError::TransfertError => write!(f, "Transfert refusé par le pair"),
            ToolError::CertificateError(e)=>write!(f,"erreur de cerification"),
            ToolError::IpaddresError(e)=>write!(f,"erreur de l'addresse ip"),
            ToolError::FormatError(e)=>write!(f,"erreur de formatage"),
            ToolError::ParseKeyError => write!(f,"erreur de parssage de la cle"),
            ToolError::ConfigQuicError(e)=>write!(f,"erreur de configuration de la connection"),
            ToolError::AppDireError=>write!(f,"erreur de creation des dossier")
        }
    }
}

// je redirige les erreurs <std::io::Error> vers ToolError
impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::IoError(err)
    }
}

// je convertir les erreur de <rcgen::Error> vers ToolError
impl From<rcgen::Error> for ToolError {
    fn from(err:rcgen::Error) -> Self {
        ToolError::CertificateError(err)
    }
}

impl From<local_ip_address::Error> for ToolError {
    fn from(value: local_ip_address::Error) -> Self {
        ToolError::IpaddresError(value)
    }
}

impl From<pem::PemError> for ToolError {
    fn from(value: pem::PemError) -> Self {
        ToolError::FormatError(value)
    }
}

impl From<quinn::rustls::Error> for ToolError {
    fn from(value: quinn::rustls::Error) -> Self {
        ToolError::ConfigQuicError(value)
    }
}
