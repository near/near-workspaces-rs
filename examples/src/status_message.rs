use serde_json::json;
use std::path::Path;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[runner::main(sandbox)]
async fn main() {
    let (contract_id, signer) = runner::dev_deploy(Path::new(STATUS_MSG_WASM_FILEPATH))
        .await
        .unwrap();

    runner::call(
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

    let result = runner::view(
        contract_id.clone(),
        "get_status".into(),
        json!({
            "account_id": contract_id.clone().to_string(),
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
