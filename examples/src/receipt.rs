use near_jsonrpc_primitives::types::receipts::ReceiptReference;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.root_account()?.deploy(&wasm).await?.into_result()?;

    let outcome = contract
        .call("set_status")
        .args_json(serde_json::json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    let receipt_ref = {
        let mut ids = outcome.outcome().receipt_ids.clone();
        if ids.is_empty() {
            println!("no receipt ids present");
            return Ok(());
        }

        println!("receipts found: {ids:?}");

        ReceiptReference {
            receipt_id: near_primitives::hash::CryptoHash(
                ids.pop().expect("expected at least one receipt id").0,
            ),
        }
    };

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let resp = worker.receipt(receipt_ref).await?;

    println!("ReceiptView: {resp:?}");
    Ok(())
}
