use crate::error::{ErrorKind, SandboxErrorCode};
use crate::result::Result;

use async_process::Child;
use portpicker::pick_unused_port;
use tempfile::TempDir;
use tracing::info;

use near_sandbox_utils as sandbox;

async fn init() -> Result<TempDir> {
    let home_dir = tempfile::tempdir().map_err(|e| ErrorKind::Io.custom(e))?;
    let output = sandbox::init(&home_dir)
        .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?
        .output()
        .await
        .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?;
    info!(target: "workspaces", "sandbox init: {:?}", output);

    Ok(home_dir)
}

pub struct SandboxServer {
    pub(crate) rpc_port: u16,
    pub(crate) net_port: u16,
    pub(crate) home_dir: TempDir,
    process: Option<Child>,
}

impl SandboxServer {
    pub(crate) async fn run_new() -> Result<Self> {
        let home_dir = init().await?;

        // Supress logs for the sandbox binary by default:
        supress_sandbox_logs_if_required();

        let (rpc_port, net_port, child) = loop {
            // Try running the server with the follow provided rpc_ports and net_ports
            let rpc_port = pick_unused_port().expect("no ports free");
            let net_port = pick_unused_port().expect("no ports free");
            match sandbox::run(home_dir.path(), rpc_port, net_port) {
                Ok(child) => break (rpc_port, net_port, child),
                Err(err) => {
                    if format!("{:?}", err).contains("AddrInUse") {
                        // continue the search for an unused port.
                        continue;
                    } else {
                        return Err(SandboxErrorCode::RunFailure.custom(err));
                    }
                }
            }
        };

        info!(target: "workspaces", "Started up sandbox at localhost:{} with pid={:?}", rpc_port, child.id());

        Ok(Self {
            rpc_port,
            net_port,
            home_dir,
            process: Some(child),
        })
    }

    pub fn rpc_addr(&self) -> String {
        format!("http://localhost:{}", self.rpc_port)
    }
}

impl Drop for SandboxServer {
    fn drop(&mut self) {
        if self.process.is_none() {
            return;
        }

        let child = self.process.as_mut().unwrap();

        info!(
            target: "workspaces",
            "Cleaning up sandbox: port={}, pid={}",
            self.rpc_port,
            child.id()
        );

        child
            .kill()
            .map_err(|e| format!("Could not cleanup sandbox due to: {:?}", e))
            .unwrap();
    }
}

/// Turn off neard-sandbox logs by default. Users can turn them back on with
/// NEAR_ENABLE_SANDBOX_LOG=1 and specify further paramters with the custom
/// NEAR_SANDBOX_LOG for higher levels of specificity. NEAR_SANDBOX_LOG args
/// will be forward into RUST_LOG environment variable as to not conflict
/// with similar named log targets.
fn supress_sandbox_logs_if_required() {
    if let Ok(val) = std::env::var("NEAR_ENABLE_SANDBOX_LOG") {
        if val != "0" {
            return;
        }
    }

    // non-exhaustive list of targets to supress, since choosing a default LogLevel
    // does nothing in this case, since nearcore seems to be overriding it somehow:
    std::env::set_var("NEAR_SANDBOX_LOG", "near=error,stats=error,network=error");
}
