use crate::network::Info;
use crate::result::CallExecution;
use crate::rpc::client::Client;
use crate::types::{AccountId, KeyType, SecretKey};
use crate::{Account, Contract, Worker};
use async_trait::async_trait;

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub trait NetworkClient {
    type Network;

    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    fn info(&self) -> &Info;
}

#[async_trait]
pub trait TopLevelAccountCreator: NetworkClient {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account<Self::Network>>>;

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract<Self::Network>>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[async_trait]
pub trait DevAccountDeployer: NetworkClient {
    async fn dev_generate(&self) -> (AccountId, SecretKey);
    async fn dev_create_account(&self) -> anyhow::Result<Account<Self::Network>>;
    async fn dev_deploy(&self, wasm: &[u8]) -> anyhow::Result<Contract<Self::Network>>;
}

#[async_trait]
impl<T> DevAccountDeployer for Worker<T>
where
    Worker<T>: TopLevelAccountCreator + NetworkInfo,
    T: AllowDevAccountCreation + Send + Sync,
{
    async fn dev_generate(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }

    async fn dev_create_account(&self) -> anyhow::Result<Account<Self::Network>> {
        let (id, sk) = self.dev_generate().await;
        let account = self.create_tla(id.clone(), sk).await?;
        account.into()
    }

    async fn dev_deploy(&self, wasm: &[u8]) -> anyhow::Result<Contract<Self::Network>> {
        let (id, sk) = self.dev_generate().await;
        let contract = self.create_tla_and_deploy(id.clone(), sk, wasm).await?;
        contract.into()
    }
}

pub trait Network: NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: TopLevelAccountCreator + AllowDevAccountCreation + Network {}

// Implemented by default if we have `AllowDevAccountCreation`
impl<T> DevNetwork for T where T: TopLevelAccountCreator + AllowDevAccountCreation + Network {}
