//! Network specific configurations used to modify behavior inside a chain.
//! This is so far only useable with sandbox networks since it would require
//! direct access to a node to change the config. Each network like mainnet
//! and testnet already have pre-configured settings; meanwhile sandbox can
//! have additional settings on top of them to facilitate custom behavior
//! such as sending large requests to the sandbox network.
//
// NOTE: nearcore has many, many configs which can easily change in the future
// so this config.rs file just purely modifies the data and does not try to
// replicate all the structs from nearcore side; which can be a huge maintenance
// churn if we were to.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::Value;

use crate::error::ErrorKind;
use crate::Result;

/// Overwrite the $home_dir/config.json file over a set of entries. `value` will be used per (key, value) pair
/// where value can also be another dict. This recursively sets all entry in `value` dict to the config
/// dict, and saves back into `home_dir` at the end of the day.
fn overwrite(home_dir: &Path, value: Value) -> Result<()> {
    let config_file =
        File::open(home_dir.join("config.json")).map_err(|err| ErrorKind::Io.custom(err))?;
    let config = BufReader::new(config_file);
    let mut config: Value =
        serde_json::from_reader(config).map_err(|err| ErrorKind::DataConversion.custom(err))?;

    json_patch::merge(&mut config, &value);
    let config_file =
        File::create(home_dir.join("config.json")).map_err(|err| ErrorKind::Io.custom(err))?;
    serde_json::to_writer(config_file, &config).map_err(|err| ErrorKind::Io.custom(err))?;

    Ok(())
}

/// Limit how much nearcore/sandbox can receive per payload. The default set by nearcore is not
/// enough for certain sandbox operations like patching contract state in the case of contracts
/// larger than 10mb.
fn max_sandbox_json_payload_size() -> Result<u64> {
    let max_files = match std::env::var("NEAR_SANDBOX_MAX_PAYLOAD_SIZE") {
        Ok(val) => val
            .parse::<u64>()
            .map_err(|err| ErrorKind::DataConversion.custom(err))?,

        // Default is 1GB which should suit most contract sizes.
        Err(_err) => 1024 * 1024 * 1024,
    };

    Ok(max_files)
}

/// Get the max files for workspaces. `NEAR_SANDBOX_MAX_FILES` env var will be used and if not
/// specified, will default to a max of 5000 handles by default as to not ulimit errors on certain
/// platforms like Windows WSL2.
fn max_files() -> Result<u64> {
    let max_files = match std::env::var("NEAR_SANDBOX_MAX_FILES") {
        Ok(val) => (&val)
            .parse::<u64>()
            .map_err(|err| ErrorKind::DataConversion.custom(err))?,
        Err(_err) => 4096,
    };

    Ok(max_files)
}

/// Set extra configs for the sandbox defined by workspaces.
pub(crate) fn set_sandbox_configs(home_dir: &Path) -> Result<()> {
    overwrite(
        home_dir,
        serde_json::json!({
            "rpc": {
                "limits_config": {
                    "json_payload_max_size": max_sandbox_json_payload_size()?,
                },
            },
            "store": {
                "max_open_files": max_files()?,
            }
        }),
    )
}
