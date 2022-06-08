use near_sdk::json_types::U128;
use near_units::parse_near;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{AccountId, Contract, Network, Worker};

/// The factory contract used in these tests can be found in
/// [near-sdk/examples/factory-contract](https://github.com/near/near-sdk-rs/tree/master/examples/factory-contract/high-level).
const FACTORY_CONTRACT: &[u8] =
    include_bytes!("../../examples/res/factory_contract_high_level.wasm");

/// Create a new contract account through a cross contract call with "deploy_status_message".
async fn cross_contract_create_contract(
    status_id: &AccountId,
    status_amt: &U128,
    worker: &Worker<impl Network>,
    contract: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    contract
        .call(worker, "deploy_status_message")
        .args_json((status_id.clone(), status_amt))?
        .deposit(parse_near!("50 N"))
        .max_gas()
        .transact()
        .await
        .map_err(Into::into)
}

#[tokio::test]
async fn test_cross_contract_create_contract() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(FACTORY_CONTRACT).await?;
    let status_amt = U128::from(parse_near!("35 N"));

    // Expect to fail for trying to create a new contract account with too short of a
    // top level account name, such as purely just "status"
    let status_id: AccountId = "status".parse().unwrap();
    let outcome =
        cross_contract_create_contract(&status_id, &status_amt, &worker, &contract).await?;
    let failures = outcome.failures();
    assert!(
        failures.len() == 1,
        "Expected one receipt failure for creating too short of a TLA, but got {} failures",
        failures.len()
    );

    // Expect to succeed after calling into the contract with expected length for a
    // top level account.
    let status_id: AccountId = "status-top-level-account-long-name".parse().unwrap();
    let outcome =
        cross_contract_create_contract(&status_id, &status_amt, &worker, &contract).await?;
    let failures = outcome.failures();
    assert!(
        failures.is_empty(),
        "Expected no failures for creating a TLA, but got {} failures",
        failures.len(),
    );

    Ok(())
}

#[tokio::test]
async fn test_cross_contract_calls() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(FACTORY_CONTRACT).await?;
    let status_amt = U128::from(parse_near!("35 N"));

    let status_id: AccountId = "status-top-level-account-long-name".parse().unwrap();
    cross_contract_create_contract(&status_id, &status_amt, &worker, &contract).await?;

    let message = "hello world";
    let result = contract
        .call(&worker, "complex_call")
        .args_json((status_id, message))?
        .max_gas()
        .transact()
        .await?
        .json::<String>()?;
    assert_eq!(
        message, result,
        "Results from cross contract call do not match."
    );

    Ok(())
}
