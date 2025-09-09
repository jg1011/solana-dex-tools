use crate::common::traits::Deserializable;
use crate::common::types::AnyResult;
use arc_swap::{ArcSwap, Guard};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;
use std::sync::Arc;

// --- The Interface (Trait) ---

/// Defines the behaviour of a single on-chain account
///
/// This trait is "object-safe", meaning we can create pointers to this abstract type 
/// (like `Arc<dyn AccountState>`). Note size not known at compile time (vtable created) 
/// at compile time), so Arc wrapper necessary even in single-threaded applications.
pub trait AccountState: Send + Sync {
    /// Updates the account's state using a new set of raw bytes.
    ///
    /// The write operation for AccountState objects, where expensive deserialization occurs.
    fn update(&self, new_bytes: Vec<u8>) -> AnyResult<()>;

    /// Returns the account's unique identifier, its public key.
    fn pubkey(&self) -> &Pubkey;

    /// Provides read-only access to the raw byte data.
    ///
    /// The return type `Guard<Arc<Vec<u8>>>` is a "guard" from `arc-swap`. It acts as a
    /// temporary, atomic snapshot of the data. This is a fast (nanoseconds) operation, 
    /// and does not block concurrent `update` calls.
    fn bytes(&self) -> Guard<Arc<Vec<u8>>>;

    /// Allows for runtime downcasting.
    ///
    /// This is a mechanism to safely convert the abstract trait object (`&dyn AccountState`)
    /// back into its original, concrete struct type (e.g., `&ManagedAccount<Whirlpool>`).
    /// This is necessary to access the cached, deserialized data. 
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
    /// 
    /// ArcSwap is a thread-safe, atomic swap operation, allowing for near-instant 
    /// lock-free updates. Swapping data done by swapping ptrs. Note the full byte 
    /// array must be stored seperately for updates, we do NOT write to the bytes field.
    /// Under the hood, ArcSwap is a ptr to an Arc, which is a ptr to a Vec<u8>. We then
    /// wrap in an Arc to allow for multiple threads to have access to the ArcSwap ptr. 
    bytes: Arc<ArcSwap<Vec<u8>>>,
    /// The deserialized, data, wrapped in concurrency primitives.
    /// 
    /// ArcSwap is a thread-safe, atomic swap operation, allowing for near-instant 
    /// lock-free updates. Swapping data done by swapping ptrs. Note the full byte 
    /// array must be stored seperately for updates, we do NOT write to the bytes field.
    /// Under the hood, ArcSwap is a ptr to an Arc, which is a ptr to a Vec<u8>. We then
    /// wrap in an Arc to allow for multiple threads to have access to the ArcSwap ptr.
    /// 
    /// The type T is the deserialized on-chain account data, e.g. `Whirlpool` from the Orca SDK.
    deserialized: Arc<ArcSwap<T>>,
}

// --- ManagedAccount Struct Implementations --- //

/// We implement a `new` method, constructing a concrete `ManagedAccount` struct, and a 
/// `get` method, providing fast, read-only access to the deserialized data.
impl<T: Deserializable + Clone + Send + Sync + 'static> ManagedAccount<T> {
    /// Constructs a new `ManagedAccount` from a byte array containing the on-chain data.
    /// 
    /// Fails if initial_bytes cannot be deserialized into `T`.
    pub fn new(pubkey: Pubkey, initial_bytes: Vec<u8>) -> AnyResult<Self> {
        // Invoke the from_bytes method from the Deserializable trait.
        let initial_deserialized = T::from_bytes(&initial_bytes)?;
        Ok(Self {
            pubkey,
            // wrap the byte array and deserialized data in concurrency primitives.
            bytes: Arc::new(ArcSwap::new(Arc::new(initial_bytes))),
            deserialized: Arc::new(ArcSwap::new(Arc::new(initial_deserialized))),
        })
    }

    /// Provides fast, read-only access to the deserialized data.
    ///
    /// Guard is a RAII pattern. The destructor will be called when the guard goes out of scope.
    /// Guard also implements the Deref trait, as does Arc, so we can just reference a field 
    /// like, for example, `let liquidity = self.get().liquidity`.
    pub fn get(&self) -> Guard<Arc<T>> {
        // Load returns a guarded arc ptr to the deserialized data, essentially dereferencing 
        // the outer Arc.
        self.deserialized.load()
    }
}

// --- AccountState Trait Implementation --- //

impl<T: Deserializable + Clone + Send + Sync + 'static> AccountState for ManagedAccount<T> {
    fn update(&self, new_bytes: Vec<u8>) -> AnyResult<()> {
        // Attempt the expensive deserialization, aborting with ? if it fails. This 
        // uses anyhow under the hood. 
        let new_deserialized = T::from_bytes(&new_bytes)?;
        
        // If successful, atomically update raw bytes and deserialized data with 
        // ArcSwap's store operation. Simple ptr swap, no cloning or copying.
        self.bytes.store(Arc::new(new_bytes));
        self.deserialized.store(Arc::new(new_deserialized));
        Ok(())
    }

    fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }

    fn bytes(&self) -> Guard<Arc<Vec<u8>>> {
        self.bytes.load()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
