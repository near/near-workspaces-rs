use serde_json::json;

const STATUS_MSG_WASM_FILEPATH: &str = "./examples/res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    let outcome = contract
        .call("set_status")
        .args_json(json!({
            "message": "hello_world",
        }))
        .transact()
        .await?;

    let outcome = outcome.outcome();

    // NOTE: this API is under the "experimental" flag and no guarantees are given.
    let resp = worker
        .tx_status(outcome.transaction_hash, outcome.executor_id.clone())
        .await?;

    // Example outcome:
    //
    // FinalExecutionOutcomeWithReceiptView FinalExecutionOutcomeWithReceiptView {
    //     final_outcome: FinalExecutionOutcome {
    //         status: SuccessValue(''),
    //         transaction: SignedTransactionView {
    //             signer_id: AccountId(
    //                 "dev-20230822130616-84011708140052",
    //             ),
    //             public_key: ed25519:5WMgq6gKZbAr7xBZmXJHjnj4C3UZkNJ4F5odisUBFcRh,
    //             nonce: 2000001,
    //             receiver_id: AccountId(
    //                 "dev-20230822130616-84011708140052",
    //             ),
    //             actions: [
    //                 FunctionCall {
    //                     method_name: "set_status",
    //                     args: FunctionArgs(
    //                         [
    //                             123,
    //                             34,
    //                             109,
    //                             101,
    //                             115,
    //                             115,
    //                             97,
    //                             103,
    //                             101,
    //                             34,
    //                             58,
    //                             34,
    //                             104,
    //                             101,
    //                             108,
    //                             108,
    //                             111,
    //                             95,
    //                             119,
    //                             111,
    //                             114,
    //                             108,
    //                             100,
    //                             34,
    //                             125,
    //                         ],
    //                     ),
    //                     gas: 10000000000000,
    //                     deposit: 0,
    //                 },
    //             ],
    //             signature: ed25519:25z5farfJh4ccYoLJeJtKTrfEfecWSXqksafEnFCA724NHaGZWowtpKxGdMZTYSzzpAJ3iT6sWNyEF2oC2u1CvCR,
    //             hash: HWRjprUXTN7fhnvzMaDxXBXbJTVqvbW8j56PvfyL8uB6,
    //         },
    //         transaction_outcome: ExecutionOutcomeWithIdView {
    //             proof: [
    //                 MerklePathItem {
    //                     hash: 8a7iJ6vWjvwHKFLXPETciDbZtdHeHaX6xPqVT42CrEfi,
    //                     direction: Right,
    //                 },
    //             ],
    //             block_hash: BeMy3czUnz7EndbaSSHTSZ7WhozdAxSRAtYYqCo5XSSo,
    //             id: HWRjprUXTN7fhnvzMaDxXBXbJTVqvbW8j56PvfyL8uB6,
    //             outcome: ExecutionOutcomeView {
    //                 logs: [],
    //                 receipt_ids: [
    //                     ByE39xUGnYHrfVsNyxTgDjkPY7yFFLCHQcUb7m5Qiwkp,
    //                 ],
    //                 gas_burnt: 2427999257690,
    //                 tokens_burnt: 242799925769000000000,
    //                 executor_id: AccountId(
    //                     "dev-20230822130616-84011708140052",
    //                 ),
    //                 status: SuccessReceiptId(ByE39xUGnYHrfVsNyxTgDjkPY7yFFLCHQcUb7m5Qiwkp),
    //                 metadata: ExecutionMetadataView {
    //                     version: 1,
    //                     gas_profile: None,
    //                 },
    //             },
    //         },
    //         receipts_outcome: [
    //             ExecutionOutcomeWithIdView {
    //                 proof: [
    //                     MerklePathItem {
    //                         hash: 5hxa61Hv5a82HUh2qWVSVEVB5f2txx4JpmbXK5qmdwnv,
    //                         direction: Left,
    //                     },
    //                 ],
    //                 block_hash: BeMy3czUnz7EndbaSSHTSZ7WhozdAxSRAtYYqCo5XSSo,
    //                 id: ByE39xUGnYHrfVsNyxTgDjkPY7yFFLCHQcUb7m5Qiwkp,
    //                 outcome: ExecutionOutcomeView {
    //                     logs: [
    //                         "A",
    //                     ],
    //                     receipt_ids: [
    //                         AhbLBVzfPa2ebNkTtdTyK1pB6zE6Srucwob32UCNFS5E,
    //                     ],
    //                     gas_burnt: 2666186302694,
    //                     tokens_burnt: 266618630269400000000,
    //                     executor_id: AccountId(
    //                         "dev-20230822130616-84011708140052",
    //                     ),
    //                     status: SuccessValue(''),
    //                     metadata: ExecutionMetadataView {
    //                         version: 3,
    //                         gas_profile: Some(
    //                             [
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "BASE",
    //                                     gas_used: 2647681110,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "CONTRACT_LOADING_BASE",
    //                                     gas_used: 35445963,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "CONTRACT_LOADING_BYTES",
    //                                     gas_used: 26988192750,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "LOG_BASE",
    //                                     gas_used: 3543313050,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "LOG_BYTE",
    //                                     gas_used: 13198791,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "READ_CACHED_TRIE_NODE",
    //                                     gas_used: 4560000000,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "READ_MEMORY_BASE",
    //                                     gas_used: 10439452800,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "READ_MEMORY_BYTE",
    //                                     gas_used: 254689311,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "READ_REGISTER_BASE",
    //                                     gas_used: 5034330372,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "READ_REGISTER_BYTE",
    //                                     gas_used: 5716596,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "STORAGE_READ_BASE",
    //                                     gas_used: 56356845750,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "STORAGE_READ_KEY_BYTE",
    //                                     gas_used: 154762665,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "STORAGE_WRITE_BASE",
    //                                     gas_used: 64196736000,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "STORAGE_WRITE_KEY_BYTE",
    //                                     gas_used: 352414335,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "STORAGE_WRITE_VALUE_BYTE",
    //                                     gas_used: 1737038184,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "TOUCHING_TRIE_NODE",
    //                                     gas_used: 32203911852,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "UTF8_DECODING_BASE",
    //                                     gas_used: 3111779061,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "UTF8_DECODING_BYTE",
    //                                     gas_used: 291580479,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "WASM_INSTRUCTION",
    //                                     gas_used: 11695476540,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "WRITE_MEMORY_BASE",
    //                                     gas_used: 8411384583,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "WRITE_MEMORY_BYTE",
    //                                     gas_used: 201559128,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "WRITE_REGISTER_BASE",
    //                                     gas_used: 5731044972,
    //                                 },
    //                                 CostGasUsed {
    //                                     cost_category: "WASM_HOST_COST",
    //                                     cost: "WRITE_REGISTER_BYTE",
    //                                     gas_used: 220490712,
    //                                 },
    //                             ],
    //                         ),
    //                     },
    //                 },
    //             },
    //             ExecutionOutcomeWithIdView {
    //                 proof: [],
    //                 block_hash: AG5nJGsWxCtAHPzq3m7NsTnSdSmQaxeLwffvFKMeWT9b,
    //                 id: AhbLBVzfPa2ebNkTtdTyK1pB6zE6Srucwob32UCNFS5E,
    //                 outcome: ExecutionOutcomeView {
    //                     logs: [],
    //                     receipt_ids: [],
    //                     gas_burnt: 223182562500,
    //                     tokens_burnt: 0,
    //                     executor_id: AccountId(
    //                         "dev-20230822130616-84011708140052",
    //                     ),
    //                     status: SuccessValue(''),
    //                     metadata: ExecutionMetadataView {
    //                         version: 3,
    //                         gas_profile: Some(
    //                             [],
    //                         ),
    //                     },
    //                 },
    //             },
    //         ],
    //     },
    //     receipts: [
    //         ReceiptView {
    //             predecessor_id: AccountId(
    //                 "system",
    //             ),
    //             receiver_id: AccountId(
    //                 "dev-20230822130616-84011708140052",
    //             ),
    //             receipt_id: AhbLBVzfPa2ebNkTtdTyK1pB6zE6Srucwob32UCNFS5E,
    //             receipt: Action {
    //                 signer_id: AccountId(
    //                     "dev-20230822130616-84011708140052",
    //                 ),
    //                 signer_public_key: ed25519:5WMgq6gKZbAr7xBZmXJHjnj4C3UZkNJ4F5odisUBFcRh,
    //                 gas_price: 0,
    //                 output_data_receivers: [],
    //                 input_data_ids: [],
    //                 actions: [
    //                     Transfer {
    //                         deposit: 1051867810978932100000,
    //                     },
    //                 ],
    //             },
    //         },
    //     ],
    // }
    println!("FinalExecutionOutcomeWithReceiptView {resp:#?}");
    Ok(())
}
