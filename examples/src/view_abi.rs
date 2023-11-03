use std::convert::TryInto;

const ADDER_WASM_FILEPATH: &str = "./examples/res/adder.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(ADDER_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    macro_run(contract.clone()).await?;
    generation_run(contract).await?;

    Ok(())
}

/// This part of the example uses the Macro API to get a client and use it.
mod macro_adder {
    near_workspaces::near_abi_client::generate!(Client for "../examples/res/adder.json");
}

pub async fn macro_run(contract: near_workspaces::Contract) -> anyhow::Result<()> {
    let contract = macro_adder::Client { contract };
    let res = contract.add(vec![1, 2], vec![3, 4]).await?;

    let res = (res[0].try_into().unwrap(), res[1].try_into().unwrap());
    assert_eq!(res, (4, 6));

    Ok(())
}

/// This part of the example uses the Generation API to generate a client and use it.
#[path = "gen/adder.rs"]
mod generation_adder;

pub async fn generation_run(contract: near_workspaces::Contract) -> anyhow::Result<()> {
    let contract = generation_adder::AbiClient { contract };
    let res = contract.add(vec![1, 2], vec![3, 4]).await?;

    let res = (res[0].try_into().unwrap(), res[1].try_into().unwrap());
    assert_eq!(res, (4, 6));

    Ok(())
}
