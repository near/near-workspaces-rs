use std::collections::HashMap;
use std::convert::TryInto;
use std::path::PathBuf;

use chrono::Utc;
use rand::Rng;
use url::Url;

use near_crypto::PublicKey;
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::hash::CryptoHash;
use near_primitives::types::{AccountId, BlockHeight, Finality};
use near_primitives::views::{AccessKeyView, QueryRequest, StateItem};

use crate::rpc::client;
use crate::runtime::context::MISSING_RUNTIME_ERROR;

pub(crate) async fn access_key(
    account_id: AccountId,
    pk: PublicKey,
) -> anyhow::Result<(AccessKeyView, BlockHeight, CryptoHash)> {
    let query_resp = client::new()
        .call(&methods::query::RpcQueryRequest {
            block_reference: Finality::Final.into(),
            request: QueryRequest::ViewAccessKey {
                account_id,
                public_key: pk,
            },
        })
        .await?;

    match query_resp.kind {
        QueryResponseKind::AccessKey(access_key) => {
            Ok((access_key, query_resp.block_height, query_resp.block_hash))
        }
        _ => Err(anyhow::anyhow!("Could not retrieve access key")),
    }
}

pub(crate) fn credentials_filepath(account_id: AccountId) -> anyhow::Result<PathBuf> {
    let mut path = crate::runtime::context::current()
        .expect(MISSING_RUNTIME_ERROR)
        .keystore_path()?;

    // Create this path's directories if they don't exist:
    std::fs::create_dir_all(path.clone())?;

    path.push(format!("{}.json", account_id));
    Ok(path)
}

/// Convert `StateItem`s over to a Map<data_key, value_bytes> representation.
/// Assumes key and value are base64 encoded, so this also decodes them.
pub(crate) fn into_state_map(
    state_items: &[StateItem],
) -> anyhow::Result<HashMap<String, Vec<u8>>> {
    let decode = |s: &StateItem| {
        Ok((
            String::from_utf8(base64::decode(&s.key)?)?,
            base64::decode(&s.value)?,
        ))
    };

    state_items.iter().map(decode).collect()
}

pub(crate) fn random_account_id() -> AccountId {
    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(10000000000000usize..99999999999999);
    let account_id = format!("dev-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), random_num);
    let account_id: AccountId = account_id
        .try_into()
        .expect("could not convert dev account into AccountId");

    account_id
}

pub(crate) async fn url_create_account(
    helper_url: Url,
    account_id: AccountId,
    pk: PublicKey,
) -> anyhow::Result<()> {
    let helper_addr = helper_url.join("account")?;

    // TODO(maybe): need this in near-jsonrpc-client as well:
    let _resp = reqwest::Client::new()
        .post(helper_addr)
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&serde_json::json!({
            "newAccountId": account_id.to_string(),
            "newAccountPublicKey": pk.to_string(),
        }))?)
        .send()
        .await?;

    Ok(())
}
