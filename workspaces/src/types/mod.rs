pub(crate) mod account;
pub(crate) mod block;
pub(crate) mod chunk;

/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::TryFrom;
use std::fmt;
use std::path::Path;

pub use near_account_id::AccountId;
use near_primitives::logging::pretty_hash;
use near_primitives::serialize::to_base58;
use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind};
use crate::result::Result;

pub use self::chunk::{Chunk, ChunkHeader};

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

/// Shard index, from 0 to NUM_SHARDS - 1.
pub type ShardId = u64;

fn from_base58(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    bs58::decode(s).into_vec().map_err(|err| err.into())
}

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
pub struct SecretKey(pub(crate) near_crypto::SecretKey);

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
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let sk = near_crypto::SecretKey::from_str(value)
            .map_err(|e| ErrorKind::DataConversion.custom(e))?;

        Ok(Self(sk))
    }
}

#[derive(Clone)]
pub struct InMemorySigner {
    pub(crate) account_id: AccountId,
    pub(crate) secret_key: SecretKey,
}

impl InMemorySigner {
    pub fn from_secret_key(account_id: AccountId, secret_key: SecretKey) -> Self {
        Self {
            account_id,
            secret_key,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let signer = near_crypto::InMemorySigner::from_file(path)
            .map_err(|err| ErrorKind::Io.custom(err))?;
        Ok(Self::from_secret_key(
            signer.account_id,
            SecretKey(signer.secret_key),
        ))
    }

    pub(crate) fn inner(&self) -> near_crypto::InMemorySigner {
        near_crypto::InMemorySigner::from_secret_key(
            self.account_id.clone(),
            self.secret_key.0.clone(),
        )
    }
}

// type taken from near_primitives::hash::CryptoHash.
/// CryptoHash is type for storing the hash of a specific block.
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CryptoHash(pub [u8; 32]);

impl std::str::FromStr for CryptoHash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = from_base58(s).map_err(|e| ErrorKind::DataConversion.custom(e))?;
        Self::try_from(bytes)
    }
}

impl TryFrom<&[u8]> for CryptoHash {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(Error::message(
                ErrorKind::DataConversion,
                format!(
                    "incorrect hash length (expected 32, but {} was given)",
                    bytes.len()
                ),
            ));
        }
        let mut buf = [0; 32];
        buf.copy_from_slice(bytes);
        Ok(CryptoHash(buf))
    }
}

impl TryFrom<Vec<u8>> for CryptoHash {
    type Error = Error;

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
        fmt::Display::fmt(&to_base58(self.0), f)
    }
}

impl From<near_primitives::hash::CryptoHash> for CryptoHash {
    fn from(hash: near_primitives::hash::CryptoHash) -> Self {
        Self(hash.0)
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
    pub nonce: Nonce,

    /// Defines permissions for this access key.
    pub permission: AccessKeyPermission,
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

/// Similar to an [`AccessKey`], but also has the [`PublicKey`] associated with it.
#[derive(Clone, Debug)]
pub struct AccessKeyInfo {
    pub public_key: PublicKey,
    pub access_key: AccessKey,
}

impl From<near_primitives::views::AccessKeyInfoView> for AccessKeyInfo {
    fn from(view: near_primitives::views::AccessKeyInfoView) -> Self {
        Self {
            public_key: PublicKey(view.public_key),
            access_key: view.access_key.into(),
        }
    }
}

/// Defines permissions for AccessKey
#[derive(Clone, Debug)]
pub enum AccessKeyPermission {
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
pub struct FunctionCallPermission {
    /// Allowance is a balance limit to use by this access key to pay for function call gas and
    /// transaction fees. When this access key is used, both account balance and the allowance is
    /// decreased by the same value.
    /// `None` means unlimited allowance.
    /// NOTE: To change or increase the allowance, the old access key needs to be deleted and a new
    /// access key should be created.
    pub allowance: Option<Balance>,

    // This isn't an AccountId because already existing records in testnet genesis have invalid
    // values for this field (see: https://github.com/near/nearcore/pull/4621#issuecomment-892099860)
    // we accomodate those by using a string, allowing us to read and parse genesis.
    /// The access key only allows transactions with the given receiver's account id.
    pub receiver_id: String,

    /// A list of method names that can be used. The access key only allows transactions with the
    /// function call of one of the given method names.
    /// Empty list means any method name can be used.
    pub method_names: Vec<String>,
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

impl From<near_primitives::views::AccessKeyView> for AccessKey {
    fn from(access_key: near_primitives::views::AccessKeyView) -> Self {
        Self {
            nonce: access_key.nonce,
            permission: match access_key.permission {
                near_primitives::views::AccessKeyPermissionView::FunctionCall {
                    allowance,
                    receiver_id,
                    method_names,
                } => AccessKeyPermission::FunctionCall(FunctionCallPermission {
                    allowance,
                    receiver_id,
                    method_names,
                }),
                near_primitives::views::AccessKeyPermissionView::FullAccess => {
                    AccessKeyPermission::FullAccess
                }
            },
        }
    }
}

/// Finality of a transaction or block in which transaction is included in. For more info
/// go to the [NEAR finality](https://docs.near.org/docs/concepts/transaction#finality) docs.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Finality {
    /// Optimistic finality. The latest block recorded on the node that responded to our query
    /// (<1 second delay after the transaction is submitted).
    Optimistic,
    /// Near-final finality. Similiarly to `Final` finality, but delay should be roughly 1 second.
    DoomSlug,
    /// Final finality. The block that has been validated on at least 66% of the nodes in the
    /// network. (At max, should be 2 second delay after the transaction is submitted.)
    Final,
}

impl From<Finality> for near_primitives::types::BlockReference {
    fn from(value: Finality) -> Self {
        let value = match value {
            Finality::Optimistic => near_primitives::types::Finality::None,
            Finality::DoomSlug => near_primitives::types::Finality::DoomSlug,
            Finality::Final => near_primitives::types::Finality::Final,
        };
        value.into()
    }
}
