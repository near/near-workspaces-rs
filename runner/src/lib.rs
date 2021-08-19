
mod rpc;
mod runtime;

pub use runtime::local::SandboxServer;
pub use runner_macros::test;
pub use rpc::api::*;


#[cfg(test)]
mod tests {
    use super::rpc::api::*;

    #[runner_macros::test(sandbox)]
    async fn test_1() {
        run_().await;
    }
}
