use near_sdk::{env, log, near_bindgen, Promise};

#[near_bindgen]
pub struct Chained;

#[near_bindgen]
impl Chained {
    /// Calls b and uses that as the transaction result. Schedules some irrelevant methods after.
    /// Call chain looks like:
    /// a -> b (execution result) -> c -> c
    pub fn a() -> Promise {
        Self::ext(env::current_account_id()).b()
    }

    #[private]
    pub fn b() -> &'static str {
        Self::ext(env::current_account_id())
            .c()
            .then(Self::ext(env::current_account_id()).c());
        "Test string"
    }

    #[private]
    pub fn c() -> &'static str{
        log!("called c");
        "some other"
    }
}
