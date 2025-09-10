//! Defines the behaviour of an RPC client.

use crate::common::{
    account::AccountData,
    types::AnyResult,
};
use async_trait::async_trait;
use solana_sdk::{
    account::Account, 
    pubkey::Pubkey
};
use solana_client::nonblocking::rpc_client::RpcClient;
use std::time::{SystemTime, UNIX_EPOCH};


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
pub trait RpcProvider: Send + Sync {
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

    fn max_accounts_per_rpc_call(&self) -> usize;
}

#[async_trait]
impl RpcProvider for RpcClient {
    type AccountType = Account;

    /// Just invokes the underlying `RpcClient::get_account` method, but 
    /// also handles the response time tracking.
    async fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> AnyResult<RpcResponse<Self::AccountType>> {
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let result = self.get_account(pubkey).await?;
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        // The number of nanoseconds in a u64 is safe for the next ~500 years.
        // We take the average of the start and end times to get the response time.
        // Idea is pings each direction roughly equal, and server time negligible, 
        // so this is a good approximation of the actual response time.
        let response_time = (start_time + (end_time - start_time) / 2) as u64;

        Ok(RpcResponse {
            result,
            response_time,
        })
    }

    /// Just invokes the underlying `RpcClient::get_multiple_accounts` method, but 
    /// also handles the response time tracking.
    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> AnyResult<RpcResponse<Vec<Option<Self::AccountType>>>> {
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let result = self.get_multiple_accounts(pubkeys).await?;
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        // The number of nanoseconds in a u64 is safe for the next ~500 years.
        // We take the average of the start and end times to get the response time.
        // Idea is pings each direction roughly equal, and server time negligible, 
        // so this is a good approximation of the actual response time.
        let response_time = (start_time + (end_time - start_time) / 2) as u64;

        Ok(RpcResponse {
            result,
            response_time,
        })
    }

    /// Returns the maximum number of accounts that can be fetched in a single RPC call.
    /// 
    /// See solana-sdk::nonblocking::rpc_client::RpcClient::get_multiple_accounts for more details.
    fn max_accounts_per_rpc_call(&self) -> usize {
        100
    }
}

