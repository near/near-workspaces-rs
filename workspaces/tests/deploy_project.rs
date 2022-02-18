#![recursion_limit = "256"]
use test_log::test;
use workspaces::prelude::*;

#[test(tokio::test)]
async fn test_dev_deploy_project() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let contract = worker
        .dev_deploy_project("./tests/test-contracts/ft")
        .await?;

    let _res = contract
        .call(&worker, "new_default_meta")
        .args_json((contract.id(), "100"))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    let res = contract.call(&worker, "ft_total_supply").view().await?;
    assert_eq!(res.json::<String>()?, "100");

    Ok(())
}
