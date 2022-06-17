use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use rand::Rng;
use url::Url;

use near_crypto::SecretKey;
use near_primitives::views::StateItem;

use crate::error::{RpcErrorKind, SerializationError};
use crate::types::{AccountId, PublicKey};

/// Convert `StateItem`s over to a Map<data_key, value_bytes> representation.
/// Assumes key and value are base64 encoded, so this also decodes them.
pub(crate) fn into_state_map(
    state_items: &[StateItem],
) -> Result<HashMap<Vec<u8>, Vec<u8>>, SerializationError> {
    let decode = |s: &StateItem| Ok((base64::decode(&s.key)?, base64::decode(&s.value)?));

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
) -> crate::result::Result<()> {
    let helper_url = helper_url.join("account").unwrap();

    // TODO(maybe): need this in near-jsonrpc-client as well:
    let _resp = reqwest::Client::new()
        .post(helper_url)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_vec(&serde_json::json!({
                "newAccountId": account_id,
                "newAccountPublicKey": pk,
            }))
            .map_err(SerializationError::SerdeError)?,
        )
        .send()
        .await
        .map_err(|e| RpcErrorKind::HelperAccountCreationFailure.with_repr(Box::new(e)))?;

    Ok(())
}

pub(crate) fn write_cred_to_file(path: &Path, id: &AccountId, sk: &SecretKey) {
    let mut file = File::create(path).expect("Failed to create / write a key file.");

    #[cfg(unix)]
    {
        use std::os::unix::prelude::PermissionsExt;
        let mut perm = file
            .metadata()
            .expect("Failed to retrieve key file metadata.")
            .permissions();

        #[cfg(target_os = "macos")]
        perm.set_mode(u32::from(libc::S_IWUSR | libc::S_IRUSR));
        #[cfg(not(target_os = "macos"))]
        perm.set_mode(libc::S_IWUSR | libc::S_IRUSR);

        file.set_permissions(perm)
            .expect("Failed to set permissions for a key file.");
    }

    let content = serde_json::json!({
        "account_id": id,
        "public_key": sk.public_key(),
        "secret_key": sk,
    })
    .to_string()
    .into_bytes();

    if let Err(err) = file.write_all(&content) {
        panic!("Failed to write a key file {}", err);
    }
}
