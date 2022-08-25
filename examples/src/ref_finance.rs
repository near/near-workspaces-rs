use std::collections::HashMap;
use std::convert::TryInto;

use near_units::{parse_gas, parse_near};
use workspaces::network::Sandbox;
use workspaces::{Account, AccountId, Contract, Worker};
use workspaces::{BlockHeight, DevNetwork};

const FT_CONTRACT_FILEPATH: &str = "./examples/res/fungible_token.wasm";

/// Contract id of ref-finance on mainnet.
const REF_FINANCE_ACCOUNT_ID: &str = "v2.ref-finance.near";

/// BlockId referencing back to a specific time just in case the contract has
/// changed or has been updated at a later time.
const BLOCK_HEIGHT: BlockHeight = 50_000_000;

/// Pull down the ref-finance contract and deploy it to the sandbox network,
/// initializing it with all data required to run the tests.
async fn create_ref(owner: &Account, worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet_archival().await?;
    let ref_finance_id: AccountId = REF_FINANCE_ACCOUNT_ID.parse()?;

    // This will pull down the relevant ref-finance contract from mainnet. We're going
    // to be overriding the initial balance with 1000N instead of what's on mainnet.
    let ref_finance = worker
        .import_contract(&ref_finance_id, &mainnet)
        .initial_balance(parse_near!("1000 N"))
        .block_height(BLOCK_HEIGHT)
        .transact()
        .await?;

    // NOTE: We are not pulling down the contract's data here, so we'll need to initalize
    // our own set of metadata. This is because the contract's data is too big for the rpc
    // service to pull down (i.e. greater than 50mb).

    owner
        .call(ref_finance.id(), "new")
        .args_json(serde_json::json!({
            "owner_id": ref_finance.id(),
            "exchange_fee": 4,
            "referral_fee": 1,
        }))
        .transact()
        .await?
        .executed()?;

    owner
        .call(ref_finance.id(), "storage_deposit")
        .args_json(serde_json::json!({}))
        .deposit(parse_near!("30 mN"))
        .transact()
        .await?
        .executed()?;

    Ok(ref_finance)
}

/// Pull down the WNear contract from mainnet and initilize it with our own metadata.
async fn create_wnear(owner: &Account, worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let mainnet = workspaces::mainnet_archival().await?;
    let wnear_id: AccountId = "wrap.near".to_string().try_into()?;
    let wnear = worker
        .import_contract(&wnear_id, &mainnet)
        .block_height(BLOCK_HEIGHT)
        .transact()
        .await?;

    owner
        .call(wnear.id(), "new")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N"),
        }))
        .transact()
        .await?
        .executed()?;

    owner
        .call(wnear.id(), "storage_deposit")
        .args_json(serde_json::json!({}))
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?
        .executed()?;

    owner
        .call(wnear.id(), "near_deposit")
        .deposit(parse_near!("200 N"))
        .transact()
        .await?
        .executed()?;

    Ok(wnear)
}

/// Create a liquidity pool on Ref-Finance, registering the tokens we provide it.
/// Add's the amount in `tokens` we set for liquidity. This will return us the
/// pool_id after the pool has been created.
async fn create_pool_with_liquidity(
    owner: &Account,
    ref_finance: &Contract,
    tokens: HashMap<&AccountId, u128>,
) -> anyhow::Result<u64> {
    let (token_ids, token_amounts): (Vec<String>, Vec<String>) = tokens
        .iter()
        .map(|(id, amount)| (id.to_string(), amount.to_string()))
        .unzip();

    ref_finance
        .call("extend_whitelisted_tokens")
        .args_json(serde_json::json!({ "tokens": token_ids }))
        .transact()
        .await?
        .executed()?;

    let pool_id: u64 = ref_finance
        .call("add_simple_pool")
        .args_json(serde_json::json!({
            "tokens": token_ids,
            "fee": 25
        }))
        .deposit(parse_near!("3 mN"))
        .transact()
        .await?
        .json()?;

    owner
        .call(ref_finance.id(), "register_tokens")
        .args_json(serde_json::json!({
            "token_ids": token_ids,
        }))
        .deposit(1)
        .transact()
        .await?
        .executed()?;
    ();

    deposit_tokens(owner, &ref_finance, tokens).await?;

    owner
        .call(ref_finance.id(), "add_liquidity")
        .args_json(serde_json::json!({
            "pool_id": pool_id,
            "amounts": token_amounts,
        }))
        .deposit(parse_near!("1 N"))
        .transact()
        .await?
        .executed()?;
    ();

    Ok(pool_id)
}

/// Deposit tokens into Ref-Finance
async fn deposit_tokens(
    owner: &Account,
    ref_finance: &Contract,
    tokens: HashMap<&AccountId, u128>,
) -> anyhow::Result<()> {
    for (contract_id, amount) in tokens {
        ref_finance
            .as_account()
            .call(contract_id, "storage_deposit")
            .args_json(serde_json::json!({
                "registration_only": true,
            }))
            .deposit(parse_near!("1 N"))
            .transact()
            .await?
            .executed()?;
        ();

        owner
            .call(contract_id, "ft_transfer_call")
            .args_json(serde_json::json!({
                "receiver_id": ref_finance.id(),
                "amount": amount.to_string(),
                "msg": "",
            }))
            .gas(parse_gas!("200 Tgas") as u64)
            .deposit(1)
            .transact()
            .await?
            .executed()?;
        ();
    }

    Ok(())
}

