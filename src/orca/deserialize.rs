//! Defines the deserialization of the Orca Whirlpool, TickArray and Oracle accounts.
//! 
//! All are done by invoking the from_bytes method from the orca-sdk, not to be confused with our method!

use crate::common::{deserialize::Deserializable, types::AnyResult};
use anyhow::anyhow;
use orca_whirlpools_client::{Oracle, TickArray, Whirlpool};

impl Deserializable for Whirlpool {
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self> {
        Whirlpool::from_bytes(bytes).map_err(|e| anyhow!("Failed to deserialize Whirlpool: {}", e))
    }
}

impl Deserializable for TickArray {
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self> {
        TickArray::from_bytes(bytes).map_err(|e| anyhow!("Failed to deserialize TickArray: {}", e))
    }
}

impl Deserializable for Oracle {
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self> {
        Oracle::from_bytes(bytes).map_err(|e| anyhow!("Failed to deserialize Oracle: {}", e))
    }
}
