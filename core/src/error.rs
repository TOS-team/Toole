use std::fmt;

// ici c'est la liste des error specifique à toolé
#[derive(Debug)]
enum ToolError {
    IoError(std::io::Error),
    Canceled,
}

//là je definit ToolError comme une erreur standard
impl std::error::Error for ToolError {}

// J'affiche maintenant ToolError pour l'utilisateur
impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::IoError(e) => write!(f, "IO error: {}", e),
            ToolError::Canceled => write!(f, "Operation canceled"),
        }
    }
}

// je redirige les erreurs <std::io::Error> vers ToolError
impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::Io(err)
    }
}