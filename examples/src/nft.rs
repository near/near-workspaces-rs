use std::path::Path;

use runner::*;

const NFT_WASM_FILEPATH: &str = "./res/non_fungible_token.wasm";


#[runner::main(sandbox)]
async fn main() {
    let (contract_id, signer) = dev_deploy(Path::new(NFT_WASM_FILEPATH)).await.unwrap();

    // Wait a few seconds for create account to finalize:
    // TODO: exponentialBackoff to not need these explicit sleeps
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

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

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    let deposit = 10000000000000000000000;
    let outcome = call(
        &signer,
        contract_id.clone(),
        contract_id.clone(),
        "nft_mint".to_string(),
        format!(
            "{{
            \"token_id\": \"0\",
            \"token_owner_id\": \"{}\",
            \"token_metadata\": {{
                \"title\": \"Olympus Mons\",
                \"description\": \"Tallest mountain in charted solar system\",
                \"copies\": 1
            }}
        }}",
            contract_id
        )
        .into(),
        Some(deposit),
    )
    .await
    .unwrap();
    println!("nft_mint outcome: {:#?}", outcome);

    let call_result = view(
        contract_id.clone(),
        "nft_metadata".to_string(),
        b"".to_vec().into(),
    )
    .await
    .unwrap();

    println!(
        "--------------\n{}",
        serde_json::to_string_pretty(&call_result).unwrap()
    );

    println!("Dev Account ID: {}", contract_id);
}
