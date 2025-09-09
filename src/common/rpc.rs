use crate::common::{
    account::AccountData,
    types::AnyResult,
};
use async_trait::async_trait;
use solana_sdk::pubkey::Pubkey;

/// A generic wrapper for RPC responses that includes the time the response was received.
///
/// This allows the `RpcProvider` trait to remain generic while providing essential metadata
/// to the core library for tracking data freshness.
pub struct RpcResponse<T> {
    pub result: T,
    pub response_time: u64, // Unix timestamp in nanoseconds
}

/// An abstract interface for a client that can provide Solana account data.
///
/// This trait is generic over the specific account type, allowing consumers of
/// this library to use alternative account data structures if needed.
#[async_trait]
pub trait RpcProvider {
    /// The concrete type this RPC client returns for an account.
    /// Must implement the `AccountData` trait.
    type AccountType: AccountData + Send + Sync;

    /// Fetches a single account.
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> AnyResult<RpcResponse<Self::AccountType>>;

    /// Fetches multiple accounts.
    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> AnyResult<RpcResponse<Vec<Option<Self::AccountType>>>>;
}
