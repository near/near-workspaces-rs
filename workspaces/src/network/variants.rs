use std::sync::Arc;

use crate::network::Info;
use crate::result::CallExecution;
use crate::rpc::client::Client;
use crate::types::{AccountId, KeyType, SecretKey};
use crate::{Account, Contract};
use async_trait::async_trait;

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub trait NetworkClient {
    fn client(&self) -> &Client;
}

pub trait NetworkInfo {
    fn info(&self) -> &Info;
}

#[async_trait]
pub trait TopLevelAccountCreator: NetworkInfo {
    async fn create_tla(
        self: &Arc<Self>,
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account<Self>>>;

    async fn create_tla_and_deploy(
        self: &Arc<Self>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract<Self>>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[async_trait]
pub trait DevAccountDeployer: NetworkInfo {
    async fn dev_generate(self: &Arc<Self>) -> (AccountId, SecretKey);
    async fn dev_create_account(self: &Arc<Self>) -> anyhow::Result<Account<Self>>;
    async fn dev_deploy(self: &Arc<Self>, wasm: &[u8]) -> anyhow::Result<Contract<Self>>;
}

#[async_trait]
impl<T> DevAccountDeployer for T
where
    T: TopLevelAccountCreator + NetworkInfo + AllowDevAccountCreation + Send + Sync,
{
    async fn dev_generate(self: &Arc<Self>) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }

    async fn dev_create_account(self: &Arc<Self>) -> anyhow::Result<Account<Self>> {
        let (id, sk) = self.dev_generate().await;
        let account = self.create_tla(id.clone(), sk).await?;
        account.into()
    }

    async fn dev_deploy(self: &Arc<Self>, wasm: &[u8]) -> anyhow::Result<Contract<Self>> {
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
