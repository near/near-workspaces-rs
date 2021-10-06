pub(crate) mod context;
pub(crate) mod local;
pub(crate) mod online;

pub use local::SandboxRuntime;
pub use online::TestnetRuntime;

use std::path::PathBuf;
use anyhow::anyhow;

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
            Self::Testnet => online::RPC_URL.to_string(),
            _ => unimplemented!(),
        }
    }

    pub fn home_dir(&self) -> PathBuf {
        match self {
            Self::Sandbox(port) => local::home_dir(*port),
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
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not get HOME_DIR".to_string()))?;
        let mut path = PathBuf::from(&home_dir);
        path.push(match self {
            Self::Sandbox(_) => SANDBOX_CREDENTIALS_DIR,
            Self::Testnet => TESTNET_CREDENTIALS_DIR,
            _ => unimplemented!(),
        });

        Ok(path)
    }
}

pub(crate) fn assert_within(runtimes: &[&str]) -> bool {
    runtimes.contains(
        &crate::runtime::context::current()
            .expect(crate::runtime::context::MISSING_RUNTIME_ERROR)
            .name(),
    )
}
