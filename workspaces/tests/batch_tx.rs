use serde_json::json;
use test_log::test;
use workspaces::operations::Function;
use workspaces::prelude::*;

#[test(tokio::test)]
async fn test_batch_tx() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;

    // Batch transaction with two `call`s into `set_status`. The second one
    // should override the first one.
    contract
        .batch(&worker)
        .call(
            Function::new("set_status")
                .args_json(json!({
                    "message": "hello_world",
                }))
                .deposit(0),
        )
        .call(Function::new("set_status").args_json(json!({
            "message": "world_hello",
        })))
        .transact()
        .await?;

    let status_msg: String = contract
        .call(&worker, "get_status")
        .args_json(serde_json::json!({
            "account_id": contract.id(),
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(status_msg, "world_hello");
    Ok(())
}
