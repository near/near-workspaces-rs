use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use url::Url;

use near_primitives::views::ExecutionStatusView;

use crate::network::Info;
use crate::network::{AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::result::{Execution, ExecutionDetails, ExecutionFinalResult, ExecutionOutcome, Result};
use crate::rpc::{client::Client, tool};
use crate::types::{AccountId, InMemorySigner, SecretKey};
use crate::{Account, Contract, CryptoHash, Network, Worker};

const RPC_URL: &str = "https://rpc.testnet.near.org";
const HELPER_URL: &str = "https://helper.testnet.near.org";
const ARCHIVAL_URL: &str = "https://archival-rpc.testnet.near.org";

/// Testnet related configuration for interacting with testnet. Look at
/// [`workspaces::testnet`] and [`workspaces::testnet_archival`] for how
/// to spin up a [`Worker`] that can be used to run tests in testnet.
///
/// [`workspaces::testnet`]: crate::testnet
/// [`workspaces::testnet_archival`]: crate::testnet_archival
/// [`Worker`]: crate::Worker
pub struct Testnet {
    client: Client,
    info: Info,
}

impl Testnet {
    pub(crate) async fn new() -> Result<Self> {
        let client = Client::new(RPC_URL);
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: "testnet".into(),
                root_id: AccountId::from_str("testnet").unwrap(),
                keystore_path: PathBuf::from(".near-credentials/testnet/"),
                rpc_url: RPC_URL.into(),
            },
        })
    }

    pub(crate) async fn archival() -> Result<Self> {
        let client = Client::new(ARCHIVAL_URL);
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: "testnet-archival".into(),
                root_id: AccountId::from_str("testnet").unwrap(),
                keystore_path: PathBuf::from(".near-credentials/testnet/"),
                rpc_url: ARCHIVAL_URL.into(),
            },
        })
    }
}

impl std::fmt::Debug for Testnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Testnet")
            .field("root_id", &self.info.root_id)
            .field("rpc_url", &self.info.rpc_url)
            .finish()
    }
}

impl AllowDevAccountCreation for Testnet {}

#[async_trait]
impl TopLevelAccountCreator for Testnet {
    async fn create_tla(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        // TODO: return Account only, but then you don't get metadata info for it...
    ) -> Result<Execution<Account>> {
        let url = Url::parse(HELPER_URL).unwrap();
        tool::url_create_account(url, id.clone(), sk.public_key()).await?;
        let signer = InMemorySigner::from_secret_key(id.clone(), sk);

        Ok(Execution {
            result: Account::new(id, signer, worker),
            details: ExecutionFinalResult {
                // We technically have not burnt any gas ourselves since someone else paid to
                // create the account for us in testnet when we used the Helper contract.
                total_gas_burnt: 0,

                status: near_primitives::views::FinalExecutionStatus::SuccessValue(String::new()),
                details: ExecutionDetails {
                    transaction: ExecutionOutcome {
                        block_hash: CryptoHash::default(),
                        logs: Vec::new(),
                        receipt_ids: Vec::new(),
                        gas_burnt: 0,
                        tokens_burnt: 0,
                        executor_id: "testnet".parse().unwrap(),
                        status: ExecutionStatusView::SuccessValue(String::new()),
                    },
                    receipts: Vec::new(),
                },
            },
        })
    }

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        let signer = InMemorySigner::from_secret_key(id.clone(), sk.clone());
        let account = self.create_tla(worker, id.clone(), sk).await?;

        let outcome = self.client().deploy(&signer, &id, wasm.into()).await?;

        Ok(Execution {
            result: Contract::account(account.into_result()?),
            details: ExecutionFinalResult::from_view(outcome),
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
