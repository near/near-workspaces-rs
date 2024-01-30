// use std::convert::TryInto;







// /// This part of the example uses the Generation API to generate a client and use it.
// #[path = "gen/adder.rs"]
// mod generation_adder;

// pub async fn generation_run(contract: near_workspaces::Contract) -> anyhow::Result<()> {
//     let contract = generation_adder::AbiClient { contract };
//     let res = contract.add(vec![1, 2], vec![3, 4]).await?;

//     let res = (res[0].try_into().unwrap(), res[1].try_into().unwrap());
//     assert_eq!(res, (4, 6));

//     Ok(())
// }
