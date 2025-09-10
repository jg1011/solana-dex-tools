//! Defines the behaviour of a single on-chain account.

use crate::common::{
    account::AccountData,
    deserialize::Deserializable,
    rpc::RpcProvider,
    types::AnyResult,
};
use arc_swap::{ArcSwap, Guard};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// --- The Account Trait --- //

/// The behaviour of a single on-chain account, which is implemented for 
/// the blanket type `ManagedAccount<T>`. 
/// 
/// Note this trait is object-safe, so we can utilise dyn. 
pub trait AccountState: Send + Sync {
    /// Updates the account's state using a new set of raw bytes.
    ///
    /// This is expensive, a singular linear clone cost is incurred in the size of 
    /// the byte array. 
    fn update(&self, new_bytes: Vec<u8>, update_time: u64) -> AnyResult<()>;

    /// Returns the account's unique identifier, its public key.
    fn pubkey(&self) -> &Pubkey;

    /// Provides read-only access to the raw byte data.
    ///
    /// The return type `Guard<Arc<Vec<u8>>>` is a "guard" from `arc-swap`, 
    /// guarding the arc ptr to the byte data. 
    fn bytes(&self) -> Guard<Arc<Vec<u8>>>;

    /// Allows for runtime downcasting to the concrete type, e.g. `&ManagedAccount<Whirlpool>`.
    fn as_any(&self) -> &dyn Any;
}

/// Generic struct that manages the state for a specific type of on-chain account.
///
/// The generic type `T` represents a deserialized on-chain account, like `Whirlpool` or 
/// `TickArray` from the Orca SDK. If the on-chain deserialized struct is used, one could 
/// likely use anchor-lang tools for deserialization. Note anchor-lang causes dependency 
/// conflicts, haven't been able to get it to work. Worth a try though! 
///
/// The `where` clause specifies the constraints on `T`, similar to stating the domain
/// of a function. `T` must satisfy:
///   - `Deserializable`: Our local trait for types that can be created from bytes.
///   - `Clone`: It must be copyable.
///   - `Send + Sync`: It must be safe to share and access across multiple threads.
///   - `'static`: It must not contain any temporary references, for cross-thread safety.
pub struct ManagedAccount<T>
where
    T: Deserializable + Clone + Send + Sync + 'static,
{
    /// The public key of the account.
    pubkey: Pubkey,
    /// The raw byte data, wrapped in concurrency primitives.
    bytes: Arc<ArcSwap<Vec<u8>>>,
    /// The deserialized, data, wrapped in concurrency primitives.
    /// 
    /// The type T is the deserialized on-chain account data, e.g. `Whirlpool` from the Orca SDK.
    deserialized: Arc<ArcSwap<T>>,
    /// A simple counter that increments on each successful `update` call.
    update_slot: AtomicU64,
    /// The Unix nanoseconds timestamp of the last successful `update` call
    last_update_time: AtomicU64,
}

// --- ManagedAccount Struct Implementations --- //

/// We implement a `new` method, constructing a concrete `ManagedAccount` struct, and a 
/// `get` method, providing fast, read-only access to the deserialized data.
impl<T: Deserializable + Clone + Send + Sync + 'static> ManagedAccount<T> {
    /// Constructs a new `ManagedAccount` from a byte array containing the on-chain data.
    ///
    /// This is a low-level constructor. Prefer `new_initialized_from_rpc` where possible.
    /// Fails if initial_bytes cannot be deserialized into `T`.
    pub fn new_initialized_from_bytes(
        pubkey: Pubkey,
        initial_bytes: Vec<u8>,
        initial_time: u64,
    ) -> AnyResult<Self> {
        // Invoke the from_bytes method from the Deserializable trait.
        let initial_deserialized = T::from_bytes(&initial_bytes)?;
        Ok(Self {
            pubkey,
            // wrap the byte array and deserialized data in concurrency primitives.
            bytes: Arc::new(ArcSwap::new(Arc::new(initial_bytes))),
            deserialized: Arc::new(ArcSwap::new(Arc::new(initial_deserialized))),
            update_slot: AtomicU64::new(1), // Initialized state is the first version
            last_update_time: AtomicU64::new(initial_time),
        })
    }

    /// Asynchronously constructs a new, initialized `ManagedAccount` by fetching its data from an RPC provider.
    /// This implementation is generic and works with any provider that implements RpcProvider and 
    /// returns a type implementing `AccountData`.
    pub async fn new_initialized_from_rpc<C: RpcProvider + Send + Sync>(
        pubkey: Pubkey,
        rpc_provider: &C,
    ) -> AnyResult<Self> {
        let response = rpc_provider.get_account(&pubkey).await?;
        let time = response.response_time;
        let account_data = response.result;
        Self::new_initialized_from_bytes(pubkey, account_data.bytes().to_vec(), time)
    }

    /// Checks if the account has been populated with on-chain data.
    ///
    /// An account is considered initialized if its update slot is greater than 0.
    pub fn is_initialized(&self) -> bool {
        self.update_slot.load(Ordering::Relaxed) > 0
    }

    /// Provides fast, read-only access to the deserialized data.
    ///
    /// Guard is a RAII pattern. The destructor will be called when the guard goes out of scope.
    /// 
    /// Guard also implements the Deref trait, as does Arc, so we can just reference a field 
    /// directly, e.g. `let liquidity = self.get().liquidity`.
    pub fn get(&self) -> Guard<Arc<T>> {
        // Load returns a guarded arc ptr to the deserialized data
        self.deserialized.load()
    }
}

// --- AccountState Trait Implementation --- //

impl<T: Deserializable + Clone + Send + Sync + 'static> AccountState for ManagedAccount<T> {
    fn update(&self, new_bytes: Vec<u8>, update_time: u64) -> AnyResult<()> {
        // Attempt the expensive deserialization, aborting with ? if it fails. 
        let new_deserialized = T::from_bytes(&new_bytes)?;

        // If successful, atomically update raw bytes, deserialized data, and metadata.
        self.bytes.store(Arc::new(new_bytes));
        self.deserialized.store(Arc::new(new_deserialized));
        // We use the fetch_add and store methods for u64 to ensure atomicity is preserved across threads.
        self.update_slot.fetch_add(1, Ordering::Relaxed);
        self.last_update_time.store(update_time, Ordering::Relaxed);
        Ok(())
    }

    fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }

    fn bytes(&self) -> Guard<Arc<Vec<u8>>> {
         // Load returns a guarded arc ptr to the deserialized data.
        self.bytes.load()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
