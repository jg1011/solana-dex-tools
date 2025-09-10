//! Defines the `OrcaWhirlpool` struct and implements the `Pool` trait for it.

use crate::common::{
    account::AccountData,
    pool::Pool,
    rpc::RpcProvider,
    state::{AccountState, ManagedAccount},
};
use crate::orca::pda;
use anyhow::anyhow;
use crate::common::types::AnyResult;
use async_trait::async_trait;
use orca_whirlpools_client::{Oracle, TickArray, Whirlpool};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::account::Account;
use spl_token::state::Mint;
use std::any::Any;
use std::sync::Arc;

// --- Orca Whirlpool Struct Definition --- //

/// The logical collection of `ManagedAccount`s that define an Orca Whirlpool.
pub struct OrcaWhirlpool {
    pub whirlpool: Arc<ManagedAccount<Whirlpool>>,
    pub tick_arrays: Vec<Arc<ManagedAccount<TickArray>>>,
    // An `Option` is used because not all pools have an oracle account.
    pub oracle: Option<Arc<ManagedAccount<Oracle>>>,
    pub mint_a: Arc<ManagedAccount<Mint>>,
    pub mint_b: Arc<ManagedAccount<Mint>>,
}

/// Holds information about an account that failed to be fetched. 
/// 
/// This could be expanded to hold more information, and could be done 
/// better with an enum instead of a string. Was just a quick hack for testing.
#[derive(Debug)]
pub struct FailedAccount {
    pub pubkey: Pubkey,
    pub account_type: String,
}

/// Implements the `Pool` trait for the `OrcaWhirlpool` struct with the 
/// account type set to the standard `solana-sdk::account::Account` type. 
/// 
/// CRUCIAL NOTE: If you desire to use this logic for your own account type, it must be 
/// reimplemented. Thankfully, if your account has the `AccountData` trait, this implementation 
/// can be reused pretty much exactly!
#[async_trait]
impl Pool for OrcaWhirlpool {
    /// We implement for the standard `solana-sdk::account::Account` type.
    type AccountType = Account;

    /// Returns the pubkey of the pool, which is the pubkey of the whirlpool account.
    fn pubkey(&self) -> &Pubkey {
        self.whirlpool.pubkey()
    }

    /// Gathers `Arc` pointers to all accounts in the pool as `AccountState` objects for the pool.
    ///
    /// Allows for generic operations on all accounts in the pool without knowing their concrete types. 
    fn accounts(&self) -> Vec<Arc<dyn AccountState>> {
        let mut accounts: Vec<Arc<dyn AccountState>> = vec![
            self.whirlpool.clone(),
            self.mint_a.clone(),
            self.mint_b.clone(),
        ];
        accounts.extend(
            self.tick_arrays
                .iter()
                .map(|ta| ta.clone() as Arc<dyn AccountState>),
        );
        if let Some(oracle) = &self.oracle {
            accounts.push(oracle.clone());
        }
        accounts
    }

    /// Downcasts the `&dyn Pool` trait object back to a concrete `&OrcaWhirlpool`.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Triggers a refresh of the accounts that define the OrcaWhirlpool instance.
    /// 
    /// We invoke the `get_multiple_accounts` RPC method, which is confirmed by the RpcProvider trait here. 
    /// 
    /// NOTE: Your implementation of get_multiple_accounts must work for large numbers of accounts for this 
    /// to be safe. Our implementation for the Solana sdk's `RpcClient` type works for n accounts by batching 
    /// into chunks of size `max_accounts_per_rpc_call`, which in their case is 100. I recommend you do the same.
    /// 
    /// NOTE: Your get_multiple_accounts implementation must also be order preserving, otherwise the zip is nonsensical. 
    /// For non-order preserving RpcProviders, you will need a new implementation. But the Orphan rule will get you here. 
    /// If this niche case ever arrives, email me! I'll see what I can do. 
    async fn refresh(&self, rpc_client: &dyn RpcProvider<AccountType = Self::AccountType>) -> AnyResult<()> {
        let accounts_to_update: Vec<_> = self.accounts().iter().map(|a| *a.pubkey()).collect();

        let rpc_response = rpc_client.get_multiple_accounts(&accounts_to_update).await?;
        let accounts_data = rpc_response.result;
        let update_time = rpc_response.response_time;

        for (managed_account, account_data_option) in self.accounts().into_iter().zip(accounts_data.into_iter()) {
            if let Some(account_data) = account_data_option {
                let bytes = account_data.into_bytes();
                managed_account.update(bytes, update_time)?;
            }
        }

        Ok(())
    }
}

