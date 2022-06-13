use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("workspace error from {0}")]
    WorkspaceError(#[from] WorkspaceError),
    #[error("workspace error from {0}")]
    SerializationError(#[from] SerializationError),
}

// TODO:
// - since account id is public, maybe expose it as-is
// - decide where DecodeError should live. It kind fits in with serialization error
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkspaceError {
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
    #[error("IO error from {0}")]
    IoError(#[from] std::io::Error),
    #[error("account error from {0}")]
    AccountError(String),
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SerializationError {
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("borsh error: {0}")]
    BorshError(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RpcErrorKind {
    #[error("failed to connect to rpc service")]
    ConnectionFailure,
    #[error("access key was unable to be retrieved")]
    UnableToRetrieveAccessKey,
    #[error("unable to broadcast the transaction to the network")]
    BroadcastTxFailure,
    #[error("unable to call into a view function")]
    ViewFunctionFailure,
    #[error("unable to fulfill the query request")]
    QueryFailure,
    #[error("unable to fulfill the block query request")]
    QueryBlockFailure,
    #[error("incorrect variant retrieved while querying (maybe a bug in RPC code?)")]
    QueryReturnedInvalidData,
    #[error("other error not expected from workspaces")]
    Other,
}

impl RpcErrorKind {
    pub(crate) fn with_repr(self, repr: anyhow::Error) -> RpcError {
        RpcError::from_repr(self, repr)
    }

    pub(crate) fn with_msg(self, msg: &'static str) -> RpcError {
        RpcError::from_msg(self, msg)
    }
}

impl From<RpcErrorKind> for RpcError {
    fn from(kind: RpcErrorKind) -> Self {
        RpcError::from_kind(kind)
    }
}

impl From<RpcErrorKind> for WorkspaceError {
    fn from(kind: RpcErrorKind) -> Self {
        RpcError::from_kind(kind).into()
    }
}

/// Possible errors coming from the RPC service.
pub struct RpcError {
    kind: RpcErrorKind,
    repr: Option<anyhow::Error>,
}

impl std::error::Error for RpcError {}

impl fmt::Debug for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind(), self.err_msg())
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind(), self.err_msg())
    }
}

impl RpcError {
    pub(crate) fn from_kind(kind: RpcErrorKind) -> Self {
        Self { kind, repr: None }
    }

    pub(crate) fn from_repr(kind: RpcErrorKind, repr: anyhow::Error) -> Self {
        Self {
            kind,
            repr: Some(repr),
        }
    }

    pub(crate) fn from_msg(kind: RpcErrorKind, msg: &'static str) -> Self {
        Self {
            kind,
            repr: Some(anyhow::anyhow!(msg)),
        }
    }

    pub fn kind(&self) -> &RpcErrorKind {
        &self.kind
    }

    pub fn err_msg(&self) -> String {
        self.repr
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or("".to_string())
    }

    pub(crate) fn inner<'a>(&'a self) -> Option<&'a (dyn std::error::Error + Send + Sync)> {
        if let Some(err) = self.repr.as_ref() {
            Some(err.as_ref())
        } else {
            None
        }
    }
}
