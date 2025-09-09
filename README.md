# solana-dex-tools: A High-Performance Solana DEX Toolkit

## Mission Statement

`solana-dex-tools` is a high-performance, thread-safe Rust library designed for developers building sophisticated, high-frequency trading (HFT) applications on the Solana blockchain. Our mission is to provide a robust, generic, and highly optimized toolkit for interacting with various decentralized exchanges (DEXs) with maximum efficiency and safety. We are, essentially, a wrapper for the `solana-sdk` and `solana-program` crates, providing thread-safe, high-performance and easy to use tools tailored to DEXs.

## Core Purpose & Use Case

In the world of high-frequency trading, every microsecond counts. Applications must manage real-time data from multiple sources concurrently without compromising state integrity. `solana_dex_tools` is engineered from the ground up to solve this problem.

This crate is ideal for:
-   **Arbitrage Bots:** Concurrently monitor multiple liquidity pools across different DEXs to identify and execute profitable arbitrage opportunities.
-   **Market Making:** Manage and update quotes on multiple markets simultaneously in a thread-safe manner.
-   **Data Analytics Platforms:** Ingest and process real-time on-chain DEX data with high throughput.
-   **High Frequency Trading Firms** With some work (our team have done this, the code is proprietary) this toolkit can be integrated with a database (e.g. SQL, kdb+) for more advanced applications, e.g. stochastic-filtering.

## Key Features

-   **Unified DEX Abstraction:** Generic `Pool` and `AccountState` traits create a standardized interface for different DEX implementations.
-   **Flexible, Generic RPC Abstraction:** The library is generic over a new `RpcProvider` trait, decoupling it from any specific RPC client implementation or account data structure. For convenience, a default implementation for the standard nonblocking `solana-client` RPC client is provided out-of-the-box. 
-   **High-Performance State Management:** `ManagedAccount` instances use `ArcSwap` for lock-free, atomic updates via the pointer swap trick. Each account tracks its own `update_slot` counter and `last_update_time` timestamp (in nanoseconds) to help consumers track data freshness.
-   **Thread-Safe by Design:** State management is built on thread-safe primitives like `Arc` and `ArcSwap`, allowing `Pool` and `ManagedAccount` data to be safely shared, read, and updated across multiple threads.

## Future Aspirations

- Add more DEXs (immediate future, on the ol todo list. Raydium and Meteora have essentially been done, are just experimental)
- Out of the box gRPC functionality.
- Event streaming utilities for transaction level data.
- Shredstream deserialisation strategies and incorporation with the aforementioned event streaming utilities.

## Usage

Simply add `solana_dex_tools` to your `Cargo.toml`:
```toml
[dependencies]
solana_dex_tools = "1.0.3" # check crates.io page for latest version
```

## Example

NOTE: More in-depth examples, for example websocket integration & multi-dex use-cases, will be added to /examples. I just need to write these up...

Here is a conceptual overview of how to use `solana_dex_tools` to interact with a DEX pool:

1.  **Choose an RPC Client:** Select an asynchronous RPC client that can connect to a Solana endpoint. For this example, we will use `solana_client::nonblocking::rpc_client::RpcClient`, with the `solana_sdk::account::Account` account type, noting we provide out of the box functionality for this client/account pair.

2.  **Define the Pool Public Key:** Identify the `Pubkey` of the on-chain liquidity pool you want to interact with.

3.  **Construct the Pool Object:** Use the implementation-specific constructor (e.g., `OrcaWhirlpool::new()`) and pass the pool's public key and an `Arc` of your chosen RPC client. The library is generic over the `solana_dex_tools::common::rpc::RpcClient` trait.

```rust,ignore
use solana_dex_tools::{
    common::{pool::Pool, rpc::RpcClient as DexRpcClient},
    orca::pool::OrcaWhirlpool,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

// This code must be run in an async context, like a tokio::main function.
async fn main() -> anyhow::Result<()> {
    // 1. Initialize a concrete, non-blocking RPC client.
    let rpc_client = Arc::new(RpcClient::new("YOUR_RPC_URL".to_string()));
    let pool_pubkey = Pubkey::new_from_array([...]); // The pool's on-chain address

    // 2. The `new_initialized_from_rpc` function is generic and takes any client that implements the trait.
    // For users of the standard solana-client, the trait is implemented automatically.
    let (pool, _failures) = OrcaWhirlpool::new_initialized_from_rpc(&pool_pubkey, &rpc_client).await?;

    // 3. Get a thread-safe guard to the deserialized Whirlpool state.
    let whirlpool_data = pool.whirlpool.get();
    println!("Current liquidity: {}", whirlpool_data.liquidity);
    println!("Update slot: {}", pool.whirlpool.update_slot.load(std::sync::atomic::Ordering::Relaxed));

    // 4. Refresh the state by passing the client again.
    pool.refresh(&rpc_client).await?;
    let latest_liquidity = pool.whirlpool.get().liquidity;
    println!("Updated liquidity: {}", latest_liquidity);
    println!("Update slot: {}", pool.whirlpool.update_slot.load(std::sync::atomic::Ordering::Relaxed));

    Ok(())
}
```
## Advanced Usage: Custom `RpcProvider`

For advanced use cases, such as using a custom RPC client, a different data source, or wrapping the blocking `solana-client`, you can implement the `RpcProvider` trait for your own type.

The trait is generic over the `AccountType` it returns, requiring only that your type implements the `AccountData` trait, which has a single method: `bytes(&self) -> &[u8]`. This provides maximum flexibility while ensuring the core library can always access the raw account data it needs for deserialization. 

## Providing To `solana-dex-tools` 

If you want to add another DEX, you must do the following: 

- identify the DEX by its name and give it its own folder (in `src/`). 
- Identify the associated account types (post-deserialization) and implement the `Deserializable` trait for these. 
- Define the liquidity pool's struct as a logical collection of `Arc` pointers to `ManagedAccount<T>` structs, where `T` here is the associated account type. 
- Implement the `Pool` trait for this. 

## License

This crate is licensed under the MIT License.
