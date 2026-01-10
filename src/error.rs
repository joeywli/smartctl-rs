#[derive(Debug, thiserror::Error)]
pub enum SmartCtlError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Smartctl command failed: {0}")]
    CommandFailed(String),
    #[error("Smartctl not found (ensure `smartmontools` is installed)")]
    NotFound,
}
