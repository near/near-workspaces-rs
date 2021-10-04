use serde_json::json;

use runner::*;

const NFT_WASM_FILEPATH: &str = "./examples/res/non_fungible_token.wasm";

#[runner::main(sandbox)]
async fn main() {
    let (contract_id, signer) = dev_deploy(NFT_WASM_FILEPATH).await.unwrap();

    let outcome = call(
        &signer,
        contract_id.clone(),
        contract_id.clone(),
        "new_default_meta".to_string(),
        format!("{{\"owner_id\": \"{}\"}}", contract_id).into(),
        None,
    )
    .await
    .unwrap();
    println!("new_default_meta outcome: {:#?}", outcome);

    let deposit = 10000000000000000000000;
    let outcome = call(
        &signer,
        contract_id.clone(),
        contract_id.clone(),
        "nft_mint".to_string(),
        json!({
            "token_id": "0",
            "token_owner_id": contract_id,
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

    let call_result = view(
        contract_id.clone(),
        "nft_metadata".to_string(),
        Vec::new().into(),
    )
    .await
    .unwrap();

    println!(
        "--------------\n{}",
        serde_json::to_string_pretty(&call_result).unwrap()
    );

    println!("Dev Account ID: {}", contract_id);
}
