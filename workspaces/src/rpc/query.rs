use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::future::IntoFuture;

use futures::future::{BoxFuture, TryFutureExt};

use near_account_id::AccountId;
use near_jsonrpc_client::methods::query::RpcQueryResponse;
use near_jsonrpc_client::methods::{self, RpcMethod};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockId, BlockReference, StoreKey};
use near_primitives::views::{BlockView, QueryRequest};

use crate::error::RpcErrorCode;
use crate::operations::Function;
use crate::result::ViewResultDetails;
use crate::rpc::client::Client;
use crate::types::{AccessKey, AccessKeyInfo, BlockHeight, PublicKey};
use crate::{AccountDetails, Block, CryptoHash, Result};

use super::tool;

pub struct Query<'a, T> {
    pub(crate) client: &'a Client,
    pub(crate) block_ref: Option<BlockReference>,
    pub(crate) method: T,
}

// impl<'a, T> Query<'a, T> {
//     fn new(client: &'a Client, )
// }

impl<'a, T> Query<'a, T> {
    /// Specify at which block height to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_height(mut self, height: BlockHeight) -> Self {
        self.block_ref = Some(BlockId::Height(height).into());
        self
    }

    /// Specify at which block hash to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_hash(mut self, hash: CryptoHash) -> Self {
        self.block_ref = Some(BlockId::Hash(near_primitives::hash::CryptoHash(hash.0)).into());
        self
    }
}

impl<'a, T, R> IntoFuture for Query<'a, T>
where
    T: Queryable<Output = R> + 'static,
    <T as Queryable>::QueryMethod: RpcMethod + Debug + Send + Sync,
    <<T as Queryable>::QueryMethod as RpcMethod>::Response: Debug + Send + Sync,
    <<T as Queryable>::QueryMethod as RpcMethod>::Error: Debug + Display + Send + Sync,
{
    type Output = Result<R>;

    // TODO: boxed future required due to impl Trait as type alias being unstable. So once
    // https://github.com/rust-lang/rust/issues/63063 is resolved, we can move to that instead.
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let block_reference = self.block_ref.unwrap_or_else(BlockReference::latest);
        let fut = self
            .client
            // query returns Future<Output = Result<Value, JsonRpcError>>
            .query(self.method.into_query_request(block_reference))
            // map the err to workspaces type: Future<Output = Result<Value, WorkspacesError>>
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e))
            // map the val to workspaces type: Future<Output = Result<WorkspacesValue, WorkspacesError>>
            .and_then(|resp| async move { T::process_response(resp) });

        Box::pin(fut)
    }
}

/// Trait used as a converter from WorkspaceRequest to near-rpc request, and
/// from near-rpc response to a WorkspaceResult
pub trait Queryable {
    // TODO: associated default type is unstable. So for now, will require writing
    // the manual impls for query_request
    type QueryMethod: RpcMethod;

    /// Expected output after performing a query. This is mainly to convert over
    /// the type from near-primitives to a workspace type.
    type Output;

    fn into_query_request(self, block_ref: BlockReference) -> Self::QueryMethod;
    fn process_response(query: <Self::QueryMethod as RpcMethod>::Response) -> Result<Self::Output>;
    // fn process_response(query: RpcQueryResponse) -> Self::Output;
}

pub(crate) struct ViewFunction {
    account_id: AccountId,
    function: Function<'static>,
}

pub(crate) struct ViewCode {
    pub(crate) account_id: AccountId,
}

pub(crate) struct ViewAccount {
    pub(crate) account_id: AccountId,
}

pub(crate) struct ViewBlock;

pub(crate) struct ViewState {
    account_id: AccountId,
    prefix: Option<Vec<u8>>,
}

pub(crate) struct ViewAccessKey {
    account_id: AccountId,
    public_key: PublicKey,
}

pub(crate) struct ViewAccessKeyList {
    account_id: AccountId,
}

impl Queryable for ViewFunction {
    type QueryMethod = methods::query::RpcQueryRequest;
    type Output = ViewResultDetails;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::CallFunction {
                account_id: self.account_id,
                method_name: self.function.name.into(),
                // TODO: result
                args: self.function.args.unwrap().into(),
            },
        }
    }

    fn process_response(query: RpcQueryResponse) -> Result<Self::Output> {
        match query.kind {
            QueryResponseKind::CallResult(result) => Ok(result.into()),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying account")),
        }
    }
}

impl Queryable for ViewCode {
    type QueryMethod = methods::query::RpcQueryRequest;
    type Output = Vec<u8>;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewCode {
                account_id: self.account_id,
            },
        }
    }

    fn process_response(query: RpcQueryResponse) -> Result<Self::Output> {
        match query.kind {
            QueryResponseKind::ViewCode(contract) => Ok(contract.code),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying account")),
        }
    }
}

impl Queryable for ViewAccount {
    type QueryMethod = methods::query::RpcQueryRequest;
    type Output = AccountDetails;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewAccount {
                account_id: self.account_id,
            },
        }
    }

    fn process_response(query: RpcQueryResponse) -> Result<Self::Output> {
        match query.kind {
            QueryResponseKind::ViewAccount(account) => Ok(account.into()),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying account")),
        }
    }
}

impl Queryable for ViewBlock {
    type QueryMethod = methods::block::RpcBlockRequest;
    type Output = Block;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod { block_reference }
    }

    fn process_response(view: BlockView) -> Result<Self::Output> {
        Ok(view.into())
    }
}

impl Queryable for ViewState {
    type QueryMethod = methods::query::RpcQueryRequest;
    type Output = HashMap<Vec<u8>, Vec<u8>>;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewState {
                account_id: self.account_id,
                prefix: StoreKey::from(self.prefix.map(Vec::from).unwrap_or_default()),
            },
        }
    }

    fn process_response(query: <Self::QueryMethod as RpcMethod>::Response) -> Result<Self::Output> {
        match query.kind {
            QueryResponseKind::ViewState(state) => Ok(tool::into_state_map(&state.values)?),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying state")),
        }
    }
}

impl Queryable for ViewAccessKey {
    type QueryMethod = methods::query::RpcQueryRequest;
    type Output = AccessKey;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewAccessKey {
                account_id: self.account_id,
                public_key: self.public_key.into(),
            },
        }
    }

    fn process_response(query: <Self::QueryMethod as RpcMethod>::Response) -> Result<Self::Output> {
        match query.kind {
            QueryResponseKind::AccessKey(key) => Ok(key.into()),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying state")),
        }
    }
}

impl Queryable for ViewAccessKeyList {
    type QueryMethod = methods::query::RpcQueryRequest;
    type Output = Vec<AccessKeyInfo>;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewAccessKeyList {
                account_id: self.account_id,
            },
        }
    }

    fn process_response(query: <Self::QueryMethod as RpcMethod>::Response) -> Result<Self::Output> {
        match query.kind {
            QueryResponseKind::AccessKeyList(keylist) => {
                Ok(keylist.keys.into_iter().map(Into::into).collect())
            }
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying state")),
        }
    }
}
