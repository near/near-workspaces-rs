use serde_json::json;

use workspaces::prelude::*;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await.unwrap();

    let outcome = worker
        .call(
            &contract,
            "new_default_meta".to_string(),
            json!({
                "owner_id": contract.id(),
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await?;

    println!("new_default_meta outcome: {:#?}", outcome);

    let deposit = 10000000000000000000000;
    let outcome = worker
        .call(
            &contract,
            "nft_mint".to_string(),
            json!({
                "token_id": "0",
                "token_owner_id": contract.id(),
                "token_metadata": {
                    "title": "Olympus Mons",
                    "dscription": "Tallest mountain in charted solar system",
                    "copies": 1,
                },
            })
            .to_string()
            .into_bytes(),
            Some(deposit),
        )
        .await?;

    println!("nft_mint outcome: {:#?}", outcome);

    let result: serde_json::Value = worker
        .view(
            contract.id().clone(),
            "nft_metadata".to_string(),
            Vec::new(),
        )
        .await?
        .try_serde_deser()?;

    println!("--------------\n{}", result);

    println!("Dev Account ID: {}", contract.id());

    Ok(())
}
