use near_jsonrpc_primitives::types::transactions::TransactionInfo;
use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call("set_status")
        .args_json(json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    println!("set_status: {:?}", outcome);

    let tx_info = {
        let outcome = outcome.outcome();
        TransactionInfo::TransactionId {
            hash: near_primitives::hash::CryptoHash(outcome.block_hash.0),
            account_id: outcome.executor_id.clone(),
        }
    };

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let resp = worker.tx_status(tx_info).await?;

    println!("FinalExecutionOutcomeWithReceiptView {resp:?}");
    Ok(())
}
