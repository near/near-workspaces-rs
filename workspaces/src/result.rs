//! Result and execution types from results of RPC calls to the network.

use near_account_id::AccountId;
use near_primitives::views::{
    CallResult, ExecutionOutcomeWithIdView, ExecutionStatusView, FinalExecutionOutcomeView,
    FinalExecutionStatus,
};

use crate::error::{Error, SerializationError};
use crate::types::{CryptoHash, Gas};

pub type Result<T, E = crate::error::Error> = core::result::Result<T, E>;

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

    pub fn into_result(self) -> Result<T> {
        Into::<Result<_>>::into(self)
    }

    /// Checks whether the transaction was successful. Returns true if
    /// the transaction has a status of FinalExecutionStatus::Success.
    pub fn is_success(&self) -> bool {
        self.details.is_success()
    }

    /// Checks whether the transaction has failed. Returns true if
    /// the transaction has a status of FinalExecutionStatus::Failure.
    pub fn is_failure(&self) -> bool {
        self.details.is_failure()
    }
}

impl<T> From<CallExecution<T>> for Result<T> {
    fn from(value: CallExecution<T>) -> Result<T> {
        value.details.try_into_result()?;
        Ok(value.result)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[non_exhaustive]
pub struct CallExecutionDetails {
    /// Execution status. Contains the result in case of successful execution.
    pub(crate) status: FinalExecutionStatus,
    /// Total gas burnt by the call execution
    pub total_gas_burnt: Gas,

    pub(crate) transaction: ExecutionOutcome,
    pub(crate) receipts: Vec<ExecutionOutcome>,
}

impl CallExecutionDetails {
    /// Deserialize an instance of type `T` from bytes of JSON text sourced from the
    /// execution result of this call. This conversion can fail if the structure of
    /// the internal state does not meet up with [`serde::de::DeserializeOwned`]'s
    /// requirements.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let buf = self.raw_bytes()?;
        serde_json::from_slice(&buf)
            .map_err(SerializationError::SerdeError)
            .map_err(Into::into)
    }

    /// Deserialize an instance of type `T` from bytes sourced from the execution
    /// result. This conversion can fail if the structure of the internal state does
    /// not meet up with [`borsh::BorshDeserialize`]'s requirements.
    pub fn borsh<T: borsh::BorshDeserialize>(&self) -> Result<T> {
        let buf = self.raw_bytes()?;
        borsh::BorshDeserialize::try_from_slice(&buf)
            .map_err(SerializationError::BorshError)
            .map_err(Into::into)
    }

    /// Grab the underlying raw bytes returned from calling into a contract's function.
    /// If we want to deserialize these bytes into a rust datatype, use [`CallExecutionDetails::json`]
    /// or [`CallExecutionDetails::borsh`] instead.
    pub fn raw_bytes(&self) -> Result<Vec<u8>> {
        let result = self.try_into_success_value()?;
        base64::decode(result)
            .map_err(SerializationError::DecodeBase64Error)
            .map_err(Into::into)
    }

    fn try_into_success_value(&self) -> Result<&str> {
        match self.status {
            FinalExecutionStatus::SuccessValue(ref val) => Ok(val),
            FinalExecutionStatus::Failure(ref err) => Err(Error::ExecutionError(err.to_string())),
            FinalExecutionStatus::NotStarted => {
                Err(Error::ExecutionError("Transaction not started.".into()))
            }
            FinalExecutionStatus::Started => Err(Error::ExecutionError(
                "Transaction still being processed.".into(),
            )),
        }
    }

    /// Convert the execution details into a Result if its status is not a successful one.
    /// Useful for checking if the call was successful and forwarding the error upwards.
    fn try_into_result(self) -> Result<Self> {
        self.try_into_success_value()?;
        Ok(self)
    }

    pub(crate) fn from_outcome(outcome: FinalExecutionOutcomeView) -> Result<Self> {
        Self::from(outcome).try_into_result()
    }

    /// Returns just the transaction outcome.
    pub fn outcome(&self) -> &ExecutionOutcome {
        &self.transaction
    }

    /// Grab all outcomes after the execution of the transaction. This includes outcomes
    /// from the transaction and all the receipts it generated.
    pub fn outcomes(&self) -> Vec<&ExecutionOutcome> {
        let mut outcomes = vec![&self.transaction];
        outcomes.extend(self.receipt_outcomes());
        outcomes
    }

    /// Grab all outcomes after the execution of the transaction. This includes outcomes
    /// only from receipts generated by this transaction.
    pub fn receipt_outcomes(&self) -> &[ExecutionOutcome] {
        &self.receipts
    }

    /// Grab all outcomes that did not succeed the execution of this transaction. This
    /// will also include the failures from receipts as well.
    pub fn failures(&self) -> Vec<&ExecutionOutcome> {
        let mut failures = Vec::new();
        if matches!(self.transaction.status, ExecutionStatusView::Failure(_)) {
            failures.push(&self.transaction);
        }
        failures.extend(self.receipt_failures());
        failures
    }

    /// Just like `failures`, grab only failed receipt outcomes.
    pub fn receipt_failures(&self) -> Vec<&ExecutionOutcome> {
        self.receipts
            .iter()
            .filter(|receipt| matches!(receipt.status, ExecutionStatusView::Failure(_)))
            .collect()
    }

    /// Checks whether the transaction was successful. Returns true if
    /// the transaction has a status of [`FinalExecutionStatus::SuccessValue`].
    pub fn is_success(&self) -> bool {
        matches!(self.status, FinalExecutionStatus::SuccessValue(_))
    }

    /// Checks whether the transaction has failed. Returns true if
    /// the transaction has a status of [`FinalExecutionStatus::Failure`].
    pub fn is_failure(&self) -> bool {
        matches!(self.status, FinalExecutionStatus::Failure(_))
    }

    /// Grab all logs from both the transaction and receipt outcomes.
    pub fn logs(&self) -> Vec<&str> {
        self.outcomes()
            .iter()
            .flat_map(|outcome| &outcome.logs)
            .map(String::as_str)
            .collect()
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
            transaction: transaction_result.transaction_outcome.into(),
            receipts: transaction_result
                .receipts_outcome
                .into_iter()
                .map(ExecutionOutcome::from)
                .collect(),
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
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.result)
            .map_err(SerializationError::SerdeError)
            .map_err(Into::into)
    }

