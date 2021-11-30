mod account;
mod info;
mod result;
mod sandbox;
mod server;
mod testnet;

use async_trait::async_trait;

use near_crypto::{InMemorySigner, KeyType, Signer};
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;
use near_primitives::types::AccountId;

pub(crate) use crate::network::info::Info;
use crate::rpc::client::Client;

pub use crate::network::account::{Account, Contract};
pub use crate::network::result::{CallExecution, CallExecutionDetails};
pub use crate::network::sandbox::Sandbox;
pub use crate::network::testnet::Testnet;

const DEV_ACCOUNT_SEED: &str = "testificate";

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
        signer: InMemorySigner,
    ) -> anyhow::Result<CallExecution<Account>>;

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        signer: InMemorySigner,
        wasm: Vec<u8>,
    ) -> anyhow::Result<CallExecution<Contract>>;
}

// NOTE: Not all networks/runtimes will have the ability to be able to do dev_deploy.
// This trait acts as segmented boundary for only specific networks such as sandbox and testnet.
pub trait AllowDevAccountCreation {}

#[async_trait]
pub trait DevAccountDeployer {
    fn dev_generate(&self) -> (AccountId, InMemorySigner);
    async fn dev_create(&self) -> anyhow::Result<Account>;
    async fn dev_deploy(&self, wasm: Vec<u8>) -> anyhow::Result<Contract>;
}

#[async_trait]
impl<T> DevAccountDeployer for T
where
    T: TopLevelAccountCreator + NetworkInfo + AllowDevAccountCreation + Send + Sync,
{
    fn dev_generate(&self) -> (AccountId, InMemorySigner) {
        let account_id = crate::rpc::tool::random_account_id();
        let signer =
            InMemorySigner::from_seed(account_id.clone(), KeyType::ED25519, DEV_ACCOUNT_SEED);

        let mut savepath = self.info().keystore_path.clone();

        // TODO: potentially make this into the async version:
        std::fs::create_dir_all(savepath.clone()).unwrap();

        savepath = savepath.join(account_id.to_string());
        savepath.set_extension("json");
        signer.write_to_file(&savepath);

        (account_id, signer)
    }

    async fn dev_create(&self) -> anyhow::Result<Account> {
        let (account_id, signer) = self.dev_generate();
        let account = self.create_tla(account_id.clone(), signer).await?;
        account.into()
    }

    async fn dev_deploy(&self, wasm: Vec<u8>) -> anyhow::Result<Contract> {
        let (account_id, signer) = self.dev_generate();
        let contract = self
            .create_tla_and_deploy(account_id.clone(), signer, wasm)
            .await?;
        contract.into()
    }
}

pub trait AllowStatePatching {}

#[async_trait]
pub trait StatePatcher {
    async fn patch_state(
        &self,
        contract_id: AccountId,
        key: String,
        value: Vec<u8>,
    ) -> anyhow::Result<()>;
}

#[async_trait]
impl<T> StatePatcher for T
where
    T: AllowStatePatching + NetworkClient + Send + Sync,
{
    async fn patch_state(
        &self,
        contract_id: AccountId,
        key: String,
        value: Vec<u8>,
    ) -> anyhow::Result<()> {
        let state = StateRecord::Data {
            account_id: contract_id,
            data_key: key.into(),
            value,
        };
        let records = vec![state];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| anyhow::anyhow!("Failed to patch state: {:?}", err))?;

        Ok(())
    }
}

pub trait Network: TopLevelAccountCreator + NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: TopLevelAccountCreator + NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: AllowDevAccountCreation + Network {}

// Implemented by default if we have `AllowDevAccountCreation`
impl<T> DevNetwork for T where T: AllowDevAccountCreation + Network {}
