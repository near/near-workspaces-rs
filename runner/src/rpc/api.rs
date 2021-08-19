use super::types::{AccountInfo, NearBalance};
use super::tool;

use chrono::Utc;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use near_crypto::{InMemorySigner, KeyType, PublicKey, Signer};
use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryRequest};
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::{AccountId, Balance, Finality, FunctionArgs, Gas};
use near_primitives::views::{FinalExecutionOutcomeView, QueryRequest};
use nearcore::NEAR_BASE;

const DEV_ACCOUNT_SEED: &str = "testificate";
const DEFAULT_CALL_FN_GAS: Gas = 10000000000000;

pub async fn display_account_info(account_id: String) -> Result<AccountInfo, String> {
    let query_resp = tool::sandbox_client()
        .query(RpcQueryRequest {
            block_reference: Finality::Final.into(),
            request: QueryRequest::ViewAccount {
                account_id: account_id.clone(),
            },
        })
        .await
        .map_err(|err| err.to_string())?;

    let account_view = match query_resp.kind {
        QueryResponseKind::ViewAccount(result) => result,
        _ => return Err("Error call result".to_owned()),
    };

    Ok(AccountInfo {
        account_id,
        block_height: query_resp.block_height,
        block_hash: query_resp.block_hash,
        balance: NearBalance::from_yoctonear(account_view.amount),
        stake: NearBalance::from_yoctonear(account_view.locked),
        used_storage_bytes: account_view.storage_usage,
    })
}

pub async fn transfer_near(
    signer: &dyn Signer,
    signer_id: AccountId,
    receiver_id: AccountId,
    amount_yocto: Balance,
) -> Result<FinalExecutionOutcomeView, String> {
    let (access_key, _, block_hash) =
        tool::access_key(signer_id.clone(), signer.public_key()).await?;

    let tx = SignedTransaction::send_money(
        access_key.nonce + 1,
        signer_id,
        receiver_id,
        signer,
        amount_yocto,
        block_hash,
    );

    let transaction_info = tool::send_tx(tx).await?;
    Ok(transaction_info)
}

pub async fn call(
    signer: &dyn Signer,
    signer_id: AccountId,
    contract_id: AccountId,
    method_name: String,
    args: Vec<u8>,
    deposit: Option<Balance>,
) -> Result<FinalExecutionOutcomeView, String> {
    let (access_key, _, block_hash) =
        tool::access_key(signer_id.clone(), signer.public_key()).await?;
    let tx = SignedTransaction::call(
        access_key.nonce + 1,
        signer_id,
        contract_id,
        signer,
        deposit.unwrap_or(0),
        method_name,
        args,
        DEFAULT_CALL_FN_GAS,
        block_hash,
    );
    let transaction_info = tool::send_tx(tx).await?;
    Ok(transaction_info)
}

pub async fn view(
    contract_id: AccountId,
    method_name: String,
    args: FunctionArgs,
) -> Result<serde_json::Value, String> {
    let query_resp = tool::sandbox_client()
        .query(RpcQueryRequest {
            block_reference: Finality::Final.into(),
            request: QueryRequest::CallFunction {
                account_id: contract_id,
                method_name,
                args,
            },
        })
        .await
        .map_err(|err| format!("Failed to fetch query for view method: {:?}", err))?;

    let call_result = match query_resp.kind {
        QueryResponseKind::CallResult(result) => result.result,
        _ => return Err("Error call result".to_string()),
    };

    let call_result_str = String::from_utf8(call_result).map_err(|e| e.to_string())?;
    let serde_call_result: serde_json::Value = serde_json::from_str(&call_result_str)
        .map_err(|err| format!("serde_json error: {:?}", err))?;

    Ok(serde_call_result)
}

