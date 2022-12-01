#![cfg(feature = "unstable")]

use near_primitives::views::FinalExecutionOutcomeView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TxWaitType {
    /// Waits until the transaction result value is executed and finalized.
    /// Note: this does not wait on all execution results, only the tx return value.
    ExecutionResult,
    /// Waits until everything has executed and is final, including refund receipts.
    Full,
}

async fn send_wait_req(
    client: reqwest::Client,
    rpc_addr: String,
    params: impl serde::Serialize,
) -> anyhow::Result<FinalExecutionOutcomeView> {
    let req = near_jsonrpc_primitives::message::Message::request(
        "EXPERIMENTAL_tx_wait".to_string(),
        Some(serde_json::to_value(params)?),
    );

    let payload = serde_json::to_vec(&req).unwrap();

    let response = client
        .post(rpc_addr)
        // .headers(headers.clone())
        .body(payload)
        .send()
        .await?;

    assert!(matches!(response.status(), reqwest::StatusCode::OK));

    let res = serde_json::from_slice(&response.bytes().await?);
    let response_message = near_jsonrpc_primitives::message::decoded_to_parsed(res).unwrap();
    let near_jsonrpc_primitives::message::Message::Response(response) = response_message else {
		return Err(anyhow::anyhow!("Expected response"));
	};
    Ok(serde_json::from_value(response.result.unwrap())?)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_async_wait_finality() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("./tests/test-contracts/promise-chain").await?;
    let contract = worker.dev_deploy(&wasm).await?;
    let account = worker.dev_create_account().await?;

    let now = std::time::Instant::now();
    let result = account
        .call(contract.id(), "a")
        .max_gas()
        .transact_async()
        .await?;

    let sender_id = result.sender_id();
    let tx_hash = near_primitives::hash::CryptoHash(result.hash().0);

    let rpc_addr = format!("http://localhost:{}", worker.rpc_port());

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers.clone())
        .build()?;

    let req_fut = send_wait_req(
        client.clone(),
        rpc_addr.clone(),
        (tx_hash, sender_id.clone(), "final", "full"),
    );
    let ff_handle = tokio::task::spawn(async move {
        let res = req_fut.await.unwrap();
        println!("final full: {}", now.elapsed().as_millis());
        res
    });

    let req_fut = send_wait_req(
        client.clone(),
        rpc_addr.clone(),
        (tx_hash, sender_id.clone(), "near-final", "execution_result"),
    );
    let dsf_handle = tokio::task::spawn(async move {
        let res = req_fut.await.unwrap();
        println!("ds execution: {}", now.elapsed().as_millis());
        res
    });

    let req_fut = send_wait_req(
        client.clone(),
        rpc_addr.clone(),
        (tx_hash, sender_id.clone(), "final", "execution_result"),
    );
    let ef_handle = tokio::task::spawn(async move {
        let res = req_fut.await.unwrap();
        println!("final execution: {}", now.elapsed().as_millis());
        println!(
            "ef_res: {:?}",
            res.receipts_outcome
                .iter()
                .map(|r| r.outcome.status.clone())
                .collect::<Vec<_>>()
        );
        res
    });

    let req_fut = send_wait_req(
        client.clone(),
        rpc_addr.clone(),
        (tx_hash, sender_id.clone(), "optimistic", "execution_result"),
    );
    let oe_handle = tokio::task::spawn(async move {
        let res = req_fut.await.unwrap();
        println!("optimistic execution: {}", now.elapsed().as_millis());
        res
    });

    println!("PRE WAIT: {}", now.elapsed().as_millis());
    // FIXME finality doesn't have an effect on the times currently. Everything is full finality
    futures::future::join_all([oe_handle, ef_handle, ff_handle, dsf_handle]).await;

    Ok(())
}
