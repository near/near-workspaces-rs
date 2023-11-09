const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let status = contract
        .call("set_status")
        .args_json(serde_json::json!({
            "message": "hello_world",
        }))
        .transact_async()
        .await?;

    let outcome = status.await;
    println!(
        "Async transaction result from setting hello world: {:#?}",
        outcome
    );

    Ok(())
}
