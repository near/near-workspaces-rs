use std::path::PathBuf;

use near_crypto::{InMemorySigner, PublicKey};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryRequest};
use near_primitives::borsh::BorshSerialize;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::{AccountId, BlockHeight, Finality};
use near_primitives::views::{AccessKeyView, FinalExecutionOutcomeView, QueryRequest};

const SANDBOX_CREDENTIALS_DIR: &str = ".near-credentials/sandbox/";
const MISSING_RUNTIME_ERROR: &str =
    "there is no runtime running: need to be ran from a near runtime context";

fn rt_current_addr() -> String {
    crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .rpc_addr()
}

pub(crate) fn json_client() -> JsonRpcClient {
    near_jsonrpc_client::new_client(&rt_current_addr())
}

pub(crate) fn root_account() -> InMemorySigner {
    let mut path = crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .home_dir();
    path.push("validator_key.json");

    let root_signer = InMemorySigner::from_file(&path);
    root_signer
}

pub(crate) async fn access_key(
    account_id: String,
    pk: PublicKey,
) -> Result<(AccessKeyView, BlockHeight, CryptoHash), String> {
    let query_resp = json_client()
        .query(RpcQueryRequest {
            block_reference: Finality::Final.into(),
            request: QueryRequest::ViewAccessKey {
                account_id,
                public_key: pk,
            },
        })
        .await
        .map_err(|err| format!("Failed to fetch public key info: {:?}", err))?;

    match query_resp.kind {
        QueryResponseKind::AccessKey(access_key) => {
            Ok((access_key, query_resp.block_height, query_resp.block_hash))
        }
        _ => Err("Could not retrieve access key".to_owned()),
    }
}

pub(crate) async fn send_tx(tx: SignedTransaction) -> Result<FinalExecutionOutcomeView, String> {
    let json_rpc_client = json_client();
    let transaction_info_result = loop {
        let transaction_info_result = json_rpc_client
            .broadcast_tx_commit(near_primitives::serialize::to_base64(
                tx.try_to_vec()
                    .expect("Transaction is not expected to fail on serialization"),
            ))
            .await;

        if let Err(ref err) = transaction_info_result {
            if let Some(serde_json::Value::String(data)) = &err.data {
                if data.contains("Timeout") {
                    println!("Error transaction: {:?}", err);
                    continue;
                }
            }
        }

        break transaction_info_result;
    };

    transaction_info_result.map_err(|e| format!("Error transaction: {:?}", e))
}

pub(crate) fn credentials_filepath(account_id: AccountId) -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Could not get HOME_DIR".to_string())?;
    let mut path = PathBuf::from(&home_dir);
    path.push(SANDBOX_CREDENTIALS_DIR);

    // Create this path's directories if they don't exist:
    std::fs::create_dir_all(path.clone())
        .map_err(|e| format!("Could not create near credential directory: {}", e))?;

    path.push(format!("{}.json", account_id));
    Ok(path)
}