impl OrcaWhirlpool {
    /// Asynchronously fetches all the necessary on-chain data and constructs a new `OrcaWhirlpool` instance. 
    /// 
    /// Note: We don't actually require you to specify the account type associated with the RpcProvider for this implementation. But, for 
    /// the refresh implementation we still need to specify the account type, so this isn't much of a win outside, perhaps, for sniping new pools.
    /// 
    /// Note: We pay an additional rpc call here, along with a small clone cost. This is as we need the deserialized whirlpool data 
    /// to derive the addresses of all the associated accounts. Not a huge deal, we run this once in a pool's lifetime, but its worth 
    /// keeping in mind for snipers. There seems to be no real way to avoid this, I even asked the Orca devs!
    /// 
    /// Note: This time we don't require you to implement get_multiple_accounts for n accounts, we only require it work for the 
    /// maximal number in one call. We do the batching ourselves. This is just legacy code I don't fancy replacing, and may change 
    /// if I think of a reason its slower. Again, this runs once in a pool's lifetime, so not a huge deal.
    pub async fn new_initialized_from_rpc<C: RpcProvider + Send + Sync>(
        pubkey: &Pubkey,
        rpc_provider: &C,
    ) -> AnyResult<(Self, Vec<FailedAccount>)> {
        let whirlpool_response = rpc_provider
            .get_account(pubkey)
            .await
            .map_err(|e| anyhow!("Failed to fetch main whirlpool account {}: {}", pubkey, e))?;
        let whirlpool_time = whirlpool_response.response_time;
        let whirlpool_account = whirlpool_response.result;
        let whirlpool_data = Whirlpool::from_bytes(whirlpool_account.bytes())?;

        let mut pubkeys_to_fetch = vec![
            whirlpool_data.token_mint_a,
            whirlpool_data.token_mint_b,
        ];

        let oracle_pubkey = if let Ok((pubkey, _)) = pda::get_oracle_address(pubkey) {
            pubkeys_to_fetch.push(pubkey);
            Some(pubkey)
        } else {
            None
        };

        let tick_arrays_pubkeys =
            pda::get_tick_array_addresses(pubkey, &whirlpool_data.tick_spacing)?;
        pubkeys_to_fetch.extend_from_slice(&tick_arrays_pubkeys);

        let mut account_map = std::collections::HashMap::new();
        let mut failures = Vec::new();
        let limit = rpc_provider.max_accounts_per_rpc_call();
        // iterate over chunks of maximal size, minimising the number of RPC calls.
        for chunk in pubkeys_to_fetch.chunks(limit) {
            let rpc_response = rpc_provider.get_multiple_accounts(chunk).await?;
            let accounts_time = rpc_response.response_time;
            let accounts = rpc_response.result;
            for (i, account_option) in accounts.into_iter().enumerate() {
                if let Some(account) = account_option {
                    // Store the data along with the timestamp
                    account_map.insert(chunk[i], (account.bytes().to_vec(), accounts_time));
                }
            }
        }

        // quick closure to extract data from the account map
        // Use `remove` to transfer ownership of the data out of the map, avoiding a clone.
        let mut get_data = |pubkey: &Pubkey| account_map.remove(pubkey);

        // Create `ManagedAccount` instances for each piece of account data via the new_initialized_from_bytes method.

        let whirlpool = Arc::new(ManagedAccount::<Whirlpool>::new_initialized_from_bytes(
            *pubkey,
            whirlpool_account.bytes().to_vec(),
            whirlpool_time,
        )?);

        let (mint_a_data, mint_a_time) = get_data(&whirlpool_data.token_mint_a).ok_or_else(|| {
            anyhow!(
                "Required account Mint A {} could not be fetched",
                whirlpool_data.token_mint_a
            )
        })?;
        let mint_a = Arc::new(ManagedAccount::<Mint>::new_initialized_from_bytes(
            whirlpool_data.token_mint_a,
            mint_a_data,
            mint_a_time,
        )?);

        let (mint_b_data, mint_b_time) = get_data(&whirlpool_data.token_mint_b).ok_or_else(|| {
            anyhow!(
                "Required account Mint B {} could not be fetched",
                whirlpool_data.token_mint_b
            )
        })?;
        let mint_b = Arc::new(ManagedAccount::<Mint>::new_initialized_from_bytes(
            whirlpool_data.token_mint_b,
            mint_b_data,
            mint_b_time,
        )?);

        let oracle = if let Some(opk) = oracle_pubkey {
            if let Some((oracle_data, oracle_time)) = get_data(&opk) {
                Some(Arc::new(
                    ManagedAccount::<Oracle>::new_initialized_from_bytes(
                        opk,
                        oracle_data,
                        oracle_time,
                    )?,
                ))
            } else {
                failures.push(FailedAccount {
                    pubkey: opk,
                    account_type: "Oracle".to_string(),
                });
                None
            }
        } else {
            None
        };

        let mut tick_arrays = Vec::new();
        for ta_pubkey in &tick_arrays_pubkeys {
            if let Some((ta_data, ta_time)) = get_data(ta_pubkey) {
                tick_arrays.push(Arc::new(
                    ManagedAccount::<TickArray>::new_initialized_from_bytes(
                        *ta_pubkey, ta_data, ta_time,
                    )?,
                ));
            } else {
                // It's expected that not all tick arrays will exist on-chain.
                failures.push(FailedAccount {
                    pubkey: *ta_pubkey,
                    account_type: "TickArray".to_string(),
                });
            }
        }

        // 5. Assemble and return the `OrcaWhirlpool` struct with the `Arc`s.
        let pool = Self {
            whirlpool,
            tick_arrays,
            oracle,
            mint_a,
            mint_b,
        };

        Ok((pool, failures))
    }
}
