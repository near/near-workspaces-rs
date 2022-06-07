#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkspaceError {
    #[error("failed to connect to rpc service")]
    RpcConnectFail(String),
    #[error("execution error")]
    ExecutionError,
    #[error("sandbox has already been started")]
    SandboxAlreadyStarted,
    #[error("IO error from {0}")]
    IoError(#[from] std::io::Error),
    // TODO: Add Parse specific error
    #[error("Parse error")]
    ParseError,
    #[error("other error")]
    Other(#[from] Box<dyn std::error::Error>),
}

unsafe impl Sync for WorkspaceError {}
unsafe impl Send for WorkspaceError {}

impl WorkspaceError {}
