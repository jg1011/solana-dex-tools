use crate::common::traits::Deserializable;
use anyhow::{anyhow, Result};
use spl_token::{state::Mint, solana_program::program_pack::Pack};

// --- Deserialization Trait Implementations --- //

impl Deserializable for Mint {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Mint is not a POD struct, so we must use the `spl_token` library's
        // `Pack` trait, which performs a safe, manual deserialization.
        Mint::unpack(bytes).map_err(|e| anyhow!("Failed to unpack Mint: {}", e))
    }
}