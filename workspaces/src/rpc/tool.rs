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

use crate::error::{ErrorKind, RpcErrorCode};
use crate::result::Result;
use crate::types::{AccountId, PublicKey};

/// Convert `StateItem`s over to a Map<data_key, value_bytes> representation.
/// Assumes key and value are base64 encoded, so this also decodes them.
pub(crate) fn into_state_map(state_items: Vec<StateItem>) -> HashMap<Vec<u8>, Vec<u8>> {
    state_items
        .into_iter()
        .map(|s| (s.key.into(), s.value.into()))
        .collect()
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
) -> Result<()> {
    let helper_url = helper_url.join("account").unwrap();

    // TODO(maybe): need this in near-jsonrpc-client as well:
    let response = reqwest::Client::new()
        .post(helper_url)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_vec(&serde_json::json!({
                "newAccountId": account_id,
                "newAccountPublicKey": pk,
            }))
            .map_err(|e| ErrorKind::DataConversion.custom(e))?,
        )
        .send()
        .await
        .map_err(|e| RpcErrorCode::HelperAccountCreationFailure.custom(e))?;

    if response.status() >= reqwest::StatusCode::BAD_REQUEST {
        return Err(ErrorKind::Other.message(format!(
            "The faucet (helper service) server failed with status code <{}>",
            response.status()
        )));
    }

    let account_creation_transaction = response
        .json::<near_jsonrpc_client::methods::tx::RpcTransactionResponse>()
        .await
        .map_err(|err| ErrorKind::DataConversion.custom(err))?
        .final_execution_outcome
        .map(|outcome| outcome.into_outcome())
        .ok_or_else(|| {
            ErrorKind::Other.message(
                "The faucet (helper service) server did not return a transaction response.",
            )
        })?;

    match account_creation_transaction.status {
        near_primitives::views::FinalExecutionStatus::SuccessValue(ref value) => {
            if value == b"false" {
                return Err(ErrorKind::Other.message(format!(
                    "The new account <{}> could not be created successfully.",
                    &account_id
                )));
            }
        }
        near_primitives::views::FinalExecutionStatus::Failure(err) => {
            return Err(ErrorKind::Execution.custom(err));
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub(crate) fn write_cred_to_file(path: &Path, id: &AccountId, sk: &SecretKey) -> Result<()> {
    let mut file = File::create(path).map_err(|err| {
        ErrorKind::Io.full(
            format!("failed to open {path:?} for writing credentials"),
            err,
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::prelude::PermissionsExt;
        let mut perm = file
            .metadata()
            .map_err(|err| ErrorKind::Io.full("Failed to retrieve key file metadata.", err))?
            .permissions();

        #[cfg(target_os = "macos")]
        perm.set_mode(u32::from(libc::S_IWUSR | libc::S_IRUSR));
        #[cfg(not(target_os = "macos"))]
        perm.set_mode(libc::S_IWUSR | libc::S_IRUSR);

        file.set_permissions(perm)
            .map_err(|err| ErrorKind::Io.full("Failed to set permissions for a key file.", err))?;
    }

    let content = serde_json::json!({
        "account_id": id,
        "public_key": sk.public_key(),
        "secret_key": sk,
    })
    .to_string()
    .into_bytes();

    file.write_all(&content)
        .map_err(|err| ErrorKind::Io.full("Failed to write a key file", err))
}
