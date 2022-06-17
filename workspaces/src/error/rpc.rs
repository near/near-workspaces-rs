use std::fmt;

use super::Error;

#[derive(Copy, Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RpcErrorKind {
    #[error("unable to create a new account via helper")]
    HelperAccountCreationFailure,
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
    #[error("incorrect variant retrieved while querying (maybe a bug in RPC code?)")]
    QueryReturnedInvalidData,
    #[error("other error not expected from workspaces")]
    Other,
}

impl RpcErrorKind {
    pub(crate) fn with_repr(self, repr: Box<dyn std::error::Error + Send + Sync>) -> RpcError {
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
    repr: Option<Box<dyn std::error::Error + Send + Sync>>,
}

unsafe impl Send for RpcError {}
unsafe impl Sync for RpcError {}
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

    pub(crate) fn from_repr(
        kind: RpcErrorKind,
        repr: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self {
            kind,
            repr: Some(repr),
        }
    }

    pub(crate) fn from_msg(kind: RpcErrorKind, msg: &'static str) -> Self {
        Self::from_repr(kind, anyhow::anyhow!(msg).into())
    }

    /// Get the kind of error that occurred in the RPC service.
    pub fn kind(&self) -> RpcErrorKind {
        self.kind
    }

    /// Get the underlying error message from respective error. This can be
    /// empty to signify no meaningful error message is present.
    fn err_msg(&self) -> String {
        self.repr
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "".to_string())
    }
}
