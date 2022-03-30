use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use url::Url;

use near_primitives::views::{ExecutionStatusView, FinalExecutionStatus};

use crate::network::Info;
use crate::network::{
    Account, AllowDevAccountCreation, CallExecution, CallExecutionDetails, NetworkClient,
    NetworkInfo, TopLevelAccountCreator,
};
use crate::result::ExecutionOutcome;
use crate::rpc::{client::Client, tool};
use crate::types::{AccountId, InMemorySigner, SecretKey};
use crate::{Contract, CryptoHash};

const RPC_URL: &str = "https://rpc.testnet.near.org";
const HELPER_URL: &str = "https://helper.testnet.near.org";
const ARCHIVAL_URL: &str = "https://archival-rpc.testnet.near.org";

pub struct Testnet {
    client: Client,
    info: Info,
}

impl Testnet {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::new(RPC_URL.into()),
            info: Info {
                name: "testnet".into(),
                root_id: AccountId::from_str("testnet").unwrap(),
                keystore_path: PathBuf::from(".near-credentials/testnet/"),
                rpc_url: RPC_URL.into(),
            },
        }
    }

    pub(crate) fn archival() -> Self {
        Self {
            client: Client::new(ARCHIVAL_URL.into()),
            info: Info {
                name: "testnet-archival".into(),
                root_id: AccountId::from_str("testnet").unwrap(),
                keystore_path: PathBuf::from(".near-credentials/testnet/"),
                rpc_url: ARCHIVAL_URL.into(),
            },
        }
    }
}

impl AllowDevAccountCreation for Testnet {}

#[async_trait]
impl TopLevelAccountCreator for Testnet {
    async fn create_tla(
        &self,
        id: AccountId,
        sk: SecretKey,
        // TODO: return Account only, but then you don't get metadata info for it...
    ) -> anyhow::Result<CallExecution<Account>> {
        tool::url_create_account(Url::parse(HELPER_URL)?, id.clone(), sk.public_key()).await?;
        let signer = InMemorySigner::from_secret_key(id.clone(), sk);

        Ok(CallExecution {
            result: Account::new(id, signer),
            details: CallExecutionDetails {
                // We technically have not burnt any gas ourselves since someone else paid to
                // create the account for us in testnet when we used the Helper contract.
                total_gas_burnt: 0,

                status: FinalExecutionStatus::SuccessValue(String::new()),

                transaction: ExecutionOutcome {
                    block_hash: CryptoHash::default(),
                    logs: Vec::new(),
                    receipt_ids: Vec::new(),
                    gas_burnt: 0,
                    executor_id: "testnet".parse()?,
                    status: ExecutionStatusView::SuccessValue(String::new()),
                },
                receipts: Vec::new(),
            },
        })
    }

    async fn create_tla_and_deploy(
        &self,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> anyhow::Result<CallExecution<Contract>> {
        let signer = InMemorySigner::from_secret_key(id.clone(), sk.clone());
        let account = self.create_tla(id.clone(), sk).await?;
        let account = account.into_result()?;

        let outcome = self.client.deploy(&signer, &id, wasm.into()).await?;

        Ok(CallExecution {
            result: Contract::account(account),
            details: outcome.into(),
        })
    }
}

impl NetworkClient for Testnet {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Testnet {
    fn info(&self) -> &Info {
        &self.info
    }
}
