use near_primitives::views::{CallResult, FinalExecutionOutcomeView, FinalExecutionStatus};

use crate::types::Gas;

/// Struct to hold a type we want to return along w/ the execution result view.
/// This view has extra info about the execution, such as gas usage and whether
/// the transaction failed to be processed on the chain.
pub struct CallExecution<T> {
    pub result: T,
    pub details: CallExecutionDetails,
}

impl<T> CallExecution<T> {
    pub fn unwrap(self) -> T {
        self.into_result().unwrap()
    }

    pub fn into_result(self) -> anyhow::Result<T> {
        Into::<anyhow::Result<_>>::into(self)
    }

    /// Checks whether the transaction was successful. Returns true if
    /// `details.status` is FinalExecutionStatus::Success.
    pub fn is_success(&self) -> bool {
        matches!(self.details.status, FinalExecutionStatus::SuccessValue(_))
    }

    /// Checks whether the transaction has failed. Returns true if
    /// `details.status` is FinalExecutionStatus::Failure.
    pub fn is_failure(&self) -> bool {
        matches!(self.details.status, FinalExecutionStatus::Failure(_))
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
#[non_exhaustive]
pub struct CallExecutionDetails {
    /// Execution status. Contains the result in case of successful execution.
    pub status: FinalExecutionStatus,
    /// Total gas burnt by the call execution
    pub total_gas_burnt: Gas,
}

impl CallExecutionDetails {
    /// Deserialize an instance of type `T` from bytes of JSON text sourced from the
    /// execution result of this call. This conversion can fail if the structure of
    /// the internal state does not meet up with [`serde::de::DeserializeOwned`]'s
    /// requirements.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> {
        let buf = self.try_into_bytes()?;
        serde_json::from_slice(&buf).map_err(Into::into)
    }

    /// Deserialize an instance of type `T` from bytes sourced from the execution
    /// result. This conversion can fail if the structure of the internal state does
    /// not meet up with [`borsh::BorshDeserialize`]'s requirements.
    pub fn borsh<T: borsh::BorshDeserialize>(&self) -> anyhow::Result<T> {
        let buf = self.try_into_bytes()?;
        borsh::BorshDeserialize::try_from_slice(&buf).map_err(Into::into)
    }

    fn try_into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let result: &str = match self.status {
            FinalExecutionStatus::SuccessValue(ref val) => val,
            FinalExecutionStatus::Failure(ref err) => anyhow::bail!(err.clone()),
            FinalExecutionStatus::NotStarted => anyhow::bail!("Transaction not started."),
            FinalExecutionStatus::Started => anyhow::bail!("Transaction still being processed."),
        };
        base64::decode(result).map_err(Into::into)
    }

    /// Convert the execution details into a Result if its status is not a successful one.
    /// Useful for checking if the call was successful and forwarding the error upwards.
    fn try_into_result(self) -> anyhow::Result<Self> {
        match self.status {
            FinalExecutionStatus::Failure(ref err) => anyhow::bail!(err.clone()),
            FinalExecutionStatus::NotStarted => anyhow::bail!("Transaction not started."),
            FinalExecutionStatus::Started => anyhow::bail!("Transaction still being processed."),
            _ => (),
        };
        Ok(self)
    }

    pub(crate) fn from_outcome(outcome: FinalExecutionOutcomeView) -> anyhow::Result<Self> {
        Self::from(outcome).try_into_result()
    }
}

impl From<FinalExecutionOutcomeView> for CallExecutionDetails {
    fn from(transaction_result: FinalExecutionOutcomeView) -> Self {
        CallExecutionDetails {
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

/// The result from a call into a View function. This contains the contents or
/// the results from the view function call itself. The consumer of this object
/// can choose how to deserialize its contents.
#[non_exhaustive]
pub struct ViewResultDetails {
    /// Our result from our call into a view function.
    pub result: Vec<u8>,
    /// Logs generated from the view function.
    pub logs: Vec<String>,
}

impl ViewResultDetails {
    /// Deserialize an instance of type `T` from bytes of JSON text sourced from the
    /// execution result of this call. This conversion can fail if the structure of
    /// the internal state does not meet up with [`serde::de::DeserializeOwned`]'s
    /// requirements.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> {
        serde_json::from_slice(&self.result).map_err(Into::into)
    }

    /// Deserialize an instance of type `T` from bytes sourced from this view call's
    /// result. This conversion can fail if the structure of the internal state does
    /// not meet up with [`borsh::BorshDeserialize`]'s requirements.
    pub fn borsh<T: borsh::BorshDeserialize>(&self) -> anyhow::Result<T> {
        borsh::BorshDeserialize::try_from_slice(&self.result).map_err(Into::into)
    }
}

impl From<CallResult> for ViewResultDetails {
    fn from(result: CallResult) -> Self {
        ViewResultDetails {
            result: result.result,
            logs: result.logs,
        }
    }
}
