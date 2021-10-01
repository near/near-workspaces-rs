// TODO: Remove this when near-jsonrpc-client crate no longer defaults to deprecation for
//       warnings about unstable API.
#![allow(deprecated)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::str;

use near_crypto::{InMemorySigner, PublicKey};
use near_jsonrpc_client::{methods, JsonRpcClient, errors::{JsonRpcError, JsonRpcServerError}};
use near_jsonrpc_primitives::types::{transactions::RpcTransactionError, query::QueryResponseKind};
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::SignedTransaction;
use near_primitives::types::{AccountId, BlockHeight, Finality};
use near_primitives::views::{AccessKeyView, FinalExecutionOutcomeView, QueryRequest, StateItem};

use crate::runtime::context::MISSING_RUNTIME_ERROR;

const SANDBOX_CREDENTIALS_DIR: &str = ".near-credentials/sandbox/";

fn rt_current_addr() -> String {
    crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .rpc_addr()
}

pub(crate) fn json_client() -> JsonRpcClient {
    JsonRpcClient::connect(&rt_current_addr())
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
    account_id: AccountId,
    pk: PublicKey,
) -> Result<(AccessKeyView, BlockHeight, CryptoHash), String> {
    let query_resp = json_client().call(&methods::query::RpcQueryRequest {
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
    let client = json_client();
    let transaction_info_result = loop {
        let transaction_info_result = client.clone()
            .call(&methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: tx.clone()
            })
            .await;

        if let Err(ref err) = transaction_info_result {
            if matches!(err, JsonRpcError::ServerError(JsonRpcServerError::HandlerError(RpcTransactionError::TimeoutError))) {
                eprintln!("transaction timeout: {:?}", err);
                continue;
            }
        }

        break transaction_info_result;
    };

    // TODO: remove this after adding exponential backoff
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

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

/// Convert `StateItem`s over to a Map<data_key, value_bytes> representation.
/// Assumes key and value are base64 encoded, so this also decode them.
pub(crate) fn into_state_map(state_items: Vec<StateItem>) -> HashMap<String, Vec<u8>> {
    let decode = |s: StateItem| {
        (str::from_utf8(&base64::decode(s.key.clone()).unwrap())
            .unwrap_or_else(|_| &s.key).to_owned(),
            base64::decode(s.value).unwrap()
        )
    };

    state_items.into_iter()
        .map(decode)
        .collect()
}
