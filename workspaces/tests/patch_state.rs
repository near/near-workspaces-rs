use borsh::{self, BorshDeserialize, BorshSerialize};
use serde_json::json;

use workspaces::prelude::*;
use workspaces::{AccountId, DevNetwork, Worker};

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

async fn view_status_state(
    worker: Worker<impl DevNetwork>,
) -> anyhow::Result<(AccountId, StatusMessage)> {
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await.unwrap();

    worker
        .call(
            &contract,
            "set_status".into(),
            json!({
                "message": "hello",
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await?;

    let mut state_items = worker.view_state(contract.id().clone(), None).await?;
    let state = state_items
        .remove("STATE")
        .ok_or_else(|| anyhow::anyhow!("Could not retrieve STATE"))?;
    let status_msg: StatusMessage = StatusMessage::try_from_slice(&state)?;

    Ok((contract.id().clone(), status_msg))
}

#[tokio::test]
async fn test_view_state() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let (contract_id, status_msg) = view_status_state(worker).await?;

    assert_eq!(
        status_msg,
        StatusMessage {
            records: vec![Record {
                k: contract_id.to_string(),
                v: "hello".to_string(),
            }]
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_patch_state() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let (contract_id, mut status_msg) = view_status_state(worker.clone()).await?;
    status_msg.records.push(Record {
        k: "alice.near".to_string(),
        v: "hello world".to_string(),
    });

    worker
        .patch_state(
            contract_id.clone(),
            "STATE".to_string(),
            status_msg.try_to_vec()?,
        )
        .await?;

    let result = worker
        .view(
            contract_id.clone(),
            "get_status".into(),
            json!({
                "account_id": "alice.near",
            })
            .to_string()
            .into_bytes(),
        )
        .await?;

    let status: String = serde_json::from_str(&result)?;
    assert_eq!(status, "hello world".to_string());

    Ok(())
}
