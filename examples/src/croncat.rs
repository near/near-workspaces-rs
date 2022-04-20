use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use near_sdk::Gas;
use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_units::parse_gas;
use near_units::parse_near;
use serde_json::json;
use serde::{Deserialize, Serialize};

use workspaces::Account;
use workspaces::AccountId;
use workspaces::Contract;
use workspaces::Worker;
use workspaces::network::Sandbox;
use workspaces::prelude::*;

const MANAGER_CONTRACT: &[u8] = include_bytes!("../res/manager.wasm");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let manager_contract = worker.dev_deploy(MANAGER_CONTRACT).await?;
    let croncat = worker.dev_create_account().await?;
    let agent_1 = croncat.create_subaccount(&worker, "agent_1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    lifecycle(&worker, &manager_contract, &agent_1).await?;

    Ok(())
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Task {
    pub owner_id: AccountId,
    pub contract_id: AccountId,
    pub function_id: String,
    pub cadence: String,
    pub recurring: bool,
    pub total_deposit: U128,
    pub deposit: U128,
    pub gas: Gas,
    pub arguments: Base64VecU8,
}

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

pub async fn lifecycle(
    worker: &Worker<Sandbox>,
    contract: &Contract,
    agent: &Account,
) -> anyhow::Result<()> {
    println!("Creating task");
    let outcome = agent
        .call(&worker, contract.id(), "create_task")
        .args_json(json!({
            "contract_id": "counter.examples.near",
            "function_id": "increment",
            "cadence": "*/1 * * * * *"
        }))?
        // .gas(parse_gas!("200 Tgas") as u64)
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    println!("-- outcome: {:?}", outcome);

    let no_agent: Option<Agent> = worker
        .view(
            contract.id(),
            "get_agent",
            json!({"account_id": agent.id() }).to_string().into_bytes(),
        )
        .await?
        .json()?;
    println!("no_agent {:#?}", no_agent);

    // register
    let outcome = agent
        .call(&worker, contract.id(), "register_agent")
        .deposit(parse_near!("0.00226 N"))
        .transact()
        .await?;
    println!("Registering agent outcome: {:?}", outcome);

    // check the right agent was registered
    let new_agent: Option<Agent> = worker
        .view(
            contract.id(),
            "get_agent",
            json!({"account_id": agent.id() }).to_string().into_bytes(),
        )
        .await?
        .json()?;
    // println!("new_agent {:#?}", new_agent);
    assert!(new_agent.is_some());
    let new_agent_data = new_agent.unwrap();
    assert_eq!(new_agent_data.status, AgentStatus::Active);
    assert_eq!(new_agent_data.payable_account_id.to_string(), agent.id().clone().to_string());

    // Check we cannot register again
    let fail_agent = agent
        .call(&worker, contract.id(), "register_agent")
        .args_json(json!({}))?
        .deposit(parse_near!("0.00226 N"))
        .transact()
        .await?;
    // println!("agent fail_agent {:#?}", fail_agent.status);
    // let fail_agent_bool: bool = match fail_agent.status {
    //     // match it against a specific error
    //     FinalExecutionStatus::Failure(e) => {
    //         e.to_string().contains("Agent already exists")
    //     },
    //     _ => false,
    // };
    // assert_eq!(fail_agent_bool, true);

    // NOTE: Once fastfwd is possible, we can remove this
    println!("Waiting until next slot occurs...");
    // pause for blocks to clear
    // thread::sleep(time::Duration::from_millis(65_000));
    worker.fast_forward(4500).await?;

    // quick proxy call to earn a reward
    agent
        .call(&worker, contract.id(), "proxy_call")
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // check accumulated agent balance
    let bal_agent: Option<Agent> = worker
        .view(
            contract.id(),
            "get_agent",
            json!({"account_id": agent.id().clone() }).to_string().into_bytes(),
        )
        .await?
        .json()?;
    // println!("bal_agent {:#?}", bal_agent);
    assert!(bal_agent.is_some());
    assert_eq!(bal_agent.unwrap().balance.0, parse_near!("0.00306 N"));

    // withdraw reward
    agent
        .call(&worker, contract.id(), "withdraw_task_balance")
        .transact()
        .await?;

    // check accumulated agent balance
    let bal_done_agent: Option<Agent> = worker
        .view(
            contract.id(),
            "get_agent",
            json!({"account_id": agent.id() }).to_string().into_bytes(),
        )
        .await?
        .json()?;
    // println!("bal_done_agent {:#?}", bal_done_agent);
    assert!(bal_done_agent.is_some());
    assert_eq!(bal_done_agent.unwrap().balance.0, parse_near!("0.00226 N"));

    // unregister agent
    agent
        .call(&worker, contract.id(), "unregister_agent")
        .deposit(parse_near!("1y"))
        .transact()
        .await?;

    let removed_agent: Option<Agent> = worker
        .view(
            contract.id(),
            "get_agent",
            json!({"account_id": agent.id() }).to_string().into_bytes(),
        )
        .await?
        .json()?;
    // println!("removed_agent {:#?}", removed_agent);
    assert!(removed_agent.is_none());

    // try to unregister agent again, check it fails
    let fail_unregister = agent
        .call(&worker, contract.id(), "unregister_agent")
        .deposit(parse_near!("1y"))
        .transact()
        .await?;
    // println!("agent fail_unregister {:#?}", fail_unregister.status);
    // TODO: get the error to trigger, not sure why state is not working
    // let fail_unregister_bool: bool = match fail_unregister.status {
    //     // match it against a specific error
    //     FinalExecutionStatus::Failure(e) => {
    //         // println!("{:?}", e);
    //         // e.to_string().contains("Agent already exists");
    //         true
    //     },
    //     FinalExecutionStatus::SuccessValue(e) => {
    //         // println!("SUS {:?}", e);
    //         // e.to_string().contains("Agent already exists");
    //         true
    //     },
    //     _ => false,
    // };
    // assert_eq!(fail_unregister_bool, true);

    Ok(())
}