use crate::common::types::AnyResult;

/// A local trait to provide a generic interface for deserialization.
///
/// This "adapter" trait allows us to create a bridge between our generic code
/// (like `ManagedAccount<T>`) and external types that don't share a common
/// deserialization interface (e.g. some use local sdk methods, some use SPL's `Pack`).
/// 
/// For any type with the deserialization method implemented, we can use the from_bytes method
/// to deserialize a byte array into an instance of the type. 
/// 
/// Note: Currently, all deserialization methods use a clone cost. By invoking ArcSwap's 
/// swap operation, we can avoid paying a second clone cost. Note clone-free deserialization 
/// would require a write-lock on the bytes, meaning ArcSwap would then pay a clone cost. 
/// This is unavoidable, so it's not worth the effort implementing clone-free deserialization 
/// via e.g. bytemuck. 
pub trait Deserializable: Sized {
    /// Constructs an instance of `Self` from a byte slice.
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self>;
}
