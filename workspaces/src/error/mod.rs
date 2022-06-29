//! All errors that can occur within workspaces, including but not limited to
//! the following: IO, RPC, parsing, and serialization errors.

mod rpc;

pub use self::rpc::{RpcError, RpcErrorKind};

/// Error type that workspaces will make use of for all the errors
/// returned from this library
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("RPC error: {0}")]
    RpcError(#[from] RpcError),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Sandbox error: {0}")]
    SandboxError(#[from] SandboxError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] SerializationError),
}

/// Bytes specific errors such as serialization and deserialization
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SerializationError {
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("borsh error: {0}")]
    BorshError(std::io::Error),
    #[error("failed to decode to base64 due to {0}")]
    DecodeBase64Error(#[from] base64::DecodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("sandbox has already been started")]
    AlreadyStarted,
    #[error("sandbox could not startup due to: {0}")]
    InitFailure(String),
    #[error("sandbox could not be ran due to: {0}")]
    RunFailure(String),
    #[error("sandbox failed to patch state: {0}")]
    PatchStateFailure(String),
    #[error("sandbox failed to fast forward: {0}")]
    FastForwardFailure(String),
}