pub async fn create_account(
    signer: &dyn Signer,
    signer_id: AccountId,
    new_account_id: AccountId,
    new_account_pk: PublicKey,
    deposit: Option<Balance>,
) -> Result<FinalExecutionOutcomeView, String> {
    let (access_key, _, block_hash) =
        tool::access_key(signer_id.clone(), signer.public_key()).await?;

    let signed_tx = SignedTransaction::create_account(
        access_key.nonce + 1,
        signer_id,
        new_account_id,
        deposit.unwrap_or(NEAR_BASE),
        new_account_pk,
        signer,
        block_hash,
    );
    let transaction_info = tool::send_tx(signed_tx).await?;
    Ok(transaction_info)
}

pub async fn create_tla_account(
    new_account_id: AccountId,
    new_account_pk: PublicKey,
) -> Result<FinalExecutionOutcomeView, String> {
    let root_signer = tool::root_account();
    create_account(
        &root_signer,
        root_signer.account_id.clone(),
        new_account_id,
        new_account_pk,
        None,
    )
    .await
}

async fn create_account_and_deploy(
    new_account_id: AccountId,
    new_account_pk: PublicKey,
    code_filepath: &Path,
) -> Result<FinalExecutionOutcomeView, String> {
    let root_signer = tool::root_account();
    let (access_key, _, block_hash) =
        tool::access_key(root_signer.account_id.clone(), root_signer.public_key()).await?;

    let mut code = Vec::new();
    File::open(code_filepath)
        .map_err(|e| e.to_string())?
        .read_to_end(&mut code)
        .map_err(|e| e.to_string())?;

    // This transaction creates the account too:
    let signed_tx = SignedTransaction::create_contract(
        access_key.nonce + 1,
        root_signer.account_id.clone(),
        new_account_id.to_string(),
        code,
        100 * NEAR_BASE,
        new_account_pk,
        &root_signer,
        block_hash,
    );
    dbg!(&signed_tx);

    let transaction_info = tool::send_tx(signed_tx).await?;
    Ok(transaction_info)
}

pub async fn delete_account(
    account_id: AccountId,
    signer: &dyn Signer,
    beneficiary_id: AccountId,
) -> Result<FinalExecutionOutcomeView, String> {
    let (access_key, _, block_hash) =
        tool::access_key(account_id.clone(), signer.public_key()).await?;

    let signed_tx = SignedTransaction::delete_account(
        access_key.nonce + 1,
        account_id.clone(),
        account_id,
        beneficiary_id,
        signer,
        block_hash,
    );
    let transaction_info = tool::send_tx(signed_tx).await?;
    Ok(transaction_info)
}

fn dev_generate() -> (AccountId, InMemorySigner) {
    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(10000000000000usize..99999999999999);
    let account_id = format!("dev-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), random_num);

    let signer = InMemorySigner::from_seed(&account_id, KeyType::ED25519, DEV_ACCOUNT_SEED);
    signer.write_to_file(&tool::credentials_filepath(account_id.clone()).unwrap());
    (account_id, signer)
}

pub async fn dev_create() -> Result<(AccountId, InMemorySigner), String> {
    let (account_id, signer) = dev_generate();
    let outcome = create_tla_account(account_id.clone(), signer.public_key()).await?;
    dbg!(outcome);
    Ok((account_id, signer))
}

pub async fn dev_deploy(contract_file: &Path) -> Result<(AccountId, InMemorySigner), String> {
    let (account_id, signer) = dev_generate();
    let outcome =
        create_account_and_deploy(account_id.clone(), signer.public_key(), contract_file).await?;
    dbg!(outcome);
    Ok((account_id, signer))
}
const NFT_WASM_FILEPATH: &'static str = "/home/tensor/space/sandbox-api/res/non_fungible_token.wasm";

pub async fn run_() {
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

pub fn run() {
    use tokio::runtime::Runtime;
    let mut rt = Runtime::new().unwrap();
    let local = tokio::task::LocalSet::new();
    local.block_on(&mut rt, async {
        run_().await;
    });
}
