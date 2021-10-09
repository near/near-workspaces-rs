// return {
//     rpcAddr: 'https://rpc.testnet.near.org',
//     walletUrl: 'https://wallet.testnet.near.org',
//     helperUrl: 'https://helper.testnet.near.org',
//     explorerUrl: 'https://explorer.testnet.near.org',
//   };
// use super::RuntimeConfig;
use super::context;
use super::RuntimeFlavor;

pub const RPC_URL: &str = "https://rpc.testnet.near.org";
pub const WALLET_URL: &str = "https://wallet.testnet.near.org";
pub const HELPER_URL: &str = "https://helper.testnet.near.org";
pub const EXPLORER_URL: &str = "https://explorer.testnet.near.org";
pub const HOME_DIR: &str = "";


pub struct TestnetRuntime {
    _guard: context::EnterGuard,
}

impl TestnetRuntime {
    pub fn run(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for TestnetRuntime {
    fn default() -> Self {
        Self { _guard: context::enter(RuntimeFlavor::Testnet) }
    }
}

pub struct MainnetRuntime {
    _guard: context::EnterGuard,
}

impl MainnetRuntime {
    pub fn run(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Default for MainnetRuntime {
    fn default() -> Self {
        Self { _guard: context::enter(RuntimeFlavor::Mainnet) }
    }
}







// impl RuntimeConfig for TestnetRuntime {
//     const BASE_ACCOUNT_ID: &'static str = "test.near";
// }
