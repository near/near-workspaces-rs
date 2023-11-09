use near_workspaces::network::Sandbox;
use near_workspaces::{Account, Contract, Worker};
use serde_json::json;
use test_log::test;

use near_workspaces::operations::Function;
use near_workspaces::types::{Gas, GasMeter, NearToken};

#[test(tokio::test)]
async fn test_gas_meter_with_single_transaction() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = NearToken::from_yoctonear(0);

    let status_msg = analogous_to_dev_deploy(&worker, &mut total_gas).await?;

    // analogous to: worker.dev_create_account().await?;
    let account = analogous_to_dev_create_account(&worker, &mut total_gas).await?;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_multiple_transactions() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = NearToken::from_yoctonear(0);

    let status_msg = analogous_to_dev_deploy(&worker, &mut total_gas).await?;

    let account = analogous_to_dev_create_account(&worker, &mut total_gas).await?;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_parallel_transactions() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = NearToken::from_yoctonear(0);

    let status_msg = analogous_to_dev_deploy(&worker, &mut total_gas).await?;

    let account = analogous_to_dev_create_account(&worker, &mut total_gas).await?;

    let mut tasks = Vec::new();

    for _ in 0..10 {
        let account = account.clone();
        let status_msg = status_msg.clone();

        tasks.push(tokio::spawn(async move {
            let txn = account
                .call(status_msg.id(), "set_status")
                .args_json(serde_json::json!({
                    "message": "hello world",
                }))
                .transact()
                .await?;

            Ok::<_, anyhow::Error>(txn.total_gas_burnt)
        }));
    }

    for task in tasks {
        total_gas = total_gas.checked_add(as_near(task.await??)).unwrap();
    }

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_multiple_transactions_and_view() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = NearToken::from_yoctonear(0);

    let status_msg = analogous_to_dev_deploy(&worker, &mut total_gas).await?;

    let account = analogous_to_dev_create_account(&worker, &mut total_gas).await?;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    let _ = account
        .call(status_msg.id(), "get_status")
        .args_json(serde_json::json!({
            "account_id": account.id(),
        }))
        .view()
        .await?;

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_batch_tx() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = NearToken::from_yoctonear(0);

    let contract = analogous_to_dev_deploy(&worker, &mut total_gas).await?;

    let txn = contract
        .batch()
        .call(
            Function::new("set_status")
                .args_json(json!({
                    "message": "hello_world",
                }))
                .deposit(NearToken::from_near(0)),
        )
        .call(Function::new("set_status").args_json(json!({
            "message": "world_hello",
        })))
        .transact()
        .await?;
    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    let txn = contract
        .batch()
        .call(
            Function::new("set_status")
                .args_json(json!({
                    "message": "hello_world",
                }))
                .deposit(NearToken::from_near(0)),
        )
        .call(Function::new("set_status").args_json(json!({
            "message": "world_hello",
        })))
        .transact()
        .await?;

    total_gas = total_gas.checked_add(as_near(txn.total_gas_burnt)).unwrap();

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_create_account_transaction() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = NearToken::from_yoctonear(0);

    let account = worker.root_account()?;
    let sub = account.create_subaccount("subaccount").transact().await?;

    total_gas = total_gas
        .checked_add(
            sub.details
                .outcomes()
                .iter()
                .fold(NearToken::from_yoctonear(0), |acc, receipt| {
                    acc.checked_add(as_near(receipt.gas_burnt)).unwrap()
                }),
        )
        .unwrap();

    assert_eq!(total_gas, as_near(gas_meter.elapsed().unwrap()));

    Ok(())
}

#[test(tokio::test)]
async fn test_dropped_gas_meter() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    drop(gas_meter);

    worker.dev_create_account().await?;

    Ok(())
}

async fn analogous_to_dev_deploy(
    worker: &Worker<Sandbox>,
    total_gas: &mut NearToken,
) -> anyhow::Result<Contract> {
    let account = worker
        .root_account()?
        .create_subaccount("contract")
        .transact()
        .await?;

    *total_gas = total_gas
        .checked_add(
            account
                .details
                .outcomes()
                .iter()
                .fold(NearToken::from_yoctonear(0), |acc, receipt| {
                    acc.checked_add(as_near(receipt.gas_burnt)).unwrap()
                }),
        )
        .unwrap();

    Ok(account
        .into_result()?
        .deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?
        .into_result()?)
}

async fn analogous_to_dev_create_account(
    worker: &Worker<Sandbox>,
    total_gas: &mut NearToken,
) -> anyhow::Result<Account> {
    let account = worker
        .root_account()?
        .create_subaccount("alice")
        .transact()
        .await?;

    *total_gas = total_gas
        .checked_add(
            account
                .details
                .outcomes()
                .iter()
                .fold(NearToken::from_yoctonear(0), |acc, receipt| {
                    acc.checked_add(as_near(receipt.gas_burnt)).unwrap()
                }),
        )
        .unwrap();

    Ok(account.into_result()?)
}

const fn as_near(gas: Gas) -> NearToken {
    NearToken::from_yoctonear(gas.as_gas() as u128)
}
