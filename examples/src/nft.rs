use serde_json::json;

use workspaces::prelude::*;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call(&worker, "new_default_meta")
        .args_json(json!({
                "owner_id": contract.id(),
        }))?
        .transact()
        .await?;

    println!("new_default_meta outcome: {:#?}", outcome);

    let deposit = 10000000000000000000000;
    let outcome = contract
        .call(&worker, "nft_mint")
        .args_json(json!({
            "token_id": "0",
            "token_owner_id": contract.id(),
            "token_metadata": {
                "title": "Olympus Mons",
                "dscription": "Tallest mountain in charted solar system",
                "copies": 1,
            },
        }))?
        .deposit(deposit)
        .transact()
        .await?;

    println!("nft_mint outcome: {:#?}", outcome);

    let result: serde_json::Value = worker
        .view(contract.id(), "nft_metadata", Vec::new())
        .await?
        .json()?;

    println!("--------------\n{}", result);

    println!("Dev Account ID: {}", contract.id());

    Ok(())
}
