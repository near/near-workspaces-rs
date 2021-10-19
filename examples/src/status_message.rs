use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[workspaces::main(sandbox)]
async fn main() {
    let (contract_id, signer) = workspaces::dev_deploy(STATUS_MSG_WASM_FILEPATH)
        .await
        .unwrap();

    workspaces::call(
        &signer,
        contract_id.clone(),
        contract_id.clone(),
        "set_status".into(),
        json!({
            "message": "hello_world",
        })
        .to_string()
        .into_bytes(),
        None,
    )
    .await
    .unwrap();

    let result = workspaces::view(
        contract_id.clone(),
        "get_status".into(),
        json!({
            "account_id": contract_id,
        })
        .to_string()
        .into_bytes()
        .into(),
    )
    .await
    .unwrap();

    println!(
        "status: {:?}",
        serde_json::to_string_pretty(&result).unwrap()
    );
}