    /// Deserialize an instance of type `T` from bytes sourced from this view call's
    /// result. This conversion can fail if the structure of the internal state does
    /// not meet up with [`borsh::BorshDeserialize`]'s requirements.
    pub fn borsh<T: borsh::BorshDeserialize>(&self) -> Result<T> {
        borsh::BorshDeserialize::try_from_slice(&self.result)
            .map_err(SerializationError::BorshError)
            .map_err(Into::into)
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

/// The execution outcome of a transaction. This type contains all data relevant to
/// calling into a function, and getting the results back.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExecutionOutcome {
    pub block_hash: CryptoHash,
    /// Logs from this transaction or receipt.
    pub logs: Vec<String>,
    /// Receipt IDs generated by this transaction or receipt.
    pub receipt_ids: Vec<CryptoHash>,
    /// The amount of the gas burnt by the given transaction or receipt.
    pub gas_burnt: Gas,
    /// The id of the account on which the execution happens. For transaction this is signer_id,
    /// for receipt this is receiver_id.
    pub executor_id: AccountId,
    /// Execution status. Contains the result in case of successful execution.
    pub(crate) status: ExecutionStatusView,
}

impl ExecutionOutcome {
    /// Checks whether this execution outcome was a success. Returns true if a success value or
    /// receipt id is present.
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            ExecutionStatusView::SuccessValue(_) | ExecutionStatusView::SuccessReceiptId(_)
        )
    }

    /// Checks whether this execution outcome was a failure. Returns true if it failed with
    /// an error or the execution state was unknown or pending.
    pub fn is_failure(&self) -> bool {
        matches!(
            self.status,
            ExecutionStatusView::Failure(_) | ExecutionStatusView::Unknown
        )
    }

    /// Converts this [`ExecutionOutcome`] into a Result type to match against whether the
    /// particular outcome has failed or not.
    pub fn into_result(self) -> Result<ValueOrReceiptId> {
        match self.status {
            ExecutionStatusView::SuccessValue(value) => Ok(ValueOrReceiptId::Value(value)),
            ExecutionStatusView::SuccessReceiptId(hash) => {
                Ok(ValueOrReceiptId::ReceiptId(CryptoHash(hash.0)))
            }
            ExecutionStatusView::Failure(err) => Err(Error::ExecutionError(err.to_string())),
            ExecutionStatusView::Unknown => Err(Error::ExecutionError(
                "Execution pending or unknown".to_string(),
            )),
        }
    }
}

/// Value or ReceiptId from a successful execution.
pub enum ValueOrReceiptId {
    /// The final action succeeded and returned some value or an empty vec encoded in base64.
    Value(String),
    /// The final action of the receipt returned a promise or the signed transaction was converted
    /// to a receipt. Contains the receipt_id of the generated receipt.
    ReceiptId(CryptoHash),
}

impl From<ExecutionOutcomeWithIdView> for ExecutionOutcome {
    fn from(view: ExecutionOutcomeWithIdView) -> Self {
        ExecutionOutcome {
            block_hash: CryptoHash(view.block_hash.0),
            logs: view.outcome.logs,
            receipt_ids: view
                .outcome
                .receipt_ids
                .into_iter()
                .map(|c| CryptoHash(c.0))
                .collect(),
            gas_burnt: view.outcome.gas_burnt,
            executor_id: view.outcome.executor_id,
            status: view.outcome.status,
        }
    }
}
