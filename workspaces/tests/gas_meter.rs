use serde_json::json;
use test_log::test;
use workspaces::operations::Function;
use workspaces::types::GasMeter;

#[test(tokio::test)]
async fn test_gas_meter_with_single_transaction() -> anyhow::Result<()> {
    let mut worker = workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);

    let status_msg = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;
    let account = worker.dev_create_account().await?;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    assert_eq!(txn.total_gas_burnt, gas_meter.elapsed().unwrap());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_multiple_transactions() -> anyhow::Result<()> {
    let mut worker = workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);

    let mut total_gas = 0;

    let status_msg = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;
    let account = worker.dev_create_account().await?;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    assert_eq!(txn.total_gas_burnt, gas_meter.elapsed().unwrap());

    total_gas += txn.total_gas_burnt;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    total_gas += txn.total_gas_burnt;

    assert_eq!(total_gas, gas_meter.elapsed().unwrap());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_parallel_transactions() -> anyhow::Result<()> {
    let mut worker = workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);

    let mut total_gas = 0;

    let status_msg = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;
    let account = worker.dev_create_account().await?;

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
        total_gas += task.await??;
    }

    assert_eq!(total_gas, gas_meter.elapsed().unwrap());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_multiple_transactions_and_view() -> anyhow::Result<()> {
    let mut worker = workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);

    let mut total_gas = 0;

    let status_msg = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;
    let account = worker.dev_create_account().await?;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    assert_eq!(txn.total_gas_burnt, gas_meter.elapsed().unwrap());

    total_gas += txn.total_gas_burnt;

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;

    total_gas += txn.total_gas_burnt;

    assert_eq!(total_gas, gas_meter.elapsed().unwrap());

    let _ = account
        .call(status_msg.id(), "get_status")
        .args_json(serde_json::json!({
            "account_id": account.id(),
        }))
        .view()
        .await?;

    assert_eq!(total_gas, gas_meter.elapsed().unwrap());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_batch_tx() -> anyhow::Result<()> {
    let mut worker = workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);

    let mut total_gas = 0;

    let contract = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;

    let txn = contract
        .batch()
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

    assert_eq!(txn.total_gas_burnt, gas_meter.elapsed().unwrap());

    total_gas += txn.total_gas_burnt;

    let txn = contract
        .batch()
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

    total_gas += txn.total_gas_burnt;

    assert_eq!(total_gas, gas_meter.elapsed().unwrap());

    Ok(())
}
