/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use near_crypto::vrf::{Proof, Value};
use near_crypto::Signature;
use near_primitives::types::AccountId as NearAccountId;

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
pub struct AccountId(Box<str>);

impl AccountId {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn validate(account_id: &str) -> Result<(), ParseAccountError> {
        NearAccountId::validate(account_id)
            .map_err(map_account_error)
    }
}

impl TryFrom<String> for AccountId {
    type Error = ParseAccountError;

    fn try_from(account_id: String) -> Result<Self, Self::Error> {
        Self::validate(&account_id)?;
        Ok(Self(account_id.into()))
    }
}

impl From<AccountId> for String {
    fn from(account_id: AccountId) -> Self {
        account_id.0.into_string()
    }
}

impl TryFrom<AccountId> for near_primitives::types::AccountId {
    type Error = ParseAccountError;

    fn try_from(id: AccountId) -> Result<Self, Self::Error> {
        id.0.into_string()
            .try_into()
            .map_err(map_account_error)
    }
}

impl FromStr for AccountId {
    type Err = ParseAccountError;

    fn from_str(account_id: &str) -> Result<Self, Self::Err> {
        Self::validate(account_id)?;
        Ok(Self(account_id.into()))
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

/// Signer that keeps secret key in memory.
#[derive(Clone, Serialize, Deserialize)]
pub struct InMemorySigner(pub(crate) near_crypto::InMemorySigner);

impl InMemorySigner {
    pub(crate) fn from_seed(account_id: AccountId, key_type: KeyType, seed: &str) -> Self {
        Self(near_crypto::InMemorySigner::from_seed(account_id.try_into().unwrap(), key_type, seed))
    }

    pub fn from_secret_key(account_id: AccountId, secret_key: SecretKey) -> Self {
        Self(near_crypto::InMemorySigner::from_secret_key(
            account_id.try_into().unwrap(),
            secret_key.0,
        ))
    }

    pub fn from_file(path: &Path) -> Self {
        Self(near_crypto::InMemorySigner::from_file(path))
    }
}

impl Signer for InMemorySigner {
    fn public_key(&self) -> near_crypto::PublicKey {
        self.0.public_key()
    }

    fn sign(&self, data: &[u8]) -> Signature {
        self.0.sign(data)
    }

    fn compute_vrf_with_proof(&self, data: &[u8]) -> (Value, Proof) {
        self.0.compute_vrf_with_proof(data)
    }

    fn write_to_file(&self, path: &Path) {
        self.0.write_to_file(path);
    }
}
