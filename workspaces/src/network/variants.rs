use crate::network::Info;
use crate::result::{Execution, Result};
use crate::rpc::client::Client;
use crate::types::{AccountId, KeyType, SecretKey, DEFAULT_DEPOSIT};
use crate::{Account, Contract, InMemorySigner, Worker};
use async_trait::async_trait;

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub trait NetworkClient {
    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    fn info(&self) -> &Info;

    /// Using the keystore path, if the credentials exists, we can load the signer from the
    /// file.
    fn root_signer(&self) -> Result<InMemorySigner> {
        InMemorySigner::from_file(&self.info().keystore_path)
    }
}

#[deprecated = "top level account creation is not possible in Protocol >=64"]
#[async_trait]
pub trait TopLevelAccountCreator {
    async fn create_tla(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>>;

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[allow(deprecated)]
impl<T> Worker<T>
where
    T: DevNetwork + TopLevelAccountCreator + 'static,
{
    #[deprecated = "use dev_create_account() instead, tla account creation is not possible in Protocol >=64"]
    pub async fn create_tla(&self, id: AccountId, sk: SecretKey) -> Result<Execution<Account>> {
        let res = self
            .workspace
            .create_tla(self.clone().coerce(), id, sk)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    #[deprecated = "use create_account_and_deploy() instead, tla account creation is not possible in Protocol >=64"]
    pub async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        let res = self
            .workspace
            .create_tla_and_deploy(self.clone().coerce(), id, sk, wasm)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

    pub async fn dev_generate(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }

    pub async fn dev_create_account(&self) -> Result<Account> {
        let id = crate::rpc::tool::random_account_id();
        let root_account = self.workspace.root_signer()?;

        let account = Account::new(root_account, self.clone().coerce())
            .create_subaccount(&id)
            .initial_balance(DEFAULT_DEPOSIT)
            .transact()
            .await?;

        Ok(account.into_result()?)
    }

    pub async fn dev_deploy(&self, wasm: &[u8]) -> Result<Contract> {
        Ok(self
            .dev_create_account()
            .await?
            .deploy(wasm)
            .await?
            .into_result()?)
    }
}

/// Network trait specifies the functionality of a network type such as mainnet, testnet or any
/// other networks that are not specified in this library.
pub trait Network: NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
#[allow(deprecated)]
pub trait DevNetwork: TopLevelAccountCreator + AllowDevAccountCreation + Network + 'static {}

#[allow(deprecated)]
impl<T> DevNetwork for T where
    T: TopLevelAccountCreator + AllowDevAccountCreation + Network + 'static
{
}
