//! Defines the deserialization of the Mint account into the spl_token::state::Mint type.

use crate::common::{deserialize::Deserializable, types::AnyResult};
use anyhow::anyhow;
use spl_token::{
    state::Mint, 
    solana_program::program_pack::Pack, // needed for deserialization
};

impl Deserializable for Mint {
    fn from_bytes(bytes: &[u8]) -> AnyResult<Self> {
        Mint::unpack(bytes).map_err(|e| anyhow!("Failed to deserialize Mint: {}", e))
    }
}
