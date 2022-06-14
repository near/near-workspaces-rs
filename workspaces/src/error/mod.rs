//! All errors that can occur within workspaces, including but not limited to
//! the following: IO, RPC, parsing, and serialization errors.

mod parse;
mod rpc;

pub use self::parse::{ParseError, ParseErrorKind};
pub use self::rpc::{RpcError, RpcErrorKind};

/// Error type that workspaces will make use of for all the errors
/// returned from this library
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("RPC errored out: {0}")]
    RpcError(#[from] RpcError),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("sandbox has already been started")]
    SandboxAlreadyStarted,
    #[error("sandbox failed to patch state: {0}")]
    SandboxPatchStateFailure(String),
    #[error("sandbox failed to fast forward: {0}")]
    SandboxFastForwardFailure(String),
    #[error("sandbox failed due to: {0}")]
    SandboxUnknownError(String),
    #[error("IO error from {0}")]
    IoError(#[from] std::io::Error),
    #[error("account error from {0}")]
    AccountError(String),
    #[error("parse error from {0}")]
    ParseError(#[from] ParseError),
    #[error("bytes error from {0}")]
    BytesError(#[from] BytesError),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}

/// Bytes specific errors such as serialization and deserialization
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BytesError {
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("borsh error: {0}")]
    BorshError(std::io::Error),
    #[error("failed to decode to base64 due to {0}")]
    DecodeBase64Error(#[from] base64::DecodeError),
}
