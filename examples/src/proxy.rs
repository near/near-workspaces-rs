use serde_json::json;
use workspaces::prelude::*;
use workspaces::types::{KeyType, SecretKey};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    _test_proxy().await?;
    Ok(())
}

async fn _test_proxy() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let mut keypom = worker
        .dev_deploy(include_bytes!("../res/main.wasm"))
        .await?;
    let nft_series = worker
        .dev_deploy(include_bytes!("../res/nft-series.wasm"))
        .await?;

    keypom
        .call(&worker, "new")
        .args_json(json!({
            "root_account": "testnet",
            "owner_id": keypom.id(),
        }))
        .transact()
        .await?;

    nft_series
        .call(&worker, "new_default_meta")
        .args_json(json!({
            "owner_id": nft_series.id(),
        }))
        .transact()
        .await?;

    // let ali = root
    //     .create_subaccount("ali")
    //     .transact()
    //     .await?
    //     .into_result()?;
    // let bob = root
    //     .create_subaccount("bob")
    //     .transact()
    //     .await?
    //     .into_result()?;
    let owner = root
        .create_subaccount(&worker, "owner")
        .initial_balance(near_units::parse_near!("10"))
        .transact()
        .await?
        .into_result()?;

    println!("Creating series");
    nft_series
        .call(&worker, "create_series")
        .args_json(json!({
            "metadata": json!({
                "title": "Linkdropped Go Team NFT",
                "description": "Testing Linkdrop NFT Go Team Token",
                "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif",
                "media_hash": null,
                "copies": 100,
                "issued_at": null,
                "expires_at": null,
                "starts_at": null,
                "updated_at": null,
                "extra": null,
                "reference": null,
                "reference_hash": null
            }),
            "id": 0,
        }))
        .deposit(1000000000000000000000000)
        .transact()
        .await?;

    println!("Adding to balance");
    owner
        .call(&worker, keypom.id(), "add_to_balance")
        .deposit(near_units::parse_near!("8"))
        .transact()
        .await?;

    println!("Creating drop");
    let sk = SecretKey::from_random(KeyType::ED25519);
    owner
        .call(&worker, keypom.id(), "create_drop")
        .args_json(json!({
            "public_keys": [sk.public_key()],
            "deposit_per_use": near_units::parse_near!("5 mN").to_string(),
            "fc_data": json!({
                "methods": [null, null, [json!({
                    "receiver_id": nft_series.id(),
                    "method_name": "nft_mint",
                    "args": "",
                    "attached_deposit": near_units::parse_near!("0.01").to_string(),
                    "account_id_field": "receiver_id",
                    "drop_id_field": "id",
                })]]
            }),
            "config": json!({
                "uses_per_key": 3,
                "on_claim_refund_deposit": true,
            }),
        }))
        .gas(300000000000000)
        .transact()
        .await?;

    keypom.as_mut_account().signer_mut().secret_key = sk;
    let res = keypom
        .call(&worker, "claim")
        .args_json(json!({
            "account_id": keypom.id()
        }))
        .gas(100000000000000)
        .transact()
        .await?;
    eprintln!("{:?}", res.logs());

    Ok(())
}
