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
    type AccountType: AccountData + Send + Sync;

    fn pubkey(&self) -> &Pubkey;

    fn accounts(&self) -> Vec<Arc<dyn AccountState>>;

    async fn refresh(
        &self,
        rpc_client: &dyn RpcProvider<AccountType = Self::AccountType>,
    ) -> AnyResult<()>;

    /// Returns this pool as `&dyn Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
}
