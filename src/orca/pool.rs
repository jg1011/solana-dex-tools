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
use spl_token::state::Mint;
use std::any::Any;
use std::sync::Arc;

// --- Orca Whirlpool Struct Definition --- //

/// A struct representing a complete Orca Whirlpool.
///
/// This struct is not the state itself. Instead, it is a lightweight, logical grouping
/// of `Arc` pointers. An `Arc` (Atomically Reference Counted) pointer is a "smart pointer"
/// that allows multiple parts of the program to share ownership of the same piece of data
/// on the heap without needing to copy the data.
///
/// Each field points to a `ManagedAccount<T>`, where `T` is the specific deserialized
/// type for that part of the pool (e.g., `Whirlpool`, `TickArray`). This `OrcaWhirlpool`
/// acts as a convenient, type-safe "view" into the global state map.
pub struct OrcaWhirlpool {
    // Each field is a thread-safe, shared pointer to a managed account state.
    pub whirlpool: Arc<ManagedAccount<Whirlpool>>,
    pub tick_arrays: Vec<Arc<ManagedAccount<TickArray>>>,
    // An `Option` is used because not all pools have an oracle account.
    pub oracle: Option<Arc<ManagedAccount<Oracle>>>,
    pub mint_a: Arc<ManagedAccount<Mint>>,
    pub mint_b: Arc<ManagedAccount<Mint>>,
}

/// A struct to hold information about an account that failed to be fetched.
#[derive(Debug)]
pub struct FailedAccount {
    pub pubkey: Pubkey,
    pub account_type: String,
}

#[async_trait]
impl Pool for OrcaWhirlpool {
    /// Gathers and returns `Arc` pointers to all the underlying `AccountState` objects for the pool.
    ///
    /// This method is crucial for the trait object system. It "erases" the concrete types
    /// (e.g., `ManagedAccount<Whirlpool>`) and returns a list of the abstract `dyn AccountState`
    /// type. This allows generic code to operate on all accounts in the pool without knowing
    /// what kind of accounts they are.
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

    /// Provides a way to downcast the `&dyn Pool` trait object back to a concrete `&OrcaWhirlpool`.
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Triggers a refresh of all accounts in the pool.
    ///
    /// The implementation will:
    /// 1. Collect all pubkeys from the `self.accounts()` vector.
    /// 2. Make a single, parallel `get_multiple_accounts` RPC call.
    /// 3. For each returned account data, find the corresponding `AccountState`
    ///    object in the `self.accounts()` vector.
    /// 4. Call the `update` method on that `AccountState` object with the new bytes.
    /// This ensures the expensive deserialization happens on the "cold path"
    /// and the cache is updated atomically.
    async fn refresh<C: RpcProvider + Send + Sync>(&self, rpc_client: &C) -> AnyResult<()> {
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
    /// Asynchronously fetches all necessary on-chain data and constructs a new `OrcaWhirlpool`.
    ///
    /// This constructor is a complex operation responsible for the initial creation of all
    /// the `ManagedAccount` states that compose the pool. It is generic over any `RpcProvider`.
    pub async fn new_initialized_from_rpc<C: RpcProvider + Send + Sync>(
        pubkey: &Pubkey,
        rpc_provider: &C,
    ) -> AnyResult<(Self, Vec<FailedAccount>)> {
        // 1. Fetch and deserialize the main whirlpool account to get necessary details
        let whirlpool_response = rpc_provider
            .get_account(pubkey)
            .await
            .map_err(|e| anyhow!("Failed to fetch main whirlpool account {}: {}", pubkey, e))?;
        let whirlpool_time = whirlpool_response.response_time;
        let whirlpool_account = whirlpool_response.result;
        let whirlpool_data = Whirlpool::from_bytes(whirlpool_account.bytes())?;

        // 2. Derive the addresses of all other required accounts using the custom PDA logic.
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

        // 3. Perform a single, parallel `get_multiple_accounts` RPC call.
            // The RPC call has a limit of 100 accounts, so we chunk into 100 accounts.
        let mut account_map = std::collections::HashMap::new();
        let mut failures = Vec::new();
        for chunk in pubkeys_to_fetch.chunks(100) {
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

        // 4. Create `ManagedAccount` instances for each piece of account data.
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
