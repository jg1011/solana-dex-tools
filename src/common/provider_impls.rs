//! Provides a default implementation of the `RpcProvider` trait for the
//! standard non-blocking `solana-client`.
use crate::common::{
    rpc::{RpcProvider, RpcResponse},
    types::AnyResult,
};
use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::time::{SystemTime, UNIX_EPOCH};

#[async_trait]
impl RpcProvider for RpcClient {
    type AccountType = Account;

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
}
