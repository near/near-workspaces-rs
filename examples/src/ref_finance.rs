use std::{convert::TryInto, collections::HashMap};
use workspaces::{prelude::*, AccountId, Contract, Account, Network, Worker};
use near_units::{parse_near, parse_gas};

const FT_CONTRACT_FILEPATH: &str =  "./examples/res/fungible_token.wasm";
const REF_FINANCE_ACCOUNT_ID: &str = "v2.ref-finance.near";

async fn create_ref(root: &Account, worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet();
    let ref_finance_id: AccountId = REF_FINANCE_ACCOUNT_ID.to_string().try_into().unwrap();
    let ref_finance = worker.import_contract(ref_finance_id.clone(), &mainnet)
        .with_initial_balance(parse_near!("1000 N"))
        .transact()
        .await?;

    root.call(&worker, ref_finance.id().clone(), "new".into())
        .with_args(
            serde_json::json!({
                "owner_id": ref_finance.id().clone(),
                "exchange_fee": 4,
                "referral_fee": 1,
            }).to_string().into_bytes(),
        )
        .transact()
        .await?;

    root.call(&worker, ref_finance.id().clone(), "storage_deposit".into())
        .with_args(serde_json::json!({}).to_string().into_bytes())
        .with_deposit(parse_near!("30 mN"))
        .transact()
        .await?;

    Ok(ref_finance)
}

async fn create_wnear(owner: &Account, worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet();
    let wnear_id: AccountId = "wrap.near".to_string().try_into().unwrap();
    let wnear = worker.import_contract(wnear_id.clone(), &mainnet).transact().await?;

    owner.call(&worker, wnear.id().clone(), "new".into())
        .with_args(
            serde_json::json!({
                "owner_id": owner.id().clone(),
                "total_supply": parse_near!("1,000,000,000 N"),
            }).to_string().into_bytes(),
        )
        .transact()
        .await?;

    owner.call(&worker, wnear.id().clone(), "storage_deposit".into())
        .with_args(serde_json::json!({}).to_string().into_bytes())
        .with_deposit(parse_near!("0.008 N"))
        .transact()
        .await?;


    owner.call(&worker, wnear.id().clone(), "near_deposit".into())
        .with_args(serde_json::json!({}).to_string().into_bytes())
        .with_deposit(parse_near!("200 N"))
        .transact()
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

    root.call(&worker, ref_finance.id().clone(), "register_tokens".into())
        .with_args(serde_json::json!({
            "token_ids": token_ids,
        }).to_string().into_bytes())
        .with_deposit(1)
        .transact()
        .await?;

    deposit_tokens(worker, root, &ref_finance, tokens).await?;

    root.call(&worker, ref_finance.id().clone(), "add_liquidity".into())
        .with_args(serde_json::json!({
            "pool_id": pool_id,
            "amounts": token_amounts,
        }).to_string().into_bytes())
        .with_deposit(parse_near!("1 N"))
        .transact()
        .await?;

    Ok(pool_id)
}