/// Create our own custom Fungible Token contract and setup the initial state.
async fn create_custom_ft(
    owner: &Account,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    let ft: Contract = worker
        .dev_deploy(&std::fs::read(FT_CONTRACT_FILEPATH)?)
        .await?;

    // Initialize our FT contract with owner metadata and total supply available
    // to be traded and transfered into other contracts such as Ref-Finance
    ft.call("new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))
        .transact()
        .await?
        .executed()?;
    ();

    Ok(ft)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account()?;

    ///////////////////////////////////////////////////////////////////////////
    // Stage 1: Deploy relevant contracts such as FT, WNear, and Ref-Finance
    ///////////////////////////////////////////////////////////////////////////

    let ft = create_custom_ft(&owner, &worker).await?;
    let ref_finance = create_ref(&owner, &worker).await?;
    let wnear = create_wnear(&owner, &worker).await?;

    ///////////////////////////////////////////////////////////////////////////
    // Stage 2: create a pool with liquidity and deposit/transfer tokens into
    // them from our contracts such as FT and WNear.
    ///////////////////////////////////////////////////////////////////////////

    let pool_id = create_pool_with_liquidity(
        &owner,
        &ref_finance,
        maplit::hashmap! {
            ft.id() => parse_near!("5 N"),
            wnear.id() => parse_near!("10 N"),
        },
    )
    .await?;
    println!(
        "Created a liquid pool on {} with id {}",
        ref_finance.id(),
        pool_id
    );

    deposit_tokens(
        &owner,
        &ref_finance,
        maplit::hashmap! {
            ft.id() => parse_near!("100 N"),
            wnear.id() => parse_near!("100 N"),
        },
    )
    .await?;

    ///////////////////////////////////////////////////////////////////////////
    // Stage 3: View our deposited/transferred tokens in ref-finance
    ///////////////////////////////////////////////////////////////////////////

    let ft_deposit: String = worker
        .view(
            ref_finance.id(),
            "get_deposit",
            serde_json::json!({
                "account_id": owner.id(),
                "token_id": ft.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    println!("Current FT deposit: {}", ft_deposit);
    assert_eq!(ft_deposit, parse_near!("100 N").to_string());

    let wnear_deposit: String = worker
        .view(
            ref_finance.id(),
            "get_deposit",
            serde_json::json!({
                "account_id": owner.id(),
                "token_id": wnear.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    println!("Current WNear deposit: {}", wnear_deposit);
    assert_eq!(wnear_deposit, parse_near!("100 N").to_string());

    ///////////////////////////////////////////////////////////////////////////
    // Stage 4: Check how much our expected rate is for swapping and then swap
    ///////////////////////////////////////////////////////////////////////////

    let expected_return: String = worker
        .view(
            ref_finance.id(),
            "get_return",
            serde_json::json!({
                "pool_id": pool_id,
                "token_in": ft.id(),
                "token_out": wnear.id(),
                "amount_in": parse_near!("1 N").to_string(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    println!(
        "Expect return for trading in 1 FT token for WNear: {}",
        expected_return
    );
    assert_eq!(expected_return, "1662497915624478906119726");

    let actual_out = owner
        .call(ref_finance.id(), "swap")
        .args_json(serde_json::json!({
            "actions": vec![serde_json::json!({
                "pool_id": pool_id,
                "token_in": ft.id(),
                "token_out": wnear.id(),
                "amount_in": parse_near!("1 N").to_string(),
                "min_amount_out": "1",
            })],
        }))
        .deposit(1)
        .gas(parse_gas!("100 Tgas") as u64)
        .transact()
        .await?;
    let gas_burnt = actual_out.total_gas_burnt;
    let actual_out: String = actual_out.json()?;
    println!(
        "Actual return for trading in 1 FT token for WNear: {}",
        actual_out
    );
    assert_eq!(actual_out, expected_return);
    println!("Gas burnt from swapping: {}", gas_burnt);

    ///////////////////////////////////////////////////////////////////////////
    // Stage 5: See that our swap tokens reflect in our deposits
    ///////////////////////////////////////////////////////////////////////////

    let ft_deposit: String = worker
        .view(
            ref_finance.id(),
            "get_deposit",
            serde_json::json!({
                "account_id": owner.id(),
                "token_id": ft.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    println!("New FT deposit after swap: {}", ft_deposit);
    assert_eq!(ft_deposit, parse_near!("99 N").to_string());

    let wnear_deposit: String = ref_finance
        .view(
            "get_deposit",
            serde_json::json!({
                "account_id": owner.id(),
                "token_id": wnear.id(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    println!("New WNear deposit after swap: {}", wnear_deposit);

    Ok(())
}
