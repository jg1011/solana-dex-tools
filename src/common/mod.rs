//! # Common Abstractions 
//! 
//! This module contains the common abstractions for the `solana-dex-tools` crate 
//! that allow consumers to perform DEX-agnostic maintenance of on-chain account/liquidity-pool 
//! states for multi-threaded, read-only consumption of DEX data by external 
//! consumers. 
pub mod account;
pub mod deserialize;
pub mod pool;
pub mod rpc;
pub mod state;
pub mod types;