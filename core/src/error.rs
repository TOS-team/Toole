use std::fmt;


//là j'enumere les differents possible , notament pour la feature discovery pour le moment
#[derive(Debug)]
pub enum ToolError{
    NoWifiInterface,
    Nmcli(String),
    Io(std::io::Error),
    Cancelled,
}

impl std::error::Error for ToolError{}

// J'implemente Display pour ToolError afin de permettre l'affichage en console
impl fmt::Display for ToolError{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)-> fmt::Result{
        match self {
            ToolError::NoWifiInterface => write!(f, "Aucune interface WiFi trouvée"),
            ToolError::Nmcli(msg) => write!(f, "Erreur nmcli : {}", msg),
            ToolError::Io(err) => write!(f, "Erreur d'entrée/sortie : {}", err),
            ToolError::Cancelled => write!(f, "Opération annulée"),
        }
    }
}

//là je j'enveloppe l'erreur <std::io::Error> dans Io de ToolError
impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::Io(err)
    }
}