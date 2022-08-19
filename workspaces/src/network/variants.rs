use crate::network::Info;
use crate::result::{CallExecution, Result};
use crate::rpc::client::Client;
use crate::types::{AccountId, KeyType, SecretKey};
use crate::{Account, Contract, Worker};
use async_trait::async_trait;

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
    ) -> Result<CallExecution<Account>>;

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<CallExecution<Contract>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

impl<T> Worker<T>
where
    T: DevNetwork + TopLevelAccountCreator + 'static,
{
    pub async fn create_tla(&self, id: AccountId, sk: SecretKey) -> Result<CallExecution<Account>> {
        self.workspace
            .create_tla(self.clone().coerce(), id, sk)
            .await
    }

    pub async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<CallExecution<Contract>> {
        self.workspace
            .create_tla_and_deploy(self.clone().coerce(), id, sk, wasm)
            .await
    }

    pub async fn dev_generate(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        (id, sk)
    }

    pub async fn dev_create_account(&self) -> Result<Account> {
        let (id, sk) = self.dev_generate().await;
        let account = self.create_tla(id.clone(), sk).await?;
        account.into()
    }

    pub async fn dev_deploy(&self, wasm: &[u8]) -> Result<Contract> {
        let (id, sk) = self.dev_generate().await;
        let contract = self.create_tla_and_deploy(id.clone(), sk, wasm).await?;
        contract.into()
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
