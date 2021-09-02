pub(crate) mod context;
pub(crate) mod local;

pub use local::SandboxRuntime;

use std::path::PathBuf;

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
            _ => unimplemented!(),
        }
    }

    pub fn home_dir(&self) -> PathBuf {
        match self {
            Self::Sandbox(port) => local::home_dir(*port),
            _ => unimplemented!(),
        }
    }
}
