use borsh::{self, BorshDeserialize, BorshSerialize};
use std::env;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use workspaces::prelude::*;
use workspaces::{AccountId, Contract, DevNetwork, Worker};

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

/// This is the cached contract_id from running `deploy_testnet` the first time. Used so we don't
/// overload testnet and have to go through a couple more cycles than we have to, to showcase spooning.
///
/// If you'd like a different account to deploy it to, run the following:
/// ```norun
/// async fn deploy_testnet() -> anyhow::Result<()> {
///     let worker = worspaces::testnet().await?;
///
///     let contract = deploy_status_contract(worker, "hello from testnet").await?;
///     println!("{}", contract.id());
/// }
/// ```
const TESTNET_PREDEPLOYED_CONTRACT_ID: &str = "dev-20211013002148-59466083160385";

// The following two structs (Record and StatusMessage) are representation of the
// internal data stored on chain. They will be deserialized into the following
// formats. Note that these will be different depending on what data structure
// we use in our contract.
#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
struct Record {
    k: String,
    v: String,
}

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
struct StatusMessage {
    records: Vec<Record>,
}

/// Deploy a status message smart contract (https://examples.near.org/rust-status-message)
/// with an attached message associated to the contract id.
///
/// For example, our predeployed testnet contract has already done this:
///    set_status(TESTNET_PREDEPLOYED_CONTRACT_ID) = "hello from testnet"
async fn deploy_status_contract(
    worker: &Worker<impl DevNetwork>,
    msg: &str,
) -> anyhow::Result<Contract> {
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // This will `call` into `set_status` with the message we want to set.
    contract
        .call(worker, "set_status")
        .args_json(serde_json::json!({
            "message": msg,
        }))?
        .transact()
        .await?;

    Ok(contract)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse log filters from RUST_LOG or fallback to INFO if empty
    let filter = if env::var(EnvFilter::DEFAULT_ENV).is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::default().add_directive(LevelFilter::INFO.into())
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Grab STATE from the testnet status_message contract. This contract contains the following data:
    //   get_status(dev-20211013002148-59466083160385) => "hello from testnet"
    let (testnet_contract_id, status_msg) = {
        let worker = workspaces::testnet().await?;
        let contract_id: AccountId = TESTNET_PREDEPLOYED_CONTRACT_ID
            .parse()
            .map_err(anyhow::Error::msg)?;

        let mut state_items = worker.view_state(&contract_id, None).await?;

        let state = state_items.remove(b"STATE".as_slice()).unwrap();
        let status_msg = StatusMessage::try_from_slice(&state)?;

        (contract_id, status_msg)
    };

    info!(target: "spooning", "Testnet: {:?}", status_msg);

    // Create our sandboxed environment and grab a worker to do stuff in it:
    let worker = workspaces::sandbox().await?;

    // Deploy with the following status_message state: sandbox_contract_id => "hello from sandbox"
    let sandbox_contract = deploy_status_contract(&worker, "hello from sandbox").await?;

    // Patch our testnet STATE into our local sandbox:
    worker
        .patch_state(sandbox_contract.id(), "STATE", status_msg.try_to_vec()?)
        .await?;

    // Now grab the state to see that it has indeed been patched:
    let status: String = sandbox_contract
        .view(
            &worker,
            "get_status",
            serde_json::json!({
                "account_id": testnet_contract_id,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    info!(target: "spooning", "New status patched: {:?}", status);
    assert_eq!(&status, "hello from testnet");

    // See that sandbox state was overriden. Grabbing get_status(sandbox_contract_id) should yield Null
    let result: Option<String> = sandbox_contract
        .view(
            &worker,
            "get_status",
            serde_json::json!({
                "account_id": sandbox_contract.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(result, None);

    Ok(())
}
