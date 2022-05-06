// Required since `test_log` adds more recursion than the standard recursion limit of 128
#![recursion_limit = "256"]

use borsh::{self, BorshDeserialize, BorshSerialize};
use serde_json::json;
use test_log::test;
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
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<(AccountId, StatusMessage)> {
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await.unwrap();

    contract
        .call(worker, "set_status")
        .args_json(json!({
                "message": "hello",
        }))?
        .transact()
        .await?;

    let mut state_items = contract.view_state(worker, None).await?;
    let state = state_items
        .remove(b"STATE".as_slice())
        .ok_or_else(|| anyhow::anyhow!("Could not retrieve STATE"))?;
    let status_msg: StatusMessage = StatusMessage::try_from_slice(&state)?;

    Ok((contract.id().clone(), status_msg))
}

#[test(tokio::test)]
async fn test_view_state() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (contract_id, status_msg) = view_status_state(&worker).await?;

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

#[test(tokio::test)]
async fn test_patch_state() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (contract_id, mut status_msg) = view_status_state(&worker).await?;
    status_msg.records.push(Record {
        k: "alice.near".to_string(),
        v: "hello world".to_string(),
    });

    worker
        .patch_state(&contract_id, b"STATE", &status_msg.try_to_vec()?)
        .await?;

    let status: String = worker
        .view(
            &contract_id,
            "get_status",
            json!({
                "account_id": "alice.near",
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert_eq!(status, "hello world".to_string());

    Ok(())
}

#[test(tokio::test)]
#[ignore]
async fn patch_state_builder() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let id: AccountId = "nino.near".parse()?;
    worker
        .patch_account(&id)
        .amount(1)
        .locked(0)
        .apply()
        .await?;

    Ok(())
}
