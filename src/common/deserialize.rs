use crate::common::types::AnyResult;

/// A trait for types that can be deserialized from a raw byte slice.
///
/// This is a local, simplified version of what might be provided by a larger
/// framework like `borsh` or `serde`. It serves to decouple the core library
/// logic from any specific deserialization framework.
pub trait Deserializable {
    /// Attempts to deserialize the type from a byte slice.
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self>
    where
        Self: Sized;
}
