/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::path::Path;

pub use near_account_id::AccountId;
pub(crate) use near_crypto::{KeyType, Signer};
use serde::{Deserialize, Serialize};

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
