//! # Dex Tools: High-Performance Solana DEX Toolkit
//!
//! `dex_tools` is a high-performance, thread-safe Rust library designed for developers
//! building sophisticated, high-frequency trading (HFT) applications on the Solana blockchain.
//! It provides a robust, generic, and highly optimized toolkit for interacting with various
//! decentralized exchanges (DEXs) with maximum efficiency and safety.
//!
//! ## Key Features
//!
//! -   **Unified DEX Abstraction:** Generic `Pool` and `AccountState` traits create a 
//!     standardized interface for different DEX implementations.
//! -   **High-Performance State Management:** ArcSwap allows ManagedAccount instances to 
//!     be updated atomically without cloning or copying (noting the initial clone is paid on 
//!     data receipt).
//! -   **Thread-Safe by Design:** State management is built on thread-safe primitives like `Arc`,
//!     allowing `Pool` and `ManagedAccount` data to be safely shared, read, and updated across 
//!     multiple threads.
pub mod common;
pub mod orca;
pub mod mint;