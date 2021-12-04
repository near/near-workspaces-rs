use std::{convert::TryInto, collections::HashMap};
use workspaces::{prelude::*, AccountId, Contract, Network, Worker};
use near_units::parse_near;

const FT_CONTRACT_FILEPATH: &str =  "./examples/res/fungible_token.wasm";
const REF_FINANCE_ACCOUNT_ID: &str = "v2.ref-finance.near";

async fn create_ref(worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet();
    let ref_finance_id: AccountId = REF_FINANCE_ACCOUNT_ID.to_string().try_into().unwrap();
    let ref_finance = worker.import_contract(ref_finance_id.clone(), &mainnet)
        .with_initial_balance(parse_near!("1000 N"))
        .transact()
        .await?;

    worker.call(
        &ref_finance,
        "new".into(),
        serde_json::json!({
            "owner_id": ref_finance_id,
            "exchange_fee": 4,
            "referral_fee": 1,
        }).to_string().into_bytes(),
        None,
    ).await?;

    worker.call(
        &ref_finance,
        "storage_deposit".into(),
        serde_json::json!({
            // TODO:
            "attachedDeposit": parse_near!("30 mN"),
        }).to_string().into_bytes(),
        None,
    ).await?;

    Ok(ref_finance)
}

async fn create_wnear(worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet();
    let wnear_id: AccountId = "wrap.near".to_string().try_into().unwrap();
    let wnear = worker.import_contract(wnear_id.clone(), &mainnet).transact().await?;

    worker.call(
        &wnear,
        "new".into(),
        serde_json::json!({
            "owner_id": wnear_id.clone(),
            "total_supply": parse_near!("1,000,000,000 N"),
        }).to_string().into_bytes(),
        None,
    ).await?;

    worker.call(
        &wnear,
        "storage_deposit".into(),
        Vec::new(),
        Some(parse_near!("0.008 N")),
    )
    .await?;

    worker.call(
        &wnear,
        "storage_deposit".into(),
        Vec::new(),
        Some(parse_near!("200 N")),
    )
    .await?;

    Ok(wnear)
  }


async fn create_pool_with_liquidity(worker: &Worker<impl Network>, ref_finance: &Contract, tokens: HashMap<&Contract, u128>) -> anyhow::Result<String> {
    // let token_ids = tokens.keys().cloned().map(Into::into).collect::<Vec<String>>();
    // let token_amounts = tokens.
    let (token_ids, token_amounts): (Vec<String>, Vec<u128>) = tokens.iter().map(|(key, val)| (key.id().clone().into(), val)).unzip();

    worker.call(
        &ref_finance,
        "extend_whitelisted_tokens".into(),
        serde_json::json!({ "tokens": token_ids }).to_string().into_bytes(),
        None,
    ).await?;

    let pool_id = worker.call(
        &ref_finance,
        "add_simple_pool".into(),
        serde_json::json!({
            "tokens": token_ids,
            "fee": 25,
        }).to_string().into_bytes(),
        Some(parse_near!("3 mN")),
    ).await?
    .try_into_call_result()?;

    worker.call(
        &ref_finance,
        "register_tokens".into(),
        serde_json::json!({
            "token_ids": token_ids,
        }).to_string().into_bytes(),
        Some(1),
    ).await?;

    deposit_tokens(worker, &ref_finance, tokens).await?;

    worker.call(
        &ref_finance,
        "add_liquidity".into(),
        serde_json::json!({
            "pool_id": pool_id,
            "amounts": token_amounts,
        }).to_string().into_bytes(),
        Some(parse_near!("1 N")),
    ).await?;

    Ok(pool_id)
}


async fn deposit_tokens(
    worker: &Worker<impl Network>,
    ref_finance: &Contract,
    tokens: HashMap<&Contract, u128>
) -> anyhow::Result<()> {
    for (contract, amount) in tokens {
        worker.call(
            &ref_finance,
            "storage_deposit".into(),
            serde_json::json!({
                "registration_only": true,
            }).to_string().into_bytes(),
            Some(parse_near!("1 N")),
        ).await?;

        worker.call(
            &contract,
            "ft_transfer_call".into(),
            serde_json::json!({
                "receiver_id": ref_finance.id().clone(),
                "amount": amount,
                "msg": "",
            }).to_string().into_bytes(),
            Some(1),
            //     gas: Gas.parse('200Tgas'),
        ).await?;
    }

    Ok(())
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();

    let ref_finance = create_ref(&worker).await?;
    let wnear = create_wnear(&worker).await?;

    println!("ref account id: {:?}", ref_finance.id());

    let ft: Contract = worker.dev_deploy(std::fs::read(FT_CONTRACT_FILEPATH)?).await?;
    worker.call(
        &ft,
        "new_default_meta".into(),
        serde_json::json!({
            "owner_id": ft.id().clone(),
            "total_supply": parse_near!("1,000,000,000 N"),
        })
        .to_string()
        .into_bytes(),
        None,
    ).await?;

    let pool_id = create_pool_with_liquidity(&worker, &ref_finance, maplit::hashmap! {
        &ft => parse_near!("5 N"),
        &wnear => parse_near!("10 N"),
    }).await?;

    println!("pool_id: {}", pool_id);

    Ok(())
}
