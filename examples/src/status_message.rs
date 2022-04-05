use serde_json::json;
use workspaces::prelude::*;

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
        }))?
        .transact()
        .await?;
    println!("set_status: {:?}", outcome);

    let result: String = contract
        .view(
            "get_status",
            json!({
                "account_id": contract.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    println!("status: {:?}", result);

    Ok(())
}
