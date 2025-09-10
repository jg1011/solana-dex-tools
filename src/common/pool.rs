use crate::common::{
    account::AccountData,
    rpc::RpcProvider,
    state::AccountState,
    types::AnyResult,
};
use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;
use std::{any::Any, sync::Arc};

#[async_trait]
pub trait Pool: Send + Sync {
    /// The type of accounts that are given by fetching from the RPC client during refresh. 
    /// 
    /// It is necessary to specify this type to maintain object safety, otherwise we can't build our vtable!
    type AccountType: AccountData + Send + Sync;

    /// Returns the pubkey of the pool's `owner`
    fn pubkey(&self) -> &Pubkey; 

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
    /// The RPC client is generic over the AccountType, allowing for this method to 
    /// be used with any RPC client that implements the RpcProvider trait over the 
    /// specified AccountType.
    async fn refresh(&self, rpc_client: &dyn RpcProvider<AccountType = Self::AccountType>) -> AnyResult<()>;
}
