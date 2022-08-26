// fn try_into_success_value(&self) -> Result<&str> {
//     match self.status {
//         FinalExecutionStatus::SuccessValue(ref val) => Ok(val),
//         FinalExecutionStatus::Failure(ref err) => {
//             Err(ErrorKind::Execution.detailed(self.clone(), err.clone()))
//         }
//         FinalExecutionStatus::NotStarted => {
//             Err(ErrorKind::Execution.message("Transaction not started."))
//         }
//         FinalExecutionStatus::Started => {
//             Err(ErrorKind::Execution.message("Transaction still being processed."))
//         }
//     }
// }

// struct ExecutionFailure {
//     /// Total gas burnt by the failed execution
//     pub total_gas_burnt: Gas,

//     tx_error: TxExecutionError,
//     transaction: ExecutionOutcome,
//     receipts: Vec<ExecutionOutcome>,
// }

use std::fmt;

use crate::error::{Error, ErrorKind};
use crate::result::ExecutionFailure;

impl From<ExecutionFailure> for Error {
    fn from(error: ExecutionFailure) -> Self {
        ErrorKind::Execution.custom(error)
    }
}

impl std::error::Error for ExecutionFailure {}

impl fmt::Display for ExecutionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
