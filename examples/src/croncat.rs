use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use near_sdk::json_types::U128;
use near_units::parse_gas;
use near_units::parse_near;
use serde::{Deserialize, Serialize};
use serde_json::json;

use workspaces::network::Sandbox;
use workspaces::prelude::*;
use workspaces::Account;
use workspaces::AccountId;
use workspaces::Contract;
use workspaces::Worker;

const MANAGER_CONTRACT: &[u8] = include_bytes!("../res/manager.wasm");
const COUNTER_CONTRACT: &[u8] = include_bytes!("../res/counter.wasm");

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum AgentStatus {
    Active,
    Pending,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Agent {
    pub status: AgentStatus,
    pub payable_account_id: AccountId,
    pub balance: U128,
    pub total_tasks_executed: U128,
    pub last_missed_slot: u128,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // Initialize counter contract, which will be pointed to in the manager contract to schedule
    // a task later to increment the counter, inside counter contract.
    let counter_contract = worker.dev_deploy(COUNTER_CONTRACT).await?;

    let manager_contract = worker.dev_deploy(MANAGER_CONTRACT).await?;
    manager_contract.call(&worker, "new").transact().await?;

    let croncat = worker.dev_create_account().await?;
    let agent_1 = croncat
        .create_subaccount(&worker, "agent_1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    // With all the setup work done, we can try to schedule the `counter.increment` contract
    // operation to happen about once every hour via the `cadence` paramter
    run_scheduling(&worker, &manager_contract, &agent_1, counter_contract.id()).await?;

    Ok(())
}

pub async fn run_scheduling(
    worker: &Worker<Sandbox>,
    contract: &Contract,
    agent: &Account,
    counter_id: &AccountId,
) -> anyhow::Result<()> {
    println!("Creating task for `counter.increment`");
    let outcome = agent
        .call(&worker, contract.id(), "create_task")
        .args_json(json!({
            "contract_id": counter_id,
            "function_id": "increment",
            "cadence": "*/1 * * * * *"
        }))?
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    println!("-- outcome: {:#?}\n", outcome);

    // register the agent to execute
    let outcome = agent
        .call(&worker, contract.id(), "register_agent")
        .args_json(json!({}))?
        .deposit(parse_near!("0.00226 N"))
        .transact()
        .await?;
    println!("Registering agent outcome: {:#?}\n", outcome);

    // Check the right agent was registered correctly:
    let registered_agent = contract
        .call(&worker, "get_agent")
        .args_json(json!({ "account_id": agent.id() }))?
        .view()
        .await?
        .json::<Option<Agent>>()?
        .unwrap();
    println!("Registered agent: {:#?}", registered_agent);
    assert_eq!(registered_agent.status, AgentStatus::Active);
    assert_eq!(&registered_agent.payable_account_id, agent.id());

    // Advance 4500 blocks in the chain. 1 block takes approx 1.5 seconds to be produced, but we
    // don't actually wait that long since we are time travelling to the future via `fast_forward`!
    println!("Waiting until next slot occurs...");
    worker.fast_forward(4500).await?;

    // TODO:
    // Quick proxy call to earn a reward
    agent
        .call(&worker, contract.id(), "proxy_call")
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Check accumulated agent balance
    let bal_agent = contract
        .call(&worker, "get_agent")
        .args_json(json!({"account_id": agent.id()}))?
        .view()
        .await?
        .json::<Option<Agent>>()?
        .unwrap();
    println!("Agent details: {:#?}", bal_agent);
    assert_eq!(bal_agent.balance.0, parse_near!("0.00306 N"));

    // Withdraw reward
    agent
        .call(&worker, contract.id(), "withdraw_task_balance")
        .transact()
        .await?;

    // Check accumulated agent balance
    let bal_done_agent: Option<Agent> = contract
        .call(&worker, "get_agent")
        .args_json(json!({"account_id": agent.id() }))?
        .view()
        .await?
        .json()?;
    println!("bal_done_agent {:#?}", bal_done_agent);
    assert!(bal_done_agent.is_some());
    assert_eq!(bal_done_agent.unwrap().balance.0, parse_near!("0.00226 N"));

    // Unregister the agent from doing anything
    agent
        .call(&worker, contract.id(), "unregister_agent")
        .deposit(parse_near!("1y"))
        .transact()
        .await?;

    // Check to see if the agent has been successfully unregistered
    let removed_agent: Option<Agent> = contract
        .call(&worker, "get_agent")
        .args_json(json!({"account_id": agent.id() }))?
        .view()
        .await?
        .json()?;
    assert!(
        removed_agent.is_none(),
        "Agent should have been removed via `unregister_agent`"
    );

    println!(
        "Balance after completing tasks: {:?}",
        worker.view_account(agent.id()).await?
    );

    Ok(())
}
