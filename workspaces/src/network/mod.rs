mod account;
mod block;
mod info;
mod mainnet;
pub mod result;
mod sandbox;
mod server;
mod testnet;

pub mod transaction;

use async_trait::async_trait;

pub(crate) use crate::network::info::Info;
use crate::rpc::client::Client;
use crate::types::{AccountId, KeyType, SecretKey};

pub use crate::network::account::{Account, AccountDetails, Contract};
pub use crate::network::block::Block;
pub use crate::network::mainnet::Mainnet;
pub use crate::network::result::{CallExecution, CallExecutionDetails, ViewResultDetails};
pub use crate::network::sandbox::Sandbox;
pub use crate::network::testnet::Testnet;

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
        id: AccountId,
        sk: SecretKey,
    ) -> anyhow::Result<CallExecution<Account>>;

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[async_trait]
pub trait DevAccountDeployer {
    async fn dev_generate(&self) -> (AccountId, SecretKey);
    async fn dev_create_account(&self) -> anyhow::Result<Account>;
    async fn dev_deploy(&self, wasm: &[u8]) -> anyhow::Result<Contract>;
}

#[async_trait]
impl<T> DevAccountDeployer for T
where
    T: TopLevelAccountCreator + NetworkInfo + AllowDevAccountCreation + Send + Sync,
{
    async fn dev_generate(&self) -> (AccountId, SecretKey) {
        let id = crate::rpc::tool::random_account_id();
        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);

        let mut savepath = self.info().keystore_path.clone();

        // TODO: potentially make this into the async version:
        std::fs::create_dir_all(savepath.clone()).unwrap();

        savepath = savepath.join(id.to_string());
        savepath.set_extension("json");
        crate::rpc::tool::write_cred_to_file(&savepath, id.clone(), sk.clone());

        (id, sk)
    }

    async fn dev_create_account(&self) -> anyhow::Result<Account> {
        let (id, sk) = self.dev_generate().await;
        let account = self.create_tla(id.clone(), sk).await?;
        account.into()
    }

    async fn dev_deploy(&self, wasm: &[u8]) -> anyhow::Result<Contract> {
        let (id, sk) = self.dev_generate().await;
        let contract = self.create_tla_and_deploy(id.clone(), sk, wasm).await?;
        contract.into()
    }
}

pub trait Network: TopLevelAccountCreator + NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: TopLevelAccountCreator + NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: AllowDevAccountCreation + Network {}

// Implemented by default if we have `AllowDevAccountCreation`
impl<T> DevNetwork for T where T: AllowDevAccountCreation + Network {}
