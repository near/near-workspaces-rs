/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::TryFrom;
use std::fmt;
use std::path::Path;

/// Error module pertaining to all the error types exposed by workspaces. This
/// is not an exhaustive list of errors and more can be added in the future.
pub mod errors {
    pub use near_primitives::errors::{
        ActionError, ActionErrorKind, InvalidAccessKeyError, InvalidTxError, TxExecutionError,
    };
}

/// Status module exposing types where we can view the status of a transaction's
/// outcome.
pub mod status {
    pub use near_primitives::views::{ExecutionStatusView, FinalExecutionStatus};
}

pub use near_account_id::AccountId;
pub(crate) use near_crypto::{KeyType, Signer};
pub use near_primitives::views::ExecutionStatusView;
use near_primitives::{
    logging::pretty_hash,
    serialize::{from_base, to_base},
};
use serde::{Deserialize, Serialize};

pub type Gas = u64;

/// Balance is type for storing amounts of tokens. Usually represents the amount of tokens
/// in yoctoNear (1e-24).
pub type Balance = u128;

/// Height of a specific block
pub type BlockHeight = u64;

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

#[derive(Clone)]
pub struct InMemorySigner(pub(crate) near_crypto::InMemorySigner);

impl InMemorySigner {
    pub fn from_secret_key(account_id: AccountId, secret_key: SecretKey) -> Self {
        Self(near_crypto::InMemorySigner::from_secret_key(
            account_id,
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

// type taken from near_primitives::hash::CryptoHash.
/// CryptoHash is type for storing the hash of a specific block.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct CryptoHash(pub [u8; 32]);

impl std::str::FromStr for CryptoHash {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = from_base(s).map_err::<Self::Err, _>(|e| e.to_string().into())?;
        Self::try_from(bytes)
    }
}

impl TryFrom<&[u8]> for CryptoHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err("incorrect length for hash".into());
        }
        let mut buf = [0; 32];
        buf.copy_from_slice(bytes);
        Ok(CryptoHash(buf))
    }
}

impl TryFrom<Vec<u8>> for CryptoHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        <Self as TryFrom<&[u8]>>::try_from(v.as_ref())
    }
}

impl fmt::Debug for CryptoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", pretty_hash(&self.to_string()))
    }
}

impl fmt::Display for CryptoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&to_base(&self.0), f)
    }
}
