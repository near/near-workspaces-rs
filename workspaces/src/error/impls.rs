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

    /// Returns the corresponding [`ErrorKind`] for this error.
    pub fn kind(&self) -> &ErrorKind {
        match &self.repr {
            ErrorRepr::Simple(kind) => kind,
            ErrorRepr::Message { kind, .. } => kind,
            ErrorRepr::Custom { kind, .. } => kind,
            ErrorRepr::Full { kind, .. } => kind,
        }
    }

    /// Consumes the `Error`, returning its inner error (if any).
    ///
    /// If this [`Error`] was constructed via [`Error::custom`] or [`Error::full`]
    /// then this function will return [`Ok`], otherwise it will return [`Err`].
    pub fn into_inner(self) -> Result<Box<dyn std::error::Error + Send + Sync>, Self> {
        match self.repr {
            ErrorRepr::Custom { error, .. } => Ok(error),
            ErrorRepr::Full { error, .. } => Ok(error),
            _ => Err(self),
        }
    }

    /// Attempt to downgrade the inner error to `E` if any.
    ///
    /// Returns `Err(self)` if the downcast is not possible
    pub fn downcast<E: std::error::Error + Send + Sync + 'static>(self) -> Result<E, Self> {
        if self.downcast_ref::<E>().is_none() {
            return Err(self);
        }
        // Unwrapping is ok here since we already check above that the downcast will work
        Ok(*self
            .into_inner()?
            .downcast()
            .expect("failed to unwrap downcast"))
    }

    /// Returns a reference to the inner error wrapped by this error (if any).
    pub fn get_ref(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        match &self.repr {
            ErrorRepr::Custom { error, .. } => Some(error.as_ref()),
            ErrorRepr::Full { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }

    /// Downcast this error object by reference.
    pub fn downcast_ref<E: std::error::Error + Send + Sync + 'static>(&self) -> Option<&E> {
        self.get_ref()?.downcast_ref()
    }

    /// Returns a mutable reference to the inner error wrapped by this error (if any).
    pub fn get_mut(&mut self) -> Option<&mut (dyn std::error::Error + Send + Sync + 'static)> {
        match &mut self.repr {
            ErrorRepr::Custom { error, .. } => Some(error.as_mut()),
            ErrorRepr::Full { error, .. } => Some(error.as_mut()),
            _ => None,
        }
    }

    /// Returns a mutable reference to the inner error (if any) downcasted to the type provided
    pub fn downcast_mut<T: std::error::Error + Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.get_mut()?.downcast_mut()
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
