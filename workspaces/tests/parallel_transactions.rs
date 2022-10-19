use std::collections::VecDeque;

use serde_json::json;

const STATUS_MSG_CONTRACT: &[u8] = include_bytes!("../../examples/res/status_message.wasm");

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_parallel() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    let parallel_tasks = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
        .iter()
        .map(|msg| {
            let id = contract.id().clone();
            let account = account.clone();
            tokio::spawn(async move {
                account
                    .call(&id, "set_status")
                    .args_json(json!({
                        "message": msg.to_string(),
                    }))
                    .transact()
                    .await?
                    .into_result()?;
                anyhow::Result::<()>::Ok(())
            })
        });
    futures::future::join_all(parallel_tasks).await;

    // Check the final set message. This should be random each time this test function is called:
    let final_set_msg = account
        .call(contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))
        .view()
        .await?
        .json::<String>()?;
    println!("Final set message: {:?}", final_set_msg);

    Ok(())
}

#[tokio::test]
async fn test_parallel_async() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    // Create a queue statuses we can check the status of later.
    let mut statuses = VecDeque::new();
    for msg in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"] {
        let status = account
            .call(contract.id(), "set_status")
            .args_json(json!({
                "message": msg,
            }))
            .transact_async()
            .await?;
        statuses.push_back(status);
    }

    // Retry checking the statuses of all transactions until the queue is empty
    // with all transactions completed.
    while let Some(status) = statuses.pop_front() {
        if let Err(_err) = status.status().await {
            statuses.push_back(status);
        }
    }

    // Check the final set message. This should be "j" due to the ordering of the queue.
    let final_set_msg = account
        .call(contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))
        .view()
        .await?
        .json::<String>()?;
    assert_eq!(final_set_msg, "j");

    Ok(())
}
