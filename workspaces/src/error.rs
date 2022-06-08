#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace error from {0}")]
    WorkspaceError(WorkspaceError),
    #[error("workspace error from {0}")]
    SerializationError(SerializationError),
}

// TODO:
// - RpcError is a little unwieldy
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkspaceError {
    #[error("failed to connect to rpc service")]
    RpcConnectFail(String),
    #[error("RPC errored out: {0}")]
    RpcError(anyhow::Error),
    #[error("access key was unstable to retrieved")]
    UnableToRetrieveAccessKey,
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("sandbox has already been started")]
    SandboxAlreadyStarted,
    #[error("IO error from {0}")]
    IoError(#[from] std::io::Error),
    // TODO: Add Parse specific error
    #[error("Parse error")]
    ParseError,
    #[error("failed to decode base64 due to {0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("other error")]
    Other(#[from] Box<dyn std::error::Error>),
}

unsafe impl Sync for WorkspaceError {}
unsafe impl Send for WorkspaceError {}

impl WorkspaceError {}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SerializationError {
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("borsh error: {0}")]
    BorshError(std::io::Error),
}
