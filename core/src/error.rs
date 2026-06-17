use thiserror::Error; 

// ToolError enum avec les variantes :
#[derive(Debug,Error)]
pub enum ToolError{
    #[error("Aucune interface trouver")]
    NoWifiInterface,     // — aucune interface WiFi trouvée

    #[error("Erreur nmcli: {0}")]
    Nmcli(String),       // — erreur nmcli

    #[error("Erreur d'entrée/sortie: {0}")]
    Io(std::io::Error),  // — erreur d'entrée/sortie
    
    #[error("Erreur TLS: {0}")]
    Tls(String),         // — erreur TLS (rustls)

    #[error("Le hash SHA-256 ne correspond pas")]
    Sha256Mismatch ,     // — SHA-256 du fichier reçu ne correspond pas
    #[error("Délai dépassé")]

    Timeout,             // — délai dépassé

    #[error("Opération annulée par l'utilisateur")]
    Cancelled,           // — annulé par l'utilisateur

    #[error("{0}")]
    Custom(String)       // — message personnalisé

}

