use std::collections::HashMap;
use std::convert::TryInto;

use chrono::Utc;
use rand::Rng;
use url::Url;

use near_crypto::PublicKey;
use near_primitives::types::AccountId;
use near_primitives::views::StateItem;

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
