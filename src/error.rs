//! Error types for the archlinux-keyring component.

/// Errors in keyring operations.
#[derive(Debug, thiserror::Error)]
pub enum KeyringError {
    /// Invalid fingerprint format.
    #[error("invalid fingerprint: {0}")]
    InvalidFingerprint(String),

    /// Key not found.
    #[error("key not found: {0}")]
    NotFound(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
