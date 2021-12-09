/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

pub(crate) use near_crypto::{KeyType, Signer};
use serde::{Deserialize, Serialize};

fn map_account_error(err: near_primitives::account::id::ParseAccountError) -> ParseAccountError {
    let kind = match err.kind() {
        near_primitives::account::id::ParseErrorKind::TooLong => ParseErrorKind::TooLong,
        near_primitives::account::id::ParseErrorKind::TooShort => ParseErrorKind::TooShort,
        near_primitives::account::id::ParseErrorKind::Invalid => ParseErrorKind::Invalid,
    };
    ParseAccountError(kind, err.get_account_id())
}

#[derive(Eq, Ord, Hash, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AccountId(near_primitives::types::AccountId);

impl AccountId {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl TryFrom<String> for AccountId {
    type Error = ParseAccountError;

    fn try_from(account_id: String) -> Result<Self, Self::Error> {
        Ok(Self(account_id.try_into().map_err(map_account_error)?))
    }
}

impl From<AccountId> for String {
    fn from(account_id: AccountId) -> Self {
        account_id.0.into()
    }
}

impl From<AccountId> for near_primitives::types::AccountId {
    fn from(id: AccountId) -> Self {
        id.0
    }
}

impl FromStr for AccountId {
    type Err = ParseAccountError;

    fn from_str(account_id: &str) -> Result<Self, Self::Err> {
        let id =
            near_primitives::types::AccountId::from_str(account_id).map_err(map_account_error)?;
        Ok(Self(id))
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An error occurred when parsing an invalid Account ID with [`AccountId::validate`](crate::AccountId::validate).
#[derive(Eq, Hash, Clone, Debug, PartialEq)]
pub struct ParseAccountError(pub(crate) ParseErrorKind, pub(crate) String);

impl std::error::Error for ParseAccountError {}
impl fmt::Display for ParseAccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]: {}", self.1, self.0)
    }
}

/// A list of errors that occur when parsing an invalid Account ID.
#[derive(Eq, Hash, Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    TooLong,
    TooShort,
    Invalid,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorKind::TooLong => write!(f, "the value is too long for account ID"),
            ParseErrorKind::TooShort => write!(f, "the value is too short for account ID"),
            ParseErrorKind::Invalid => write!(f, "the value has invalid characters for account ID"),
        }
    }
}

impl From<PublicKey> for near_crypto::PublicKey {
    fn from(pk: PublicKey) -> Self {
        pk.0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PublicKey(pub(crate) near_crypto::PublicKey);

#[derive(Clone, Serialize, Deserialize)]
pub struct SecretKey(near_crypto::SecretKey);

impl SecretKey {
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }

    pub fn from_seed(key_type: KeyType, seed: &str) -> Self {
        Self(near_crypto::SecretKey::from_seed(key_type, seed))
    }
}

pub struct InMemorySigner(near_crypto::InMemorySigner);

impl InMemorySigner {
    pub fn from_secret_key(account_id: AccountId, secret_key: SecretKey) -> Self {
        Self(near_crypto::InMemorySigner::from_secret_key(
            account_id.0,
            secret_key.0,
        ))
    }

    pub fn from_file(path: &Path) -> Self {
        Self(near_crypto::InMemorySigner::from_file(path))
    }

    pub(crate) fn inner(&self) -> &near_crypto::InMemorySigner {
        &self.0
    }
}
