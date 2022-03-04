/// Types copied over from near_primitives since those APIs are not yet stable.
/// and internal libraries like near-jsonrpc-client requires specific versions
/// of these types which shouldn't be exposed either.
use std::convert::TryFrom;
use std::fmt;
use std::path::Path;

pub use near_account_id::AccountId;
pub(crate) use near_crypto::{KeyType, Signer};
use near_primitives::logging::pretty_hash;
use near_primitives::serialize::{from_base, to_base};
use serde::{Deserialize, Serialize};

/// Nonce for transactions.
pub type Nonce = u64;

/// Gas is a type for storing amount of gas.
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
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
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
