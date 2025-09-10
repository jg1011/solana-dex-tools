//! Defines a trait for types that can be deserialized from a raw byte slice.
use crate::common::types::AnyResult;

/// Our own trait for types that can be deserialized from a raw byte slice.
///
/// Note: This trait is not the same as the `Deserialize` trait from `serde`, 
/// we have our own for convenience on implementation. Different DEXs take 
/// different approaches to deserialization, so we aggregate into our own 
/// trait for convenience in DEX-agnostic development (orphan rule strikes again!). 
pub trait Deserializable {
    /// Attempts to deserialize the type from a byte slice.
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self>
    where
        Self: Sized;
}
