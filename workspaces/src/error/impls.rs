use std::{borrow::Cow, fmt, io};

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
}

impl Error {
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

    pub fn kind(&self) -> &ErrorKind {
        match &self.repr {
            ErrorRepr::Simple(kind) => kind,
            ErrorRepr::Message { kind, .. } => kind,
            ErrorRepr::Custom { kind, .. } => kind,
            ErrorRepr::Full { kind, .. } => kind,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::custom(ErrorKind::Io, error)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(error: base64::DecodeError) -> Self {
        Self::custom(ErrorKind::DataConversion, error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::custom(ErrorKind::DataConversion, error)
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
            ErrorRepr::Custom { error, .. } => Some(error.as_ref()),
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
