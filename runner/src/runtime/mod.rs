pub(crate) mod context;
pub(crate) mod local;
pub(crate) mod online;

pub use local::SandboxRuntime;
pub use online::TestnetRuntime;

use anyhow::anyhow;
use std::path::{Path, PathBuf};
use url::Url;

use near_crypto::{PublicKey, Signer};
use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;

const SANDBOX_CREDENTIALS_DIR: &str = ".near-credentials/sandbox/";
const TESTNET_CREDENTIALS_DIR: &str = ".near-credentials/testnet/runner";

// TODO: implement mainnet/testnet runtimes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum RuntimeFlavor {
    Mainnet,
    Testnet,
    Sandbox(u16),
}

impl RuntimeFlavor {
    pub fn rpc_addr(&self) -> String {
        match self {
            Self::Sandbox(port) => format!("http://localhost:{}", port),
            Self::Testnet => online::TestnetRuntime::RPC_URL.to_string(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Sandbox(_) => "sandbox",
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub fn keystore_path(&self) -> anyhow::Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow!("Could not get HOME_DIR".to_string()))?;
        let mut path = PathBuf::from(&home_dir);
        path.push(match self {
            Self::Sandbox(_) => SANDBOX_CREDENTIALS_DIR,
            Self::Testnet => TESTNET_CREDENTIALS_DIR,
            _ => unimplemented!(),
        });

        Ok(path)
    }

    pub fn helper_url(&self) -> Url {
        match self {
            Self::Testnet => Url::parse(online::TestnetRuntime::HELPER_URL).unwrap(),
            _ => unimplemented!(),
        }
    }

    pub async fn create_tla_account(
        &self,
        new_account_id: AccountId,
        new_account_pk: PublicKey,
    ) -> anyhow::Result<Option<FinalExecutionOutcomeView>> {
        match self {
            Self::Sandbox(_) => Ok(Some(
                local::create_tla_account(new_account_id, new_account_pk).await?,
            )),
            Self::Testnet => {
                online::create_tla_account(new_account_id, new_account_pk).await?;
                Ok(None)
            }
            _ => unimplemented!(),
        }
    }

    pub async fn create_tla_and_deploy(
        &self,
        new_account_id: AccountId,
        new_account_pk: PublicKey,
        signer: &dyn Signer,
        code_filepath: impl AsRef<Path>,
    ) -> anyhow::Result<FinalExecutionOutcomeView> {
        match self {
            Self::Sandbox(_) => {
                local::create_tla_and_deploy(new_account_id, new_account_pk, signer, code_filepath)
                    .await
            }
            Self::Testnet => {
                online::create_tla_and_deploy(new_account_id, new_account_pk, signer, code_filepath)
                    .await
            }
            _ => unimplemented!(),
        }
    }
}

pub(crate) fn assert_within(runtimes: &[&str]) -> bool {
    runtimes.contains(
        &crate::runtime::context::current()
            .expect(crate::runtime::context::MISSING_RUNTIME_ERROR)
            .name(),
    )
}
