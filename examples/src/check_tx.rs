use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let (fn_name, fn_args) = ("set_status", json!({  "message": "hello_world"}));

    let _outcome = contract
        .call(fn_name)
        .args_json(fn_args.clone())
        .transact()
        .await?;

    let signed_tx = worker
        .signed_transaction(
            contract.id(),
            contract.signer(),
            fn_name.to_string(),
            fn_args,
        )
        .await?;

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let resp = worker.check_tx(signed_tx).await?;

    println!("RpcBroadcastTxSyncResponse {resp:?}");
    Ok(())
}
