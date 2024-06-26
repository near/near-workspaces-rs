use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Noop;

#[near_bindgen]
impl Noop {
    pub fn noop() {}
}
