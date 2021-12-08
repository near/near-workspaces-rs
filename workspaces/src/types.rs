/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

use near_crypto::key_conversion::convert_secret_key;
use near_crypto::vrf::{Proof, Value};
use near_crypto::Signature;
use near_primitives::types::AccountId as NearAccountId;

pub(crate) use near_crypto::{KeyType, PublicKey, SecretKey, Signer};
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
pub(crate) struct KeyFile {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl KeyFile {
    pub fn write_to_file(&self, path: &Path) {
        let mut file = File::create(path).expect("Failed to create / write a key file.");

        #[cfg(unix)]
        {
            use std::os::unix::prelude::PermissionsExt;
            let mut perm = file
                .metadata()
                .expect("Failed to retrieve key file metadata.")
                .permissions();

            #[cfg(target_os = "macos")]
            perm.set_mode(u32::from(libc::S_IWUSR | libc::S_IRUSR));
            #[cfg(not(target_os = "macos"))]
            perm.set_mode(libc::S_IWUSR | libc::S_IRUSR);

            file.set_permissions(perm)
                .expect("Failed to set permissions for a key file.");
        }

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
