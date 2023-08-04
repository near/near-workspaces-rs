const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call("set_status")
        .args_json(serde_json::json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    let block_reference = {
        let hash = outcome.outcome().block_hash;
        near_primitives::types::BlockReference::BlockId(near_primitives::types::BlockId::Hash(
            near_primitives::hash::CryptoHash(hash.0),
        ))
    };

    // NOTE: this API is under the "experimental" flag.
    let protocol_config = worker.protocol_config(block_reference).await?;

    println!("Protocol Config {protocol_config:?}");
    Ok(())
}
