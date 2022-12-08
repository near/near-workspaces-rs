use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::error::{ErrorKind, SandboxErrorCode};
use crate::network::Sandbox;
use crate::result::Result;

use async_process::Child;
use portpicker::pick_unused_port;
use tracing::info;

use near_sandbox_utils as sandbox;

pub struct SandboxServer {
    pub(crate) rpc_port: u16,
    pub(crate) net_port: u16,
    process: Option<Child>,
}

impl SandboxServer {
    pub fn new(rpc_port: u16, net_port: u16) -> Self {
        Self {
            rpc_port,
            net_port,
            process: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.process.is_some() {
            return Err(SandboxErrorCode::AlreadyStarted.into());
        }

        info!(target: "workspaces", "Starting up sandbox at localhost:{}", self.rpc_port);
        let home_dir = Sandbox::home_dir(self.rpc_port);

        // Supress logs for the sandbox binary by default:
        supress_sandbox_logs_if_required();

        // Remove dir if it already exists:
        let _ = std::fs::remove_dir_all(&home_dir);
        let output = sandbox::init(&home_dir)
            .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?
            .output()
            .await
            .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?;
        info!(target: "workspaces", "sandbox init: {:?}", output);

        // Configure `$home_dir/config.json` to our liking. Sandbox requires extra settings
        // for the best user experience, and being able to offer patching large state payloads.
        crate::network::config::set_sandbox_configs(&home_dir)?;

        let child = sandbox::run(&home_dir, self.rpc_port, self.net_port)
            .map_err(|e| SandboxErrorCode::RunFailure.custom(e))?;

        info!(target: "workspaces", "Started sandbox: pid={:?}", child.id());
        self.process = Some(child);

        Ok(())
    }

    pub fn rpc_addr(&self) -> String {
        format!("http://localhost:{}", self.rpc_port)
    }
}

impl Default for SandboxServer {
    fn default() -> Self {
        let rpc_port = pick_unused_port().expect("no ports free");
        let net_port = pick_unused_port().expect("no ports free");
        Self::new(rpc_port, net_port)
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
