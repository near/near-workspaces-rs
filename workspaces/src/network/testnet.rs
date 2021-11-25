use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use async_trait::async_trait;
use url::Url;

use near_crypto::{InMemorySigner, Signer};
use near_primitives::{types::AccountId, views::FinalExecutionStatus};

use crate::network::{
    Account, AllowDevAccountCreation, CallExecution, NetworkActions, NetworkClient, NetworkInfo,
    TopLevelAccountCreator,
};
use crate::rpc::{client::Client, tool};
use crate::{CallExecutionResult, Contract};

const RPC_URL: &str = "https://rpc.testnet.near.org";
const HELPER_URL: &str = "https://helper.testnet.near.org";

pub struct Testnet {
    client: Client,
}

impl Testnet {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::new(RPC_URL.into()),
        }
    }
}

impl AllowDevAccountCreation for Testnet {}

// TODO: maybe sig should take InMmeorySigner instead of id/pk
#[async_trait]
impl TopLevelAccountCreator for Testnet {
    async fn create_tla(
        &self,
        id: AccountId,
        signer: InMemorySigner,
    ) -> anyhow::Result<CallExecution<Account>> {
        tool::url_create_account(
            Url::parse(&self.helper_url())?,
            id.clone(),
            signer.public_key(),
        )
        .await?;

        Ok(CallExecution {
            result: Account::new(id, signer),
            details: CallExecutionResult {
                // We technically have not burnt any gas ourselves since someone else paid to
                // create the account for us in testnet when we used the Helper contract.
                total_gas_burnt: 0,

                status: FinalExecutionStatus::SuccessValue(String::new()),
            },
        })
    }

    async fn create_tla_and_deploy<P: AsRef<Path> + Send + Sync>(
        &self,
        id: AccountId,
        signer: InMemorySigner,
        wasm: P,
    ) -> anyhow::Result<CallExecution<Contract>> {
        // TODO: async_compat/async version of File
        let mut code = Vec::new();
        File::open(wasm)?.read_to_end(&mut code)?;

        let account = self.create_tla(id.clone(), signer.clone()).await?;
        let account = Into::<anyhow::Result<_>>::into(account)?;
        let outcome = self.client.deploy(&signer, id, code).await?;

        Ok(CallExecution {
            result: Contract::account(account),
            details: outcome.into(),
        })
    }
}

impl NetworkInfo for Testnet {
    fn name(&self) -> String {
        "testnet".into()
    }

    fn root_account_id(&self) -> AccountId {
        AccountId::from_str("testnet").unwrap()
    }

    fn keystore_path(&self) -> PathBuf {
        PathBuf::from(".near-credentials/testnet/")
    }

    fn rpc_url(&self) -> String {
        RPC_URL.into()
    }

    fn helper_url(&self) -> String {
        HELPER_URL.into()
    }
}

impl NetworkClient for Testnet {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkActions for Testnet {}
