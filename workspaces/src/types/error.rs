use std::fmt;

#[derive(Debug, thiserror::Error)]
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

    pub(crate) fn from_msg(kind: ParseErrorKind, msg: &'static str) -> Self {
        Self::from_repr(kind, anyhow::anyhow!(msg).into())
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    pub fn err_msg(&self) -> String {
        self.repr
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or("".to_string())
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
