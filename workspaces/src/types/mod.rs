pub(crate) mod account;
pub(crate) mod block;

/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::TryFrom;
use std::fmt;
use std::path::Path;

pub use near_account_id::AccountId;
pub(crate) use near_crypto::Signer;
use near_primitives::logging::pretty_hash;
use near_primitives::serialize::{from_base, to_base};
use serde::{Deserialize, Serialize};

use crate::error::{ParseError, ParseErrorKind};

/// Nonce is a unit used to determine the order of transactions in the pool.
pub type Nonce = u64;

/// Gas units used in the execution of transactions. For a more in depth description of
/// how and where it can be used, visit [Gas](https://docs.near.org/docs/concepts/gas).
pub type Gas = u64;

/// Balance is type for storing amounts of tokens. Usually represents the amount of tokens
/// in yoctoNear (1e-24).
pub type Balance = u128;

/// Height of a specific block
pub type BlockHeight = u64;

/// Key types supported for either a [`SecretKey`] or [`PublicKey`]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KeyType {
    ED25519,
    SECP256K1,
}

impl KeyType {
    const fn into_near_keytype(self) -> near_crypto::KeyType {
        match self {
            Self::ED25519 => near_crypto::KeyType::ED25519,
            Self::SECP256K1 => near_crypto::KeyType::SECP256K1,
        }
    }

    const fn from_near_keytype(key_type: near_crypto::KeyType) -> Self {
        match key_type {
            near_crypto::KeyType::ED25519 => Self::ED25519,
            near_crypto::KeyType::SECP256K1 => Self::SECP256K1,
        }
    }
}

impl From<PublicKey> for near_crypto::PublicKey {
    fn from(pk: PublicKey) -> Self {
        pk.0
    }
}

/// Public key of an account on chain. Usually created along with a [`SecretKey`]
/// to form a keypair associated to the account.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublicKey(pub(crate) near_crypto::PublicKey);

impl PublicKey {
    pub fn key_type(&self) -> KeyType {
        KeyType::from_near_keytype(self.0.key_type())
    }
}

/// Secret key of an account on chain. Usually created along with a [`PublicKey`]
/// to form a keypair associated to the account. To generate a new keypair, use
/// one of the creation methods found here, such as [`SecretKey::from_seed`]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretKey(near_crypto::SecretKey);

impl SecretKey {
    pub fn key_type(&self) -> KeyType {
        KeyType::from_near_keytype(self.0.key_type())
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }

    pub fn from_seed(key_type: KeyType, seed: &str) -> Self {
        let key_type = key_type.into_near_keytype();
        Self(near_crypto::SecretKey::from_seed(key_type, seed))
    }

    pub fn from_random(key_type: KeyType) -> Self {
        let key_type = key_type.into_near_keytype();
        Self(near_crypto::SecretKey::from_random(key_type))
    }
}

impl std::str::FromStr for SecretKey {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let sk = near_crypto::SecretKey::from_str(value)?;

        Ok(Self(sk))
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
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CryptoHash(pub [u8; 32]);

impl std::str::FromStr for CryptoHash {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = from_base(s).map_err(|e| Self::Err::from_repr(ParseErrorKind::Unknown, e))?;
        Self::try_from(bytes)
    }
}

impl TryFrom<&[u8]> for CryptoHash {
    type Error = ParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(ParseErrorKind::IncorrectHashLength {
                expected_length: 32,
                received_length: bytes.len(),
            }
            .into());
        }
        let mut buf = [0; 32];
        buf.copy_from_slice(bytes);
        Ok(CryptoHash(buf))
    }
}

impl TryFrom<Vec<u8>> for CryptoHash {
    type Error = ParseError;

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

/// Access key provides limited access to an account. Each access key belongs to some account and
/// is identified by a unique (within the account) public key. One account may have large number of
/// access keys. Access keys allow to act on behalf of the account by restricting transactions
/// that can be issued.
#[derive(Clone, Debug)]
pub struct AccessKey {
    /// The nonce for this access key.
    /// NOTE: In some cases the access key needs to be recreated. If the new access key reuses the
    /// same public key, the nonce of the new access key should be equal to the nonce of the old
    /// access key. It's required to avoid replaying old transactions again.
    nonce: Nonce,

    /// Defines permissions for this access key.
    permission: AccessKeyPermission,
}

impl AccessKey {
    pub fn full_access() -> Self {
        Self {
            nonce: 0,
            permission: AccessKeyPermission::FullAccess,
        }
    }

    pub fn function_call_access(
        receiver_id: &AccountId,
        method_names: &[&str],
        allowance: Option<Balance>,
    ) -> Self {
        Self {
            nonce: 0,
            permission: AccessKeyPermission::FunctionCall(FunctionCallPermission {
                receiver_id: receiver_id.clone().into(),
                method_names: method_names.iter().map(|s| s.to_string()).collect(),
                allowance,
            }),
        }
    }
}

/// Defines permissions for AccessKey
#[derive(Clone, Debug)]
enum AccessKeyPermission {
    FunctionCall(FunctionCallPermission),

    /// Grants full access to the account.
    /// NOTE: It's used to replace account-level public keys.
    FullAccess,
}

/// Grants limited permission to make transactions with FunctionCallActions
/// The permission can limit the allowed balance to be spent on the prepaid gas.
/// It also restrict the account ID of the receiver for this function call.
/// It also can restrict the method name for the allowed function calls.
#[derive(Clone, Debug)]
struct FunctionCallPermission {
    /// Allowance is a balance limit to use by this access key to pay for function call gas and
    /// transaction fees. When this access key is used, both account balance and the allowance is
    /// decreased by the same value.
    /// `None` means unlimited allowance.
    /// NOTE: To change or increase the allowance, the old access key needs to be deleted and a new
    /// access key should be created.
    allowance: Option<Balance>,

    // This isn't an AccountId because already existing records in testnet genesis have invalid
    // values for this field (see: https://github.com/near/nearcore/pull/4621#issuecomment-892099860)
    // we accomodate those by using a string, allowing us to read and parse genesis.
    /// The access key only allows transactions with the given receiver's account id.
    receiver_id: String,

    /// A list of method names that can be used. The access key only allows transactions with the
    /// function call of one of the given method names.
    /// Empty list means any method name can be used.
    method_names: Vec<String>,
}

impl From<AccessKey> for near_primitives::account::AccessKey {
    fn from(access_key: AccessKey) -> Self {
        Self {
            nonce: access_key.nonce,
            permission: match access_key.permission {
                AccessKeyPermission::FunctionCall(function_call_permission) => {
                    near_primitives::account::AccessKeyPermission::FunctionCall(
                        near_primitives::account::FunctionCallPermission {
                            allowance: function_call_permission.allowance,
                            receiver_id: function_call_permission.receiver_id,
                            method_names: function_call_permission.method_names,
                        },
                    )
                }
                AccessKeyPermission::FullAccess => {
                    near_primitives::account::AccessKeyPermission::FullAccess
                }
            },
        }
    }
}
