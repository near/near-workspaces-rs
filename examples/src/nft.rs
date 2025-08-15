use near_workspaces::types::NearToken;
use serde_json::json;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call("new_default_meta")
        .args_json(json!({
                "owner_id": contract.id(),
        }))
        .transact()
        .await?;

    println!("new_default_meta outcome: {outcome:#?}");

    let deposit = NearToken::from_yoctonear(10000000000000000000000);
    let outcome = contract
        .call("nft_mint")
        .args_json(json!({
            "token_id": "0",
            "token_owner_id": contract.id(),
            "token_metadata": {
                "title": "Olympus Mons",
                "description": "Tallest mountain in charted solar system",
                "copies": 1,
            },
        }))
        .deposit(deposit)
        .transact()
        .await?;

    println!("nft_mint outcome: {outcome:#?}");

    let result: serde_json::Value = worker.view(contract.id(), "nft_metadata").await?.json()?;

    println!("--------------\n{result}");
    println!("Dev Account ID: {}", contract.id());

    Ok(())
}
