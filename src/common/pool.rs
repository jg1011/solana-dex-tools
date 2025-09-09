use crate::common::state::AccountState;
use crate::common::types::AnyResult;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use std::any::Any;
use std::sync::Arc;

/// Defining behaviour of a liquidity pool.
///
/// This trait is designed to be object-safe, allowing for dynamic dispatch
/// over different pool types (e.g., Orca, Meteora). 
/// 
/// A `Pool` acts as a high-level container or a "view" over a set of individual `AccountState` objects, 
/// which represent the state of a liquidity pool.
#[async_trait]
pub trait Pool: Send + Sync {
    /// Returns a list of all the underlying `AccountState` objects managed by this pool.
    /// 
    /// The return type `Vec<Arc<dyn AccountState>>` is a vector of `Arc`s pointing to the
    /// abstract `AccountState` trait, allowing a caller to perform generic operations on 
    /// all accounts in a pool without needing to know their concrete types.
    fn accounts(&self) -> Vec<Arc<dyn AccountState>>;

    /// Downcasts the trait object to its concrete type (e.g., `OrcaWhirlpool`).
    fn as_any(&self) -> &dyn Any;

    /// Triggers a refresh of all accounts in the pool using the provided RPC client.
    ///
    /// Any implementation of this method should be highly optimized, fetching 
    /// all required account data in a minimum number of RPC calls and invoking the 
    /// update method on each respective `AccountState` object.
    /// 
    /// We recommend using the RpcClient::get_multiple_accounts method to fetch all required 
    /// account data, which works for up to 100 accounts at a time, on implementation.
    async fn refresh(&self, rpc_client: &RpcClient) -> AnyResult<()>;
}
