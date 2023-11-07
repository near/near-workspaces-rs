use near_primitives::{types::BlockReference, views::StateChangesRequestView};
use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.root_account()?.deploy(&wasm).await?.into_result()?;

    let outcome = contract
        .call("set_status")
        .args_json(json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    let block_ref = {
        let hash = near_primitives::hash::CryptoHash(outcome.outcome().block_hash.0);
        BlockReference::BlockId(near_primitives::types::BlockId::Hash(hash))
    };

    let state_changes = {
        StateChangesRequestView::ContractCodeChanges {
            account_ids: vec![contract.id().clone()],
        }
    };

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let res = worker.changes(block_ref, state_changes).await?;

    // Example output:
    //
    // StateChangesInBlock RpcStateChangesInBlockResponse {
    //     block_hash: 5SnL82tfQX1NtsSuqU5334ThZxM1B5KkUWUbeeMvVNRH,
    //     changes: [],
    // }
    println!("StateChangesInBlock {res:#?}");
    Ok(())
}
