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

    // const refFinance = await creator.createAccountFrom({
    //     mainnetContract: REF_FINANCE_ACCOUNT,
    //     block_id,
    //     initialBalance: NEAR.parse('1000 N').toJSON(),
    // });

    Ok(ref_finance)
}


async fn create_pool_with_liquidity(worker: &Worker<impl Network>, ft: &Contract, ref_finance: &Contract, token_amounts: HashMap<AccountId, u128>) -> anyhow::Result<()> {
    let token_ids = token_amounts.keys().cloned().map(Into::into).collect::<Vec<String>>();
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
    ).await?;



    // const pool_id: string = await root.call(refFinance, 'add_simple_pool', {tokens, fee: 25}, {
    //   attachedDeposit: NEAR.parse('3mN'),
    // });

    // await root.call(refFinance, 'register_tokens', {token_ids: tokens}, {
    //   attachedDeposit: '1',
    // });
    // await depositTokens(root, refFinance, tokenAmounts);
    // await root.call(refFinance, 'add_liquidity', {
    //   pool_id,
    //   amounts: Object.values(tokenAmounts),
    // }, {
    //   attachedDeposit: NEAR.parse('1N'),
    // });
    // return pool_id;

    Ok(())
  }



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let testnet_worker = workspaces::testnet();
    let worker = workspaces::sandbox();

    let ref_finance = create_ref(&worker).await?;
    println!("ref account id: {:?}", ref_finance.id());

    // let ft: Contract = worker.dev_deploy(std::fs::read(FT_CONTRACT_FILEPATH)?).await?;
    // worker.call(
    //     &ft,
    //     "new_default_meta".into(),
    //     serde_json::json!({
    //         "owner_id": ft.id().clone(),
    //         "total_supply": parse_near!("1,000,000,000 N"),
    //     })
    //     .to_string()
    //     .into_bytes(),
    //     None,
    // ).await?;

    // let pool_id = create_pool_with_liquidity(&worker, &ft, &ref_finance, maplit::hashmap! {
    //     ft.id().clone() => parse_near!("5 N"),
    //     ref_finance.id().clone() => parse_near!("10 N"),
    // }).await?;

    // let status: String = serde_json::from_value(result)?;
    // println!("New status patched: {:?}", status);

    Ok(())
}
