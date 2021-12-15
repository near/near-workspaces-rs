use serde_json::json;
use workspaces::prelude::*;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await?;

    let outcome = worker
        .call(
            &contract,
            "set_status".into(),
            json!({
                "message": "hello_world",
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await?;
    println!("set_status: {:?}", outcome);

    let result: String = worker
        .view(
            contract.id().clone(),
            "get_status".into(),
            json!({
                "account_id": contract.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .try_serde_deser()?;

    println!("status: {:?}", result);

    Ok(())
}
