use serde_json::json;

use workspaces::prelude::*;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[tokio::main]
async fn main() {
    let worker = workspaces::sandbox();
    let contract = worker.dev_deploy(NFT_WASM_FILEPATH).await.unwrap();

    let outcome = worker
        .call(
            &contract,
            "new_default_meta".to_string(),
            format!("{{\"owner_id\": \"{}\"}}", contract.id()).into(),
            None,
        )
        .await
        .unwrap();
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
        .await
        .unwrap();
    println!("nft_mint outcome: {:#?}", outcome);

    let result = worker
        .view(contract.id(), "nft_metadata".to_string(), Vec::new().into())
        .await
        .unwrap();

    println!(
        "--------------\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    println!("Dev Account ID: {}", contract.id());
}
