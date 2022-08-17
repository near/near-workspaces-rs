#![recursion_limit = "256"]
use near_units::parse_near;
use test_log::test;
use workspaces::{Contract, DevNetwork, Worker};

async fn init(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    let contract = worker
        .dev_deploy(include_bytes!("../../examples/res/fungible_token.wasm"))
        .await?;

    contract
        .call("new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": contract.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))
        .transact()
        .await?
        .ok()?;

    Ok(contract)
}

#[test(tokio::test)]
async fn test_empty_args_error() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = init(&worker).await?;

    let res = contract
        .call("storage_unregister")
        .max_gas()
        .deposit(1)
        .transact()
        .await?;

    match res.ok() {
        Ok(()) => panic!("Expected error: Failed to deserialize input from JSON"),
        Err(err) => match err {
            workspaces::error::Error::ExecutionError(msg) => {
                assert!(msg.contains("Failed to deserialize input from JSON"));
            }
            other => panic!("Expected ExecutionError, got: {:?}", other),
        },
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_optional_args_present() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = init(&worker).await?;

    let res = contract
        .call("storage_unregister")
        .args_json(serde_json::json!({
            "force": true
        }))
        .max_gas()
        .deposit(1)
        .transact()
        .await?;
    assert!(res.json::<bool>()?);

    Ok(())
}
