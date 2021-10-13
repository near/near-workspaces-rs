use portpicker::pick_unused_port;
use std::fs;
use std::path::PathBuf;
use std::process::Child;
use std::{thread, time::Duration};

use super::context;
use super::RuntimeFlavor;

pub(crate) fn home_dir(port: u16) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("sandbox-{}", port));
    path
}

pub struct SandboxServer {
    pub(self) rpc_port: u16,
    pub(self) net_port: u16,
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
        println!("Starting up sandbox at localhost:{}", self.rpc_port);
        let home_dir = home_dir(self.rpc_port);

        // Remove dir if it already exists:
        let _ = fs::remove_dir_all(&home_dir);
        near_sandbox_utils::init(&home_dir)?.wait()?;

        let child = near_sandbox_utils::run(&home_dir, self.rpc_port, self.net_port)?;
        println!("Started sandbox: pid={:?}", child.id());
        self.process = Some(child);

        // TODO: Get rid of this sleep, and ping sandbox is alive instead:
        thread::sleep(Duration::from_secs(3));
        Ok(())
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

        eprintln!(
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

pub struct SandboxRuntime {
    server: SandboxServer,
    _guard: context::EnterGuard,
}

impl SandboxRuntime {
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.server.start()
    }
}

impl Default for SandboxRuntime {
    fn default() -> Self {
        let server = SandboxServer::default();
        let rpc_port = server.rpc_port;

        Self {
            server,
            _guard: context::enter(RuntimeFlavor::Sandbox(rpc_port)),
        }
    }
}
