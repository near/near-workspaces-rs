use near_primitives::types::BlockReference;
use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

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

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let res = worker.changes_in_block(block_ref).await?;

    println!("StateChangesInBlockByType {res:?}");
    Ok(())
}
