/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
use std::str::FromStr;

use near_crypto::key_conversion::convert_secret_key;
use near_crypto::vrf::{Proof, Value};
use near_crypto::Signature;
use near_primitives::account::id::{MAX_ACCOUNT_ID_LEN, MIN_ACCOUNT_ID_LEN};

pub(crate) use near_crypto::{KeyType, PublicKey, SecretKey, Signer};
use serde::{Deserialize, Serialize};

#[derive(Eq, Ord, Hash, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AccountId(Box<str>);

impl AccountId {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn validate(account_id: &str) -> Result<(), ParseAccountError> {
        if account_id.len() < MIN_ACCOUNT_ID_LEN {
            return Err(ParseAccountError(
                ParseErrorKind::TooShort,
                account_id.to_string(),
            ));
        }

        if account_id.len() > MAX_ACCOUNT_ID_LEN {
            return Err(ParseAccountError(
                ParseErrorKind::TooLong,
                account_id.to_string(),
            ));
        }
        // Adapted from https://github.com/near/near-sdk-rs/blob/fd7d4f82d0dfd15f824a1cf110e552e940ea9073/near-sdk/src/environment/env.rs#L819

        // NOTE: We don't want to use Regex here, because it requires extra time to compile it.
        // The valid account ID regex is /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/
        // Instead the implementation is based on the previous character checks.

        // We can safely assume that last char was a separator.
        let mut last_char_is_separator = true;

        for c in account_id.bytes() {
            let current_char_is_separator = match c {
                b'a'..=b'z' | b'0'..=b'9' => false,
                b'-' | b'_' | b'.' => true,
                _ => {
                    return Err(ParseAccountError(
                        ParseErrorKind::Invalid,
                        account_id.to_string(),
                    ))
                }
            };
            if current_char_is_separator && last_char_is_separator {
                return Err(ParseAccountError(
                    ParseErrorKind::Invalid,
                    account_id.to_string(),
                ));
            }
            last_char_is_separator = current_char_is_separator;
        }

        (!last_char_is_separator)
            .then(|| ())
            .ok_or(ParseAccountError(
                ParseErrorKind::Invalid,
                account_id.to_string(),
            ))
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

impl Into<near_primitives::types::AccountId> for AccountId {
    fn into(self) -> near_primitives::types::AccountId {
        self.0
            .into_string()
            .try_into()
            .expect("Should not fail since this is already validated before")
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

/// A list of errors that occur when parsing an invalid Account ID.
#[derive(Eq, Hash, Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    TooLong,
    TooShort,
    Invalid,
}

/// Signer that keeps secret key in memory.
#[derive(Clone)]
pub struct InMemorySigner {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl InMemorySigner {
    pub(crate) fn from_seed(account_id: AccountId, key_type: KeyType, seed: &str) -> Self {
        let secret_key = SecretKey::from_seed(key_type, seed);
        Self {
            account_id,
            public_key: secret_key.public_key(),
            secret_key,
        }
    }

    pub fn from_secret_key(account_id: AccountId, secret_key: &str) -> Self {
        let secret_key = SecretKey::from_str(secret_key).expect("Invalid secret key");

        Self {
            account_id,
            public_key: secret_key.public_key(),
            secret_key,
        }
    }

    pub fn from_file(path: &Path) -> Self {
        KeyFile::from_file(path).into()
    }
}

impl Signer for InMemorySigner {
    fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    fn sign(&self, data: &[u8]) -> Signature {
        self.secret_key.sign(data)
    }

    fn compute_vrf_with_proof(&self, data: &[u8]) -> (Value, Proof) {
        let secret_key = convert_secret_key(self.secret_key.unwrap_as_ed25519());
        secret_key.compute_vrf_with_proof(&data)
    }

    fn write_to_file(&self, path: &Path) {
        KeyFile::from(self).write_to_file(path);
    }
}

impl From<&InMemorySigner> for KeyFile {
    fn from(signer: &InMemorySigner) -> KeyFile {
        KeyFile {
            account_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            secret_key: signer.secret_key.clone(),
        }
    }
}

impl From<KeyFile> for InMemorySigner {
    fn from(key_file: KeyFile) -> Self {
        Self {
            account_id: key_file.account_id,
            public_key: key_file.public_key,
            secret_key: key_file.secret_key,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KeyFile {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl KeyFile {
    pub fn write_to_file(&self, path: &Path) {
        let mut file = File::create(path).expect("Failed to create / write a key file.");
        let mut perm = file
            .metadata()
            .expect("Failed to retrieve key file metadata.")
            .permissions();
        perm.set_mode(u32::from(libc::S_IWUSR | libc::S_IRUSR));
        file.set_permissions(perm)
            .expect("Failed to set permissions for a key file.");
        let str = serde_json::to_string_pretty(self).expect("Error serializing the key file.");
        if let Err(err) = file.write_all(str.as_bytes()) {
            panic!("Failed to write a key file {}", err);
        }
    }

    pub fn from_file(path: &Path) -> Self {
        let mut file = File::open(path).expect("Could not open key file.");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Could not read from key file.");
        serde_json::from_str(&content).expect("Failed to deserialize KeyFile")
    }
}
