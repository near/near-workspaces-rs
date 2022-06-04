// use std::fmt;

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

    pub fn kind(&self) -> &WorkspaceErrorKind {
        &self.kind
    }
}

// impl fmt::Display for WorkspaceError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let body = if let Some(inner) = self.repr {
//             Display::fmt(&self.inner, f)
//         } else {

//         }

//         // match self.kind() {
//         //     WorkspaceErrorKind::ExecutionError => write!(f, "Workspace(ExeuctionError):"),
//         //     WorkspaceErrorKind::ParseError => write!(f, "Workspace(ParseError):"),
//         // }
//     }
// }

// impl std::error::Error for WorkspaceError {}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkspaceErrorKind {
    #[error("execution error")]
    ExecutionError,
    #[error("sandbox has already been started")]
    SandboxAlreadyStarted,
    #[error("other error")]
    Other,
}

impl WorkspaceErrorKind {
    pub(crate) fn into_error(self) -> WorkspaceError {
        WorkspaceError::with_kind(self)
    }
}
