use std::convert::TryInto;
use workspaces::{prelude::*, AccountId};

const TESTNET_PREDEPLOYED_CONTRACT_ID: &str = "dev-20211013002148-59466083160385";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let testnet_worker = workspaces::testnet();
    let worker = workspaces::sandbox();

    let contract_id: AccountId = TESTNET_PREDEPLOYED_CONTRACT_ID
        .to_string()
        .try_into()
        .unwrap();

    let status_msg = worker.create_contract_from(contract_id, testnet_worker).await?;

    println!("ACCOUNT: {:?}", status_msg.id());

    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let result = worker
        .view(
            status_msg.id().clone(),
            "get_status".into(),
            serde_json::json!({
                "account_id": TESTNET_PREDEPLOYED_CONTRACT_ID,
            })
            .to_string()
            .into_bytes(),
        )
        .await?;


    let status: String = serde_json::from_value(result)?;
    println!("New status patched: {:?}", status);

    Ok(())
}
