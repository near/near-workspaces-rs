#![recursion_limit = "256"]
use serde::{Deserialize, Serialize};
use test_log::test;

const NFT_WASM_FILEPATH: &str = "../examples/res/non_fungible_token.wasm";
const EXPECTED_NFT_METADATA: &str = r#"{
  "spec": "nft-1.0.0",
  "name": "Example NEAR non-fungible token",
  "symbol": "EXAMPLE",
  "icon": "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E",
  "base_uri": null,
  "reference": null,
  "reference_hash": null
}"#;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct NftMetadata {
    spec: String,
    name: String,
    symbol: String,
    icon: String,
    base_uri: Option<String>,
    reference: Option<String>,
    reference_hash: Option<String>,
}

fn expected() -> NftMetadata {
    serde_json::from_str(EXPECTED_NFT_METADATA).unwrap()
}

#[test(tokio::test)]
async fn test_dev_deploy() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    contract
        .call("new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": contract.id()
        }))
        .transact()
        .await?
        .into_result()?;

    let actual: NftMetadata = contract.view("nft_metadata", Vec::new()).await?.json()?;

    assert_eq!(actual, expected());

    Ok(())
}
