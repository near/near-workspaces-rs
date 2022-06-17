use std::fmt;

/// All the possible error kinds that can occur when parsing within workspaces.
#[derive(Copy, Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParseErrorKind {
    #[error("incorrect hash length (expected {expected_length}, but {received_length} was given)")]
    IncorrectHashLength {
        expected_length: usize,
        received_length: usize,
    },
    #[error("unknown key type")]
    UnknownKeyType,
    #[error("invalid key length")]
    InvalidKeyLength,
    #[error("invalid key data")]
    InvalidKeyData,
    #[error("unknown parse error occurred")]
    Unknown,
}

/// Any errors related to parsing, potentially coming from dependencies
/// and then forwarded to this error type.
pub struct ParseError {
    kind: ParseErrorKind,
    repr: Option<Box<dyn std::error::Error>>,
}

impl ParseError {
    pub(crate) fn from_kind(kind: ParseErrorKind) -> Self {
        Self { kind, repr: None }
    }

    pub(crate) fn from_repr(kind: ParseErrorKind, repr: Box<dyn std::error::Error>) -> Self {
        Self {
            kind,
            repr: Some(repr),
        }
    }

    /// Get the kind of error that occurred from parsing.
    pub fn kind(&self) -> ParseErrorKind {
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

impl std::error::Error for ParseError {}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind(), self.err_msg())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind(), self.err_msg())
    }
}

impl From<near_crypto::ParseKeyError> for ParseError {
    fn from(err: near_crypto::ParseKeyError) -> Self {
        let kind = match err {
            near_crypto::ParseKeyError::UnknownKeyType { .. } => ParseErrorKind::UnknownKeyType,
            near_crypto::ParseKeyError::InvalidLength { .. } => ParseErrorKind::InvalidKeyLength,
            near_crypto::ParseKeyError::InvalidData { .. } => ParseErrorKind::InvalidKeyData,
        };

        Self::from_repr(kind, err.into())
    }
}

impl From<ParseErrorKind> for ParseError {
    fn from(kind: ParseErrorKind) -> Self {
        Self::from_kind(kind)
    }
}
