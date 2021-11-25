use near_primitives::types::Gas;
use near_primitives::views::{FinalExecutionOutcomeView, FinalExecutionStatus};

/// Struct to hold a type we want to return along w/ the execution result view.
/// This view has extra info about the execution, such as gas usage and whether
/// the transaction failed to be processed on the chain.
pub struct CallExecution<T> {
    pub result: T,
    pub details: CallExecutionResult,
}

impl<T> CallExecution<T> {
    pub fn unwrap(self) -> T {
        Into::<anyhow::Result<_>>::into(self).unwrap()
    }
}

impl<T> From<CallExecution<T>> for anyhow::Result<T> {
    fn from(value: CallExecution<T>) -> anyhow::Result<T> {
        match value.details.status {
            FinalExecutionStatus::SuccessValue(_) => Ok(value.result),
            FinalExecutionStatus::Failure(err) => Err(anyhow::anyhow!(err)),
            FinalExecutionStatus::NotStarted => Err(anyhow::anyhow!("Transaction not started.")),
            FinalExecutionStatus::Started => {
                Err(anyhow::anyhow!("Transaction still being processed."))
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CallExecutionResult {
    /// Execution status. Contains the result in case of successful execution.
    pub status: FinalExecutionStatus,
    /// Total gas burnt by the call execution
    pub total_gas_burnt: Gas,
}

impl From<FinalExecutionOutcomeView> for CallExecutionResult {
    fn from(transaction_result: FinalExecutionOutcomeView) -> Self {
        CallExecutionResult {
            status: transaction_result.status,
            total_gas_burnt: transaction_result.transaction_outcome.outcome.gas_burnt
                + transaction_result
                    .receipts_outcome
                    .iter()
                    .map(|t| t.outcome.gas_burnt)
                    .sum::<u64>(),
        }
    }
}
