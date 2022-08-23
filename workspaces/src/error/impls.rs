use std::borrow::Cow;
use std::fmt;

use crate::result::CallExecutionDetails;

use super::{Error, ErrorKind, ErrorRepr, RpcErrorCode, SandboxErrorCode};

impl ErrorKind {
    pub(crate) fn custom<E>(self, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::custom(self, error)
    }

    pub(crate) fn message<T>(self, msg: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::message(self, msg)
    }

    pub(crate) fn detailed<E>(self, details: CallExecutionDetails, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::detailed(self, details, error)
    }
}

impl Error {
    pub(crate) fn detailed<E>(kind: ErrorKind, details: CallExecutionDetails, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            repr: ErrorRepr::Detailed {
                kind,
                details: Box::new(details),
                error: error.into(),
            },
        }
    }

    pub(crate) fn full<T, E>(kind: ErrorKind, msg: T, error: E) -> Self
    where
        T: Into<Cow<'static, str>>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            repr: ErrorRepr::Full {
                kind,
                message: msg.into(),
                error: error.into(),
            },
        }
    }

    pub(crate) fn custom<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self {
            repr: ErrorRepr::Custom {
                kind,
                error: error.into(),
            },
        }
    }

    pub(crate) fn message<T>(kind: ErrorKind, msg: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            repr: ErrorRepr::Message {
                kind,
                message: msg.into(),
            },
        }
    }

    pub(crate) fn simple(kind: ErrorKind) -> Self {
        Self {
            repr: ErrorRepr::Simple(kind),
        }
    }

    /// Get the associated execution details of this error. Usually found with
    /// an error occuring from executing a transaction.
    pub fn details(&self) -> Option<&CallExecutionDetails> {
        match &self.repr {
            ErrorRepr::Detailed { details, .. } => Some(details),
            _ => None,
        }
    }

    /// Returns the corresponding [`ErrorKind`] for this error.
    pub fn kind(&self) -> &ErrorKind {
        match &self.repr {
            ErrorRepr::Simple(kind) => kind,
            ErrorRepr::Message { kind, .. } => kind,
            ErrorRepr::Custom { kind, .. } => kind,
            ErrorRepr::Full { kind, .. } => kind,
            ErrorRepr::Detailed { kind, .. } => kind,
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    ///
    /// If this [`Error`] was constructed via a Custom or Full variant, then
    /// this function will return [`Ok`], otherwise it will return [`Err`].
    pub fn into_inner(self) -> Result<Box<dyn std::error::Error + Send + Sync>, Self> {
        match self.repr {
            ErrorRepr::Custom { error, .. } => Ok(error),
            ErrorRepr::Full { error, .. } => Ok(error),
            ErrorRepr::Detailed { error, .. } => Ok(error),
            _ => Err(self),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.repr {
            ErrorRepr::Custom { error, .. } => error.source(),
            ErrorRepr::Full { error, .. } => error.source(),
            _ => None,
        }
    }
}

impl SandboxErrorCode {
    pub(crate) fn custom<E>(self, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::custom(ErrorKind::Sandbox(self), error)
    }
}

impl From<SandboxErrorCode> for Error {
    fn from(code: SandboxErrorCode) -> Self {
        Error::simple(ErrorKind::Sandbox(code))
    }
}

impl RpcErrorCode {
    pub(crate) fn message<T>(self, msg: T) -> Error
    where
        T: Into<Cow<'static, str>>,
    {
        Error::message(ErrorKind::Rpc(self), msg)
    }

    pub(crate) fn custom<E>(self, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::custom(ErrorKind::Rpc(self), error)
    }
}

impl From<RpcErrorCode> for Error {
    fn from(code: RpcErrorCode) -> Self {
        Error::simple(ErrorKind::Rpc(code))
    }
}
