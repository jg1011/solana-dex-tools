//! Generalises the solana-sdk::account::Account type to allow for any type that can 
//! provide the raw byte data of a Solana account. This is subject to change.

use solana_sdk::account::Account;

/// A trait for any type that can provide the raw byte data of a Solana account.
///
/// This trait allows the library to remain generic over the specific account data
/// structure returned by an `RpcProvider`, decoupling it from the `solana_sdk`.
pub trait AccountData {
    /// Provides read-only, slice access to the raw byte data of the account.
    fn bytes(&self) -> &[u8]; // points

    /// Consumes the account and returns the owned raw byte data.
    ///
    /// This is more efficient than `bytes()` when the caller needs ownership of the
    /// data, as it avoids a clone.
    fn into_bytes(self) -> Vec<u8> where Self: Sized; // consumes
}

/// Provide a default implementation for the most common account type.
///
/// This allows users of the standard `solana-client` to use the `RpcProvider`
/// trait without needing to write any boilerplate.
impl AccountData for Account {
    fn bytes(&self) -> &[u8] {
        &self.data // points
    }

    fn into_bytes(self) -> Vec<u8> {
        self.data // consumes
    }
}
