use std::env;

use tracing::metadata::LevelFilter;
use tracing_subscriber::EnvFilter;
use workspaces::prelude::*;
use workspaces::AccountId;

const PATH: &str = "./examples/res/cross_contract_high_level.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = if env::var(EnvFilter::DEFAULT_ENV).is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::default().add_directive(LevelFilter::INFO.into())
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let worker = workspaces::sandbox();
    let root = worker.root_account();
    let contract = worker.dev_deploy(&std::fs::read(PATH)?).await?;
    let status_id: AccountId = format!("status.{}", contract.id()).parse().unwrap();
    let status_init_bal = near_units::parse_near!("2 N");

    let welp = root
        .call(&worker, contract.id(), "deploy_status_message")
        .args_json((status_id.clone(), status_init_bal.to_string()))?
        .deposit(50_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("---> {:?}", welp);

    let value: String = root
        .call(&worker, contract.id(), "complex_call")
        .args_json((status_id.clone(), "hello world"))?
        .gas(300_000_000_000_000)
        .transact()
        .await?
        .json()?;

    println!("---> {}", value);

    // let value = res.unwrap_json_value();
    // assert_eq!(message, value.to_string().trim_matches(|c| c == '"'));

    let v1: Vec<u8> = vec![42];
    let _v: Vec<u8> = vec![7, 1, 6, 5, 9, 255, 100, 11];
    // call!(master_account, contract.merge_sort(v1)).assert_success();
    root.call(&worker, contract.id(), "merge_sort")
        .args_json((v1,))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;

    // let res = call!(master_account, contract.merge_sort(_v.clone()), gas = DEFAULT_GAS * 500);
    let array = root
        .call(&worker, contract.id(), "merge_sort")
        .args_json((_v,))?
        .gas(300_000_000_000_000 * 500)
        .transact()
        .await?
        .borsh::<Vec<u8>>()?;

    // let arr = res.unwrap_borsh::<Vec<u8>>();
    let (_last, b) = array
        .iter()
        .fold((0u8, true), |(prev, b), curr| (*curr, prev <= *curr && b));
    println!(">>> {}", b);
    assert!(b, "array is not sorted.");
    Ok(())
}
