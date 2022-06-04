use std::fmt;

// TODO: for IO error, top level workspaces error can hold instead, where ErrorKind is just
// a blanket view of all custom defined workspace errors.
#[derive(Debug)]
pub struct WorkspaceError {
    kind: WorkspaceErrorKind,
    repr: Option<Box<dyn std::error::Error>>,
}

impl WorkspaceError {
    pub(crate) fn with_kind(kind: WorkspaceErrorKind) -> Self {
        Self { kind, repr: None }
    }

    pub(crate) fn new<E>(kind: WorkspaceErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error>>,
    {
        Self {
            kind,
            repr: Some(error.into()),
        }
    }

    pub(crate) fn simple(kind: WorkspaceErrorKind, msg: &'static str) -> Self {
        Self::new(kind, anyhow::Error::msg(msg))
    }

    pub(crate) fn parse_error<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error>>,
    {
        Self::new(WorkspaceErrorKind::Other, error)
    }

    pub(crate) fn other(error: anyhow::Error) -> Self {
        Self::new(WorkspaceErrorKind::Other, error)
    }

    pub(crate) fn io(error: std::io::Error) -> Self {
        Self::with_kind(WorkspaceErrorKind::IoError(error))
    }

    pub fn kind(&self) -> &WorkspaceErrorKind {
        &self.kind
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: find better way of formatting
        write!(f, "WorkspaceError {{ msg: ")?;
        fmt::Display::fmt(&self.kind, f)?;
        write!(f, " }}")
    }
}

unsafe impl Sync for WorkspaceError {}
unsafe impl Send for WorkspaceError {}
impl std::error::Error for WorkspaceError {}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkspaceErrorKind {
    #[error("failed to connect to rpc service")]
    RpcConnectFail,
    #[error("execution error")]
    ExecutionError,
    #[error("sandbox has already been started")]
    SandboxAlreadyStarted,
    #[error("IO error from {0}")]
    IoError(#[from] std::io::Error),
    #[error("other error")]
    Other,
}

// TODO: better function naming?
impl WorkspaceErrorKind {
    pub(crate) fn into_error(self) -> WorkspaceError {
        WorkspaceError::with_kind(self)
    }

    pub(crate) fn into_error_with_repr<E>(self, err: E) -> WorkspaceError
    where
        E: Into<Box<dyn std::error::Error>>,
    {
        WorkspaceError::new(self, err)
    }
}
