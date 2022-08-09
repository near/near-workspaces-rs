#![recursion_limit = "256"]
use near_units::parse_near;
use test_log::test;
use workspaces::prelude::*;
use workspaces::{Contract, DevNetwork, Worker};

async fn init(worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    let contract = worker
        .dev_deploy(include_bytes!("../../examples/res/fungible_token.wasm"))
        .await?;

    contract
        .call(worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": contract.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))
        .transact()
        .await?;

    Ok(contract)
}

#[test(tokio::test)]
async fn test_empty_args_error() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = init(&worker).await?;

    let res = contract
        .call(&worker, "storage_unregister")
        .max_gas()
        .deposit(1)
        .transact()
        .await;
    assert!(format!("{:?}", res.unwrap_err()).contains("Failed to deserialize input from JSON"));

    Ok(())
}

#[test(tokio::test)]
async fn test_optional_args_present() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = init(&worker).await?;

    let res = contract
        .call(&worker, "storage_unregister")
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
