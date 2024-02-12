use async_trait::async_trait;
use near_token::NearToken;

use crate::{
    network::Info,
    result::{Execution, Result},
    rpc::client::Client,
    types::{AccountId, KeyType, SecretKey},
    Account, Contract, Worker,
};

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub trait NetworkClient {
    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    fn info(&self) -> &Info;
}

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

    async fn create_tla_with_deposit(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        deposit: NearToken,
    ) -> Result<Execution<Account>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

impl<T> Worker<T>
where
    T: DevNetwork + TopLevelAccountCreator + 'static,
{
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

    pub async fn create_tla_with_deposit(
        &self,
        id: AccountId,
        sk: SecretKey,
        deposit: NearToken,
    ) -> Result<Execution<Account>> {
        let res = self
            .workspace
            .create_tla_with_deposit(self.clone().coerce(), id, sk, deposit)
            .await?;

        for callback in self.tx_callbacks.iter() {
            callback(res.details.total_gas_burnt)?;
        }

        Ok(res)
    }

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
        let (id, sk) = self.dev_generate().await;
        let account = self.create_tla(id.clone(), sk).await?;
        Ok(account.into_result()?)
    }

    pub async fn dev_create_account_with_deposit(&self, deposit: NearToken) -> Result<Account> {
        let (id, sk) = self.dev_generate().await;
        let account = self
            .create_tla_with_deposit(id.clone(), sk, deposit)
            .await?;
        Ok(account.into_result()?)
    }

    pub async fn dev_deploy(&self, wasm: &[u8]) -> Result<Contract> {
        let (id, sk) = self.dev_generate().await;
        let contract = self.create_tla_and_deploy(id.clone(), sk, wasm).await?;
        Ok(contract.into_result()?)
    }
}

/// Network trait specifies the functionality of a network type such as mainnet, testnet or any
/// other networks that are not specified in this library.
pub trait Network: NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: TopLevelAccountCreator + AllowDevAccountCreation + Network + 'static {}

impl<T> DevNetwork for T where
    T: TopLevelAccountCreator + AllowDevAccountCreation + Network + 'static
{
}
