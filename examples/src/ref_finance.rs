use std::{convert::TryInto, collections::HashMap};
use workspaces::{prelude::*, AccountId, Contract, Network, Worker};

const TESTNET_PREDEPLOYED_CONTRACT_ID: &str = "dev-20211013002148-59466083160385";

const FT_CONTRACT_FILEPATH: &str =  "./examples/res/fungible_token.wasm";
const REF_FINANCE_ACCOUNT_ID: &str = "v2.ref-finance.near";

async fn create_ref(worker: &Worker<impl Network + StatePatcher>) -> anyhow::Result<Contract> {
    let testnet = workspaces::testnet();
    let ref_finance_id: AccountId = REF_FINANCE_ACCOUNT_ID.to_string().try_into().unwrap();
    let ref_finance = worker.create_contract_from(ref_finance_id.clone(), testnet).await?;

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
            "attachedDeposit": 3000000, //NEAR.parse('30 mN'),
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
//   await root.call(refFinance, 'extend_whitelisted_tokens', {tokens});

     worker.call(
         &ref_finance,
         "extend_whitelisted_tokens".into(),
        serde_json::json!({
            "tokens": token_amounts.keys().cloned().map(Into::into).collect::<Vec<String>>(),
        }).to_string().into_bytes(),
         None,
    ).await?;




    // root: Account,
    // refFinance: NearAccount,
    // tokenAmounts: Record<AccountID, string>,
//   ): Promise<string> {


    // const tokens = Object.keys(tokenAmounts);
    // await root.call(refFinance, 'extend_whitelisted_tokens', {tokens});
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
    let testnet_worker = workspaces::testnet();
    let worker = workspaces::sandbox();

    let ft: Contract = worker.dev_deploy(std::fs::read(FT_CONTRACT_FILEPATH)?).await?;
    // create_pool_with_liquidity(&worker, &ft, ), HashMap::new()).await?;

    let contract_id: AccountId = TESTNET_PREDEPLOYED_CONTRACT_ID
        .to_string()
        .try_into()
        .unwrap();

    let status_msg = worker.create_contract_from(contract_id, testnet_worker).await?;

    println!("ACCOUNT: {:?}", status_msg.id());

    // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let result = worker
        .view(
            status_msg.id().clone(),
            "get_status".into(),
            serde_json::json!({
                "account_id": TESTNET_PREDEPLOYED_CONTRACT_ID,
            })
            .to_string()
            .into_bytes(),
        )
        .await?;


    let status: String = serde_json::from_value(result)?;
    println!("New status patched: {:?}", status);

    Ok(())
}
