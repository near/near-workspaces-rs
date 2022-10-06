use std::future::{ready, Future, IntoFuture, Ready};
use std::marker::PhantomData;

use futures::future::AndThen;
use futures::future::BoxFuture;
use futures::future::TryFutureExt;
// use futures::TryFuture;

use near_account_id::AccountId;
use near_jsonrpc_client::errors::JsonRpcError;
use near_jsonrpc_client::methods::query::{RpcQueryError, RpcQueryRequest, RpcQueryResponse};
use near_jsonrpc_client::methods::{self, RpcMethod};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockId, BlockReference};
use near_primitives::views::QueryRequest;

use crate::error::{Error, RpcErrorCode};
use crate::rpc::client::Client;
use crate::types::BlockHeight;
use crate::{AccountDetails, CryptoHash, Result};

pub struct Query<'a, T> {
    pub(crate) client: &'a Client,
    pub(crate) request: near_primitives::views::QueryRequest,
    pub(crate) block_ref: Option<BlockReference>,
    // pub(crate) method: T,
    pub(crate) _data: PhantomData<T>,
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
    <T as Queryable>::QueryMethod: RpcMethod + std::fmt::Debug,
    <<T as Queryable>::QueryMethod as RpcMethod>::Response: std::fmt::Debug,
    <<T as Queryable>::QueryMethod as RpcMethod>::Error: std::fmt::Debug,
    // Fut: TryFuture<Error = Error>,
    // F: FnOnce(QueryResponseKind) -> Fut,
    // Self: Sized,
{
    type Output = Result<R>;
    // type IntoFuture = Ready<Self::Output>;
    // type IntoFuture = AndThen<
    //     impl Future<
    //         Output = Result<
    //             <RpcQueryRequest as RpcMethod>::Response,
    //             JsonRpcError<<RpcQueryRequest as RpcMethod>::Error>,
    //         >,
    //     >,
    //     impl Future<Output = Result<R, JsonRpcError<RpcQueryError>>>,
    //     FnOnce(RpcQueryResponse) -> impl Future<Output = Result<R, JsonRpcError<RpcQueryError>>>,
    // >;
    // type IntoFuture = futures::future::IntoFuture;

    // TODO: boxed future required due to impl Trait as type alias being unstable. So once
    // https://github.com/rust-lang/rust/issues/63063 is resolved, we can move to that instead.
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let block_reference = self.block_ref.unwrap_or_else(BlockReference::latest);
        let fut = self
            .client
            .query(methods::query::RpcQueryRequest {
                block_reference,
                request: self.request,
            })
            // .query(self.method.into_query_request(block_ref))
            .map_ok(T::process_response)
            .map_err(|e| RpcErrorCode::QueryFailure.custom(e));
        // .and_then(|result| async move { Ok(T::into_value(result)) });

        Box::pin(fut)
        // Box::pin(TryFutureExt::into_future(fut))
    }
}

pub trait Queryable {
    // TODO: associated default type is unstable. So for now, will require writing
    // the manual impls for query_request
    type QueryMethod: RpcMethod;

    /// Expected output after performing a query. This is mainly to convert over
    /// the type from near-primitives to a workspace type.
    type Output;

    fn into_query_request(self, block_ref: BlockReference) -> Self::QueryMethod;
    // fn process_response(query: <Self::QueryMethod as RpcMethod>::Response) -> Self::Output;
    fn process_response(query: RpcQueryResponse) -> Self::Output;
}

struct View;
pub struct ViewCode {
    account_id: AccountId,
}
pub struct ViewAccount {
    account_id: AccountId,
}
struct ViewBlock;
struct ViewState;
struct ViewAccessKey;
struct ViewAccessKeyList;

impl Queryable for ViewCode {
    type Output = Result<Vec<u8>>;
    type QueryMethod = methods::query::RpcQueryRequest;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewCode {
                account_id: self.account_id,
            },
        }
    }

    fn process_response(query: RpcQueryResponse) -> Self::Output {
        match query.kind {
            QueryResponseKind::ViewCode(contract) => Ok(contract.code),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying account")),
        }
    }
}

impl Queryable for ViewAccount {
    type Output = Result<AccountDetails>;
    type QueryMethod = methods::query::RpcQueryRequest;

    fn into_query_request(self, block_reference: BlockReference) -> Self::QueryMethod {
        Self::QueryMethod {
            block_reference,
            request: QueryRequest::ViewAccount {
                account_id: self.account_id,
            },
        }
    }

    fn process_response(query: RpcQueryResponse) -> Self::Output {
        match query.kind {
            QueryResponseKind::ViewAccount(account) => Ok(account.into()),
            _ => Err(RpcErrorCode::QueryReturnedInvalidData.message("while querying account")),
        }
    }
}
