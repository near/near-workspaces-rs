use serde::{Deserialize, Serialize};
use serde_json::json;

use std::path::Path;

use near_primitives::borsh::{self, BorshDeserialize, BorshSchema, BorshSerialize};
use near_primitives::types::AccountId;

const STATUS_MSG_WASM_FILEPATH: &str = "../examples/res/status_message.wasm";

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
struct Record {
    k: String,
    v: String,
}

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
struct StatusMessage {
    records: Vec<Record>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ViewStatus {
    status: String,
}

async fn view_status_state() -> (AccountId, StatusMessage) {
    let (contract_id, signer) = runner::dev_deploy(Path::new(STATUS_MSG_WASM_FILEPATH))
        .await
        .unwrap();

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

    let _outcome = runner::patch_state(contract_id.clone(), "STATE".to_string(), status_msg)
        .await
        .unwrap();

    // TODO: here because patch state takes longer than most requests. backoff should help this.
    std::thread::sleep(std::time::Duration::from_secs(5));

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
