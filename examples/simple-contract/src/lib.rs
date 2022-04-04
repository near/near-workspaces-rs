use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen};

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Simple {}

#[near_bindgen]
impl Simple {
    pub fn current_env_data() -> (u64, u64) {
        let now = env::block_timestamp();
        let eh = env::epoch_height();
        log!("Timestamp: {}", now);
        (now, eh)
    }
}
