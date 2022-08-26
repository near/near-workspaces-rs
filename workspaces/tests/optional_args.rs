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
        .into_result()?;

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
        .await?
        .into_result();

    if let Some(exeuction_err) = res.err() {
        assert!(format!("{}", exeuction_err).contains("Failed to deserialize input from JSON"));
        assert!(
            exeuction_err.total_gas_burnt > 0,
            "Gas is still burnt for transaction although inputs are incorrect"
        );
    } else {
        panic!("Expected execution to error out");
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
