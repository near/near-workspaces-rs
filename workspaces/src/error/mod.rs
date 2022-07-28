//! All errors that can occur within workspaces, including but not limited to
//! the following: IO, RPC, Execution, Sandbox, DataConversion errors.

mod impls;

use std::borrow::Cow;

/// A list specifying general categories of NEAR workspace error.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An error occurred while performing an RPC request.
    #[error("Rpc({0})")]
    Rpc(#[from] RpcErrorCode),
    /// An error occurred while processing a transaction.
    #[error("Execution")]
    Execution,
    /// An error having to do with running sandbox.
    #[error("Sandbox({0})")]
    Sandbox(#[from] SandboxErrorCode),
    /// An error from performing IO.
    #[error("IO")]
    Io,
    /// An error from converting data.
    #[error("DataConversion")]
    DataConversion,
}

#[derive(Debug, thiserror::Error)]
enum ErrorRepr {
    #[error("{0}")]
    Simple(ErrorKind),
    #[error("{message}")]
    Message {
        kind: ErrorKind,
        message: Cow<'static, str>,
    },
    #[error("{error}")]
    Custom {
        kind: ErrorKind,
        error: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("{message}: {error}")]
    Full {
        kind: ErrorKind,
        message: Cow<'static, str>,
        error: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type that workspaces will make use of for all the errors
/// returned from this library
#[derive(Debug)]
pub struct Error {
    repr: ErrorRepr,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum SandboxErrorCode {
    #[error("Sandbox has already been started")]
    AlreadyStarted,
    #[error("Could not initialize sandbox node")]
    InitFailure,
    #[error("Could not startup and run sandbox node")]
    RunFailure,
    #[error("Sandbox failed to patch state")]
    PatchStateFailure,
    #[error("Sandbox failed to fast forward")]
    FastForwardFailure,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum RpcErrorCode {
    #[error("unable to create a new account via helper")]
    HelperAccountCreationFailure,
    #[error("failed to connect to rpc service")]
    ConnectionFailure,
    #[error("access key was unable to be retrieved")]
    UnableToRetrieveAccessKey,
    #[error("unable to broadcast the transaction to the network")]
    BroadcastTxFailure,
    #[error("unable to call into a view function")]
    ViewFunctionFailure,
    #[error("unable to fulfill the query request")]
    QueryFailure,
    #[error("incorrect variant retrieved while querying (maybe a bug in RPC code?)")]
    QueryReturnedInvalidData,
}
