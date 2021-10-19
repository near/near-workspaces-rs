use anyhow::anyhow;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use near_crypto::{PublicKey, Signer};
use near_primitives::transaction::{Action, DeployContractAction, SignedTransaction};
use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;

use super::context;
use super::RuntimeFlavor;
use crate::rpc::tool;
use crate::runtime::context::MISSING_RUNTIME_ERROR;

pub struct TestnetRuntime {
    _guard: context::EnterGuard,
}

impl TestnetRuntime {
    pub const RPC_URL: &'static str = "https://rpc.testnet.near.org";
    pub const HELPER_URL: &'static str = "https://helper.testnet.near.org";

    pub fn run(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for TestnetRuntime {
    fn default() -> Self {
        Self {
            _guard: context::enter(RuntimeFlavor::Testnet),
        }
    }
}

pub(crate) async fn create_top_level_account(
    new_account_id: AccountId,
    new_account_pk: PublicKey,
) -> anyhow::Result<()> {
    let rt = crate::runtime::context::current().expect(MISSING_RUNTIME_ERROR);
    let helper_url = rt.helper_url();
    tool::url_create_account(helper_url, new_account_id, new_account_pk).await
}

// TODO: Vec[ExecutionOutcomeView] due to tla account doing multiple transactions?
pub(crate) async fn create_tla_and_deploy(
    new_account_id: AccountId,
    new_account_pk: PublicKey,
    signer: &dyn Signer,
    code_filepath: impl AsRef<Path>,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    create_top_level_account(new_account_id.clone(), new_account_pk.clone()).await?;

    // TODO: backoff-and-retry: two separate transactions, requires a sleep in between.
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let (access_key, _, block_hash) = tool::access_key(new_account_id.clone(), new_account_pk)
        .await
        .map_err(|e| anyhow!(e))?;

    let mut code = Vec::new();
    File::open(code_filepath)?.read_to_end(&mut code)?;

    let signed_tx = SignedTransaction::from_actions(
        access_key.nonce + 1,
        new_account_id.clone(),
        new_account_id.clone(),
        signer,
        vec![Action::DeployContract(DeployContractAction { code })],
        block_hash,
    );

    let transaction_info = tool::send_tx(signed_tx).await.map_err(|e| anyhow!(e))?;
    Ok(transaction_info)
}
