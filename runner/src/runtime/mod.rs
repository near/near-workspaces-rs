pub(crate) mod local;
pub(crate) mod context;

pub use local::SandboxRuntime;

use std::path::PathBuf;


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
            Self::Sandbox(port) => sandbox_home_dir(*port),
            _ => unimplemented!(),
        }
    }
}


fn sandbox_home_dir(port: u16) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("sandbox-{}", port));
    path
}