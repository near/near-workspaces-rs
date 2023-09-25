use near_account_id::AccountId;

use crate::{
    error::{Error, ErrorKind},
    network::NetworkClient,
    rpc::query::{Query, ViewAccessKeyList},
    types::AccessKeyPermission,
    Worker,
};

use super::{PublicKey, SecretKey};

#[derive(Debug, serde::Serialize)]
struct KeyPairProperties {
    public_key: near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
}

#[derive(Debug, serde::Deserialize)]
pub struct AccountKeyPair {
    pub public_key: near_crypto::PublicKey,
    pub private_key: near_crypto::SecretKey,
}

pub type KeyLoader = AccountKeyPair;

impl KeyLoader {
    pub fn new(secret_key: SecretKey, public_key: PublicKey) -> Self {
        Self {
            public_key: public_key.0,
            private_key: secret_key.0,
        }
    }

    /// This loads the account information from the keychain. This is interoperable with credentials saved using
    /// `near-cli-rs` using the "save-to-keychain" option.
    ///
    /// Note: Other tools may use different paths/formats.
    pub async fn from_keychain(
        worker: &Worker<impl NetworkClient>,
        network: &str,
        account_id: AccountId,
    ) -> Result<AccountKeyPair, Error> {
        let service_name: std::borrow::Cow<'_, str> =
            std::borrow::Cow::Owned(format!("near-{}-{}", network, account_id.as_str()));

        let access_key_list = Query::new(
            worker.client(),
            ViewAccessKeyList {
                account_id: account_id.clone(),
            },
        )
        .await?;

        let credentials = access_key_list
            .into_iter()
            .filter(|key| matches!(key.access_key.permission, AccessKeyPermission::FullAccess,))
            .map(|key| key.public_key)
            .find_map(|public_key| {
                let keyring =
                    keyring::Entry::new(&service_name, &format!("{}:{}", account_id, public_key))
                        .ok()?;
                keyring.get_password().ok()
            });

        match credentials {
            Some(cred) => serde_json::from_str::<AccountKeyPair>(&cred)
                .map_err(|e| Error::custom(ErrorKind::DataConversion, e)),

            None => Err(Error::custom(
                ErrorKind::Other,
                "No access keys found in keychain",
            )),
        }
    }

    /// This saves the account information to the keychain. This is interoperable with credentials saved using
    /// `near-cli-rs` using the "save-to-keychain" option.
    pub async fn to_keychain(&self, network: &str, account_id: &str) -> Result<(), Error> {
        let service_name = std::borrow::Cow::Owned(format!("near-{}-{}", network, account_id));

        keyring::Entry::new(
            &service_name,
            &format!("{}:{}", account_id, self.public_key),
        )
        .map_err(|e| {
            Error::custom(
                ErrorKind::Io,
                format!("Failed to create keyring entry: {}", e),
            )
        })?
        .set_password(
            &serde_json::to_string(&KeyPairProperties {
                public_key: self.public_key.clone(),
                private_key: self.private_key.clone(),
            })
            .expect("KeyPairProperties is serializable"),
        )
        .map_err(|e| {
            Error::custom(
                ErrorKind::Io,
                format!("Failed to set keyring credentials: {}", e),
            )
        })?;

        Ok(())
    }
}
