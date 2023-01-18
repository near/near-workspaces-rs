use serde_json::json;
use workspaces::{
    types::{KeyType, SecretKey},
    AccessKey, Account, AccountId,
};

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    // let contract = worker.dev_deploy(&wasm).await?;

    // let id: AccountId = "3jicfTiSAkKjcEGHlMEkhlHTZVAUVVyR".parse()?;
    let (id, _) = worker.dev_generate().await;
    let sk = SecretKey::from_seed(KeyType::SECP256K1, "test");
    let account = worker.create_tla(id, sk).await?.into_result()?;
    let contract = account.deploy(&wasm).await?.into_result()?;

    // let root = worker.root_account()?;
    // root.batch(&id)
    //     .create_account()
    //     .transfer(near_units::parse_near!("100 N"))
    //     .add_key(sk.public_key(), AccessKey::full_access())
    //     .transact()
    //     .await?
    //     .into_result()?;

    // let account = Account::from_secret_key(id, sk, &worker);
    // let contract = account.deploy(&wasm).await?.into_result()?;

    let outcome = contract
        .call("set_status")
        .args_json(json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;
    println!("set_status: {:?}", outcome);

    let result: String = contract
        .view("get_status")
        .args_json(json!({
            "account_id": contract.id(),
        }))
        .await?
        .json()?;

    println!("status: {:?}", result);

    Ok(())
}
