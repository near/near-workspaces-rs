mod account;
mod info;
mod mainnet;
mod result;
mod sandbox;
mod server;
mod testnet;

use anyhow::anyhow;
use async_trait::async_trait;
use std::fs;

use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;

pub(crate) use crate::network::info::Info;
use crate::rpc::client::Client;
use crate::rpc::patch::ImportContractBuilder;
use crate::types::{AccountId, KeyType, SecretKey};
use crate::{cargo, Worker};

pub use crate::network::account::{Account, Contract};
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

    /// Builds the cargo project located at `project_path` and deploys the resulting smart
    /// contract using a freshly generated dev account.
    ///
    /// NOTE: This function does not check whether the resulting wasm file is a valid smart
    /// contract or not.
    async fn dev_deploy_project(&self, project_path: &str) -> anyhow::Result<Contract>;
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

    async fn dev_deploy_project(&self, project_path: &str) -> anyhow::Result<Contract> {
        let messages = cargo::build_cargo_project(fs::canonicalize(project_path)?)?;
        // We find the last compiler artifact message which should contain information about the
        // resulting .wasm file
        let compile_artifact = messages
            .iter()
            .filter_map(|m| match m {
                cargo_metadata::Message::CompilerArtifact(artifact) => Some(artifact),
                _ => None,
            })
            .last()
            .ok_or(anyhow!(
                "Cargo failed to produce any compilation artifacts. \
                 Please check that your project contains a NEAR smart contract."
            ))?;
        // The project could have generated many auxiliary files, we are only interested in .wasm files
        let wasm_files = compile_artifact
            .filenames
            .iter()
            .filter(|f| f.as_str().ends_with(".wasm"))
            .collect::<Vec<_>>();
        if wasm_files.len() == 0 {
            Err(anyhow!(
                "Compilation resulted in no '.wasm' target files. \
                 Please check that your project contains a NEAR smart contract."
            ))
        } else if wasm_files.len() > 1 {
            Err(anyhow!(
                "Compilation resulted in more than one '.wasm' target file: {:?}",
                wasm_files
            ))
        } else {
            let file = wasm_files.first().unwrap();
            let wasm = fs::read(file.canonicalize()?)?;
            self.dev_deploy(&wasm).await
        }
    }
}

pub trait AllowStatePatching {}

#[async_trait]
pub trait StatePatcher {
    async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: String,
        value: Vec<u8>,
    ) -> anyhow::Result<()>;

    fn import_contract<'a, 'b>(
        &'b self,
        id: &AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractBuilder<'a, 'b>;
}

#[async_trait]
impl<T> StatePatcher for T
where
    T: AllowStatePatching + NetworkClient + Send + Sync,
{
    async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: String,
        value: Vec<u8>,
    ) -> anyhow::Result<()> {
        let state = StateRecord::Data {
            account_id: contract_id.to_owned(),
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

    fn import_contract<'a, 'b>(
        &'b self,
        id: &AccountId,
        worker: &'a Worker<impl Network>,
    ) -> ImportContractBuilder<'a, 'b> {
        ImportContractBuilder::new(id.to_owned(), worker.client(), self.client())
    }
}

pub trait Network: TopLevelAccountCreator + NetworkInfo + NetworkClient + Send + Sync {}

impl<T> Network for T where T: TopLevelAccountCreator + NetworkInfo + NetworkClient + Send + Sync {}

/// DevNetwork is a Network that can call into `dev_create` and `dev_deploy` to create developer accounts.
pub trait DevNetwork: AllowDevAccountCreation + Network {}

// Implemented by default if we have `AllowDevAccountCreation`
impl<T> DevNetwork for T where T: AllowDevAccountCreation + Network {}
