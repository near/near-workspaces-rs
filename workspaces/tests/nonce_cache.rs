use serde_json::json;

const STATUS_MSG_CONTRACT: &[u8] = include_bytes!("../../examples/res/status_message.wasm");

#[tokio::test]
async fn test_nonce_caching_parallel() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    // Get the initial nonce
    let initial_nonce = worker
        .view_access_key(account.id(), &account.secret_key().public_key())
        .await?
        .nonce;

    // Prepare a large number of transactions to test caching
    const NUM_TRANSACTIONS: usize = 50;

    // Send transactions in parallel
    let parallel_tasks = (0..NUM_TRANSACTIONS).map(|i| {
        let msg = format!("msg{}", i);
        let account = account.clone();
        let contract_id = contract.id().clone();
        tokio::spawn(async move {
            account
                .call(&contract_id, "set_status")
                .args_json(json!({ "message": msg }))
                .transact_async()
                .await
        })
    });

    // Collect all transaction statuses
    let statuses = futures::future::join_all(parallel_tasks).await;

    // Wait for all transactions to complete
    for status in statuses {
        let status = status??;
        loop {
            match status.status().await? {
                std::task::Poll::Ready(outcome) => {
                    if outcome.is_success() {
                        break;
                    } else {
                        return Err(anyhow::anyhow!("Transaction failed: {:?}", outcome));
                    }
                }
                std::task::Poll::Pending => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    // Get the final nonce
    let final_nonce = worker
        .view_access_key(account.id(), &account.secret_key().public_key())
        .await?
        .nonce;

    // Verify that the nonce increased by exactly the number of transactions
    assert_eq!(
        final_nonce - initial_nonce,
        NUM_TRANSACTIONS as u64,
        "Nonce did not increment correctly"
    );

    // Verify the final message
    let final_msg = account
        .call(contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))
        .view()
        .await?
        .json::<String>()?;

    assert_eq!(
        final_msg,
        format!("msg{}", NUM_TRANSACTIONS - 1),
        "Final message does not match expected value"
    );

    Ok(())
}