async fn deposit_tokens(
    worker: &Worker<impl Network>,
    root: &Account,
    ref_finance: &Contract,
    tokens: HashMap<&Contract, u128>
) -> anyhow::Result<()> {
    for (contract, amount) in tokens {
        ref_finance.call(&worker, contract.id().clone(), "storage_deposit".into())
            .with_args(serde_json::json!({
                "registration_only": true,
            }).to_string().into_bytes())
            .with_deposit(parse_near!("1 N"))
            .transact()
            .await?;

        root.call(&worker, contract.id().clone(), "ft_transfer_call".into())
            .with_args(serde_json::json!({
                "receiver_id": ref_finance.id().clone(),
                "amount": amount.to_string(),
                "msg": "",
            }).to_string().into_bytes())
            .with_gas(parse_gas!("200 Tgas") as u64)
            .with_deposit(1)
            .transact()
            .await?;
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
            "owner_id": root.id().clone(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        })
        .to_string()
        .into_bytes(),
        None,
    ).await?;

    let ref_finance = create_ref(&root, &worker).await?;
    let wnear = create_wnear(&root, &worker).await?;

    let pool_id = create_pool_with_liquidity(&worker, &root, &ref_finance, maplit::hashmap! {
        &ft => parse_near!("5 N"),
        &wnear => parse_near!("10 N"),
    }).await?;
    println!("Created a liquid pool with id: {:?}", ref_finance.id());

    deposit_tokens(&worker, &root, &ref_finance, maplit::hashmap! {
        &ft => parse_near!("100 N"),
        &wnear => parse_near!("100 N"),
    }).await?;

    let ft_deposit = worker.view(
        ref_finance.id().clone(),
        "get_deposit".into(),
        serde_json::json!({
            "account_id": root.id().clone(),
            "token_id": ft.id().clone(),
        })
        .to_string()
        .into_bytes(),
    ).await?;
    println!("Current FT deposit: {}", ft_deposit);
    let ft_deposit: String = serde_json::from_str(&ft_deposit)?;
    assert_eq!(ft_deposit, parse_near!("100 N").to_string());

    let wnear_deposit = worker.view(
        ref_finance.id().clone(),
        "get_deposit".into(),
        serde_json::json!({
            "account_id": root.id().clone(),
            "token_id": wnear.id().clone(),
        })
        .to_string()
        .into_bytes(),
    ).await?;
    println!("Current WNear deposit: {}", wnear_deposit);
    let wnear_deposit: String = serde_json::from_str(&wnear_deposit)?;
    assert_eq!(wnear_deposit, parse_near!("100 N").to_string());

    let total_shares = worker.view(
        ref_finance.id().clone(),
        "get_pool_total_shares".into(),
        serde_json::json!({
            "pool_id": pool_id,
        }).to_string().into_bytes(),
    ).await?;
    println!("Pool[{}] total_shares: {}", pool_id, total_shares);
    let total_shares: String = serde_json::from_str(&total_shares)?;
    assert_eq!(total_shares, "1000000000000000000000000");

    let expected_return = worker.view(
        ref_finance.id().clone(),
        "get_return".into(),
        serde_json::json!({
            "pool_id": pool_id,
            "token_in": ft.id().clone(),
            "token_out": wnear.id().clone(),
            "amount_in": parse_near!("1 N").to_string(),
        }).to_string().into_bytes(),
    ).await?;
    println!("Expect return for trading in 1 FT token for WNear: {}", expected_return);
    let expected_return: String = serde_json::from_str(&expected_return)?;
    assert_eq!(expected_return, "1662497915624478906119726");

    let actual_out = root.call(&worker, ref_finance.id().clone(), "swap".into())
        .with_args(
            serde_json::json!({
                "actions": vec![serde_json::json!({
                    "pool_id": pool_id,
                    "token_in": ft.id().clone(),
                    "token_out": wnear.id().clone(),
                    "amount_in": parse_near!("1 N").to_string(),
                    "min_amount_out": "1",
                })],
            }).to_string().into_bytes(),
        )
        .with_deposit(1)
        .with_gas(parse_gas!("100 Tgas") as u64)
        .transact()
        .await?
        .try_into_call_result()?;
    println!("Actual return for trading in 1 FT token for WNear: {}", actual_out);
    let actual_out: String = serde_json::from_str(&actual_out)?;
    assert_eq!(actual_out, expected_return);

    let ft_deposit = worker.view(ref_finance.id().clone(), "get_deposit".into(),
        serde_json::json!({
            "account_id": root.id().clone(),
            "token_id": ft.id().clone(),
        }).to_string().into_bytes(),
    ).await?;
    println!("New FT deposit after swap: {}", ft_deposit);
    let ft_deposit: String = serde_json::from_str(&ft_deposit)?;
    assert_eq!(ft_deposit, parse_near!("99 N").to_string());

    let wnear_deposit = worker.view(
        ref_finance.id().clone(),
        "get_deposit".into(),
        serde_json::json!({
            "account_id": root.id().clone(),
            "token_id": wnear.id().clone(),
        })
        .to_string()
        .into_bytes(),
    ).await?;
    println!("New WNear deposit after swap: {}", wnear_deposit);

    Ok(())
}
