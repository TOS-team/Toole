use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse private key")]
    ParseKeyError,

    #[error("Failed to get app directories")]
    AppDirError,

    #[error("QUIC error: {0}")]
    QuinnError(String),

    #[error("Transfer error: {0}")]
    TransferError(String),

    #[error("Stream closed")]
    ClosedStream(#[from] quinn::ClosedStream),

    #[error("TLS error: {0}")]
    TlsError(#[from] rustls::Error),
}

impl From<quinn::ConnectionError> for ToolError {
    fn from(e: quinn::ConnectionError) -> Self {
        ToolError::QuinnError(e.to_string())
    }
}

impl From<quinn::WriteError> for ToolError {
    fn from(e: quinn::WriteError) -> Self {
        ToolError::QuinnError(e.to_string())
    }
}

impl From<quinn::ReadError> for ToolError {
    fn from(e: quinn::ReadError) -> Self {
        ToolError::QuinnError(e.to_string())
    }
}

// quinn::ReadExactError (pas tokio::io::ReadExactError)
impl From<quinn::ReadExactError> for ToolError {
    fn from(e: quinn::ReadExactError) -> Self {
        ToolError::QuinnError(e.to_string())
    }
}

impl From<rcgen::Error> for ToolError {
    fn from(e: rcgen::Error) -> Self {
        ToolError::TransferError(e.to_string())
    }
}
