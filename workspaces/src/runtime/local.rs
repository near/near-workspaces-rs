use anyhow::anyhow;
use portpicker::pick_unused_port;

use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::{thread, time::Duration};

use near_crypto::{InMemorySigner, PublicKey, Signer};
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;

use super::context;
use super::RuntimeFlavor;
use crate::rpc::tool;
use crate::NEAR_BASE;

fn home_dir(port: u16) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("sandbox-{}", port));
    path
}

fn root_account() -> InMemorySigner {
    let rt = crate::runtime::context::current().expect(context::MISSING_RUNTIME_ERROR);
    let port = match rt {
        RuntimeFlavor::Sandbox(port) => port,
        _ => panic!("expected to be in sandbox runtime while retrieving port"),
    };

    let mut path = home_dir(port);
    path.push("validator_key.json");

    InMemorySigner::from_file(&path)
}

pub(crate) async fn create_top_level_account(
    new_account_id: AccountId,
    new_account_pk: PublicKey,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    let root_signer = root_account();
    crate::create_account(
        &root_signer,
        root_signer.account_id.clone(),
        new_account_id,
        new_account_pk,
        None,
    )
    .await
}

pub(crate) async fn create_tla_and_deploy(
    new_account_id: AccountId,
    new_account_pk: PublicKey,
    _signer: &dyn Signer,
    code_filepath: impl AsRef<Path>,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    let root_signer = root_account();
    let (access_key, _, block_hash) =
        tool::access_key(root_signer.account_id.clone(), root_signer.public_key())
            .await
            .map_err(|e| anyhow!(e))?;

    let mut code = Vec::new();
    File::open(code_filepath)?.read_to_end(&mut code)?;

    // This transaction creates the account too:
    let signed_tx = SignedTransaction::create_contract(
        access_key.nonce + 1,
        root_signer.account_id.clone(),
        new_account_id,
        code,
        100 * NEAR_BASE,
        new_account_pk,
        &root_signer,
        block_hash,
    );
    dbg!(&signed_tx);

    let transaction_info = tool::send_tx(signed_tx).await.map_err(|e| anyhow!(e))?;
    Ok(transaction_info)
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
