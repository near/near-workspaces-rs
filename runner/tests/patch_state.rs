use serde_json::json;

use near_primitives::borsh::{self, BorshDeserialize, BorshSerialize};
use near_primitives::types::AccountId;

const STATUS_MSG_WASM_FILEPATH: &str = "../examples/res/status_message.wasm";

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
struct Record {
    k: String,
    v: String,
}

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
struct StatusMessage {
    records: Vec<Record>,
}

async fn view_status_state() -> (AccountId, StatusMessage) {
    let (contract_id, signer) = runner::dev_deploy(STATUS_MSG_WASM_FILEPATH).await.unwrap();

    runner::call(
        &signer,
        contract_id.clone(),
        contract_id.clone(),
        "set_status".into(),
        json!({
            "message": "hello",
        })
        .to_string()
        .into_bytes(),
        None,
    )
    .await
    .unwrap();

    let mut state_items = runner::view_state(contract_id.clone(), None).await.unwrap();
    let state = state_items.remove("STATE").unwrap();
    let status_msg: StatusMessage =
        StatusMessage::try_from_slice(&state).expect("Expected to retrieve state");

    (contract_id, status_msg)
}

#[runner::test(sandbox)]
async fn test_view_state() {
    let (contract_id, status_msg) = view_status_state().await;
    assert_eq!(
        status_msg,
        StatusMessage {
            records: vec![Record {
                k: contract_id.to_string(),
                v: "hello".to_string(),
            }]
        }
    );
}

#[runner::test(sandbox)]
async fn test_patch_state() {
    let (contract_id, mut status_msg) = view_status_state().await;
    status_msg.records.push(Record {
        k: "alice.near".to_string(),
        v: "hello world".to_string(),
    });

    let _outcome = runner::patch_state(contract_id.clone(), "STATE".to_string(), &status_msg)
        .await
        .unwrap();

    let result = runner::view(
        contract_id.clone(),
        "get_status".into(),
        json!({
            "account_id": "alice.near",
        })
        .to_string()
        .into_bytes()
        .into(),
    )
    .await
    .unwrap();

    let status: String = serde_json::from_value(result).unwrap();
    assert_eq!(status, "hello world".to_string());
}
