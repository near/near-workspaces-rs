use std::{convert::TryInto, collections::HashMap};
use workspaces::{prelude::*, AccountId, Contract, Account, Network, Worker};
use near_units::{parse_near, parse_gas};

const FT_CONTRACT_FILEPATH: &str =  "./examples/res/fungible_token.wasm";
const REF_FINANCE_ACCOUNT_ID: &str = "v2.ref-finance.near";

async fn create_ref(owner: AccountId, worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
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
            "owner_id": ref_finance.id().clone(),
            "exchange_fee": 4,
            "referral_fee": 1,
        }).to_string().into_bytes(),
        None,
    ).await?;

    worker.call(
        &ref_finance,
        "storage_deposit".into(),
        // Vec::new(),
        serde_json::json!({}).to_string().into_bytes(),
        Some(parse_near!("30 mN")),
    ).await?;

    Ok(ref_finance)
}

async fn create_wnear(owner: AccountId, worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet();
    let wnear_id: AccountId = "wrap.near".to_string().try_into().unwrap();
    let wnear = worker.import_contract(wnear_id.clone(), &mainnet).transact().await?;

    worker.call(
        &wnear,
        "new".into(),
        serde_json::json!({
            "owner_id": owner,
            "total_supply": parse_near!("1,000,000,000 N"),
        }).to_string().into_bytes(),
        None,
    ).await?;

    worker.call(
        &wnear,
        "storage_deposit".into(),
        serde_json::json!({}).to_string().into_bytes(),
        Some(parse_near!("0.008 N")),
    )
    .await?;

    worker.call(
        &wnear,
        "near_deposit".into(),
        serde_json::json!({}).to_string().into_bytes(),
        Some(parse_near!("200 N")),
    )
    .await?;

    Ok(wnear)
  }


async fn create_pool_with_liquidity(worker: &Worker<impl Network>, root: &Account, ref_finance: &Contract, tokens: HashMap<&Contract, u128>) -> anyhow::Result<u64> {
    let (token_ids, token_amounts): (Vec<String>, Vec<String>) = tokens.iter().map(|(key, val)| (key.id().clone().into(), val.to_string())).unzip();

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
    let pool_id: u64 = serde_json::from_str(&pool_id)?;

    root.call_other(&worker, ref_finance.id().clone(), "register_tokens".into())
        .with_args(serde_json::json!({
            "token_ids": token_ids,
        }).to_string().into_bytes())
        .with_deposit(1)
        .transact()
        .await?;

    // worker.call(
    //     &ref_finance,
    //     "register_tokens".into(),
    //     serde_json::json!({
    //         "token_ids": token_ids,
    //     }).to_string().into_bytes(),
    //     Some(1),
    // ).await?;

    deposit_tokens(worker, root, &ref_finance, tokens).await?;

    root.call_other(&worker, ref_finance.id().clone(), "add_liquidity".into())
        .with_args(serde_json::json!({
            "pool_id": pool_id,
            "token_amounts": token_amounts,
        }).to_string().into_bytes())
        .with_deposit(parse_near!("1 N"))
        .transact()
        .await?;

    // worker.call(
    //     &ref_finance,
    //     "add_liquidity".into(),
    //     serde_json::json!({
    //         "pool_id": pool_id,
    //         "amounts": token_amounts,
    //     }).to_string().into_bytes(),
    //     Some(parse_near!("1 N")),
    // ).await?;

    Ok(pool_id)
}


async fn deposit_tokens(
    worker: &Worker<impl Network>,
    root: &Account,
    ref_finance: &Contract,
    tokens: HashMap<&Contract, u128>
) -> anyhow::Result<()> {
    for (contract, amount) in tokens {
        // worker.call(
        //     &ref_finance,
        //     "storage_deposit".into(),
        //     serde_json::json!({
        //         "registration_only": true,
        //     }).to_string().into_bytes(),
        //     Some(parse_near!("1 N")),
        // ).await?;

        ref_finance.call_other(&worker, contract.id().clone(), "storage_deposit".into())
            .with_args(serde_json::json!({
                "registration_only": true,
            }).to_string().into_bytes())
            .with_deposit(parse_near!("1 N"))
            .transact()
            .await?;

        root.call_other(&worker, contract.id().clone(), "ft_transfer_call".into())
            .with_args(serde_json::json!({
                "receiver_id": ref_finance.id().clone(),
                "amount": amount.to_string(),
                "msg": "",
            }).to_string().into_bytes())
            .with_gas(parse_gas!("200 Tgas") as u64)
            .with_deposit(1)
            .transact()
            .await?;

        // worker.call(
        //     &contract,
        //     "ft_transfer_call".into(),
        //     serde_json::json!({
        //         "receiver_id": ref_finance.id().clone(),
        //         "amount": amount,
        //         "msg": "",
        //     }).to_string().into_bytes(),
        //     Some(1),
        //     //     gas: Gas.parse('200Tgas'),
        // ).await?;

        // contract.call(&worker, "ft_transfer_call".into())
        //     .with_args(serde_json::json!({
        //         "receiver_id": ref_finance.id().clone(),
        //         "amount": amount.to_string(),
        //         "msg": "",
        //     }).to_string().into_bytes())
        //     .with_gas(parse_gas!("200 Tgas") as u64)
        //     .with_deposit(1)
        //     .transact()
        //     .await?;
    }

    Ok(())
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let root = worker.root_account();

    let ft: Contract = worker.dev_deploy(std::fs::read(FT_CONTRACT_FILEPATH)?).await?;
    worker.call(
        &ft,
        "new_default_meta".into(),
        serde_json::json!({
            "owner_id": ft.id().clone(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        })
        .to_string()
        .into_bytes(),
        None,
    ).await?;

    let ref_finance = create_ref(ft.id().clone(), &worker).await?;
    let wnear = create_wnear(ft.id().clone(), &worker).await?;
    println!("ref account id: {:?}", ref_finance.id());

    let pool_id = create_pool_with_liquidity(&worker, &root, &ref_finance, maplit::hashmap! {
        &ft => parse_near!("5 N"),
        &wnear => parse_near!("10 N"),
    }).await?;

    // deposit_tokens(&worker, &ref_finance, maplit::hashmap! {
    //     &ft => parse_near!("100 N"),
    //     &wnear => parse_near!("100 N"),
    // }).await?;

    // let result1 = worker.view(
    //     ref_finance.id().clone(),
    //     "get_deposit".into(),
    //     serde_json::json!({
    //         "account_id": ft.id().clone(),
    //         "token_id": ft.id().clone(),
    //     })
    //     .to_string()
    //     .into_bytes(),
    // ).await?;

    // let result2 = worker.view(
    //     ref_finance.id().clone(),
    //     "get_deposit".into(),
    //     serde_json::json!({
    //         "account_id": ft.id().clone(),
    //         "token_id": wnear.id().clone(),
    //     })
    //     .to_string()
    //     .into_bytes(),
    // ).await?;


    // println!(
    //     "--------------\n{}\n{}",
    //     serde_json::to_string_pretty(&result1).unwrap(),
    //     serde_json::to_string_pretty(&result2).unwrap(),
    // );

    // let result3 = worker.view(
    //     ref_finance.id().clone(),
    //     "get_pool_total_shares".into(),
    //     serde_json::json!({
    //         "pool_id": pool_id,
    //     }).to_string().into_bytes(),
    // ).await?;

    // println!(
    //     "--------------\n{}",
    //     serde_json::to_string_pretty(&result3).unwrap(),
    // );

    Ok(())
}
