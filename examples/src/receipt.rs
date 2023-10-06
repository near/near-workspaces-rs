const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call("set_status")
        .args_json(serde_json::json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    //  let receipt_ref = receipt(&outcome.outcome().receipt_ids)?;
    let ids = &outcome.outcome().receipt_ids;
    println!("receipts found: {ids:?}");

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let resp = worker.receipt(ids).await?;

    println!("ReceiptView: {resp:?}");
    Ok(())
}
