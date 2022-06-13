use std::fmt;

// TODO:
// - since account id is public, maybe expose it as-is
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
    ParseError(#[from] crate::types::error::ParseError),
    #[error("bytes error from {0}")]
    BytesError(#[from] BytesError),
    #[error("other error")]
    Other(#[from] Box<dyn std::error::Error>),
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}

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

impl From<RpcErrorKind> for Error {
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
        Self::from_repr(kind, anyhow::anyhow!(msg))
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
