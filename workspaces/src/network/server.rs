use crate::network::Sandbox;
use portpicker::pick_unused_port;
use std::process::Child;
use tracing::info;

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

    pub fn start(&mut self) -> anyhow::Result<()> {
        info!(target: "workspaces", "Starting up sandbox at localhost:{}", self.rpc_port);
        let home_dir = Sandbox::home_dir(self.rpc_port);

        // Supress logs for the sandbox binary by default:
        supress_sandbox_logs_if_required();

        // Remove dir if it already exists:
        let _ = std::fs::remove_dir_all(&home_dir);
        near_sandbox_utils::init(&home_dir)?.wait()?;

        let child = near_sandbox_utils::run(&home_dir, self.rpc_port, self.net_port)?;
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
/// NEAR_ENABLE_SANDBOX_LOG=on and specify further paramters with the custom
/// NEAR_SANDBOX_LOG for higher levels of specificity. NEAR_SANDBOX_LOG args
/// will be forward into RUST_LOG environment variable as to not conflict
/// with similar named log targets.
fn supress_sandbox_logs_if_required() {
    if let Err(_) = std::env::var("NEAR_ENABLE_SANDBOX_LOG") {
        // non-exhaustive list of targets to supress, since choosing a default LogLevel
        // does nothing in this case, since nearcore seems to be overriding it somehow:
        std::env::set_var("NEAR_SANDBOX_LOG", "near=error,stats=error,network=error");
    };
}
