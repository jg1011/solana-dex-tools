# solana-dex-tools : A High-Performance Solana DEX Toolkit

## Mission Statement

`solana-dex-tools` is a high-performance, thread-safe Rust library designed for developers building sophisticated, high-frequency trading (HFT) applications on the Solana blockchain. Our mission is to provide a robust, generic, and highly optimized toolkit for interacting with various decentralized exchanges (DEXs) with maximum efficiency and safety. We are, essentially, a wrapper for the `solana-sdk` and `solana-client` crates, providing thread-safe, high-performance and easy to use tools tailored to DEXs.

## Core Purpose & Use Case

In the world of high-frequency trading, every microsecond counts. Applications must manage real-time data from multiple sources concurrently without compromising state integrity. `solana_dex_tools` is engineered from the ground up to solve this problem.

This crate is ideal for:
-   **Arbitrage Bots:** Concurrently monitor multiple liquidity pools across different DEXs to identify and execute profitable arbitrage opportunities.
-   **Market Making:** Manage and update quotes on multiple markets simultaneously in a thread-safe manner.
-   **Data Analytics Platforms:** Ingest and process real-time on-chain DEX data with high throughput.
-   **High Frequency Trading Firms** With some work (our team have done this, the code is proprietary) this toolkit can be integrated with a database (e.g. SQL, kdb+) for more advanced applications, e.g. stochastic-filtering. 

## Key Features

-   **Unified DEX Abstraction:** Generic `Pool` and `AccountState` traits create a standardized interface for different DEX implementations.
-   **High-Performance State Management:** `ArcSwap` allows `ManagedAccount` instances to be updated atomically without cloning or copying.
-   **Thread-Safe by Design:** State management is built on thread-safe primitives like `Arc` and `ArcSwap`, allowing `Pool` and `ManagedAccount` data to be safely shared, read, and updated across multiple threads.

## Future Aspirations

- Add more DEXs (immediate future, on the ol todo list. Raydium and Meteora have essentially been done, are just experimental)
- gRPC support
- Event streaming utilities for transaction level data.
- Shredstream deserialisation strategies and incorporation with the aforementioned event streaming utilities.

## Usage

Simply add `solana_dex_tools` to your `Cargo.toml`:
```toml
[dependencies]
solana_dex_tools = "1.0.0" # check crates.io apage for latest version
```

## Example

NOTE: More in-depth examples, for example websocket integration & multi-dex use-cases, will be added to /examples. I just need to write these up...

Here is a conceptual overview of how to use `dex_tools` to interact with a DEX pool:

1.  **Initialize an RPC Client:** Create an instance of `solana_client::rpc_client::RpcClient` to connect to a Solana RPC endpoint.

2.  **Define the Pool Public Key:** Identify the `Pubkey` of the on-chain liquidity pool you want to interact with.

3.  **Construct the Pool Object:** Use the implementation-specific constructor (e.g., `OrcaWhirlpool::new()`) and pass the pool's public key and the RPC client. This will perform the initial fetch of all necessary on-chain accounts.

```rust,ignore
use dex_tools::orca::pool::OrcaWhirlpool;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

let rpc_client = RpcClient::new("YOUR_RPC_URL".to_string());
let pool_pubkey = Pubkey::new_from_array([...]); // The pool's on-chain address

// The `new` function is async and returns the pool object and a list of any
// accounts that failed to fetch (which may be expected for some tick arrays).
let (pool, _failures) = OrcaWhirlpool::new(&pool_pubkey, &rpc_client).await?;
```

4.  **Access Pool Data:** The constructed `pool` object contains several `ManagedAccount` fields (e.g., `pool.whirlpool`, `pool.mint_a`). Call the `.get()` method on these fields to get a fast, lock-free, read-only guard to the deserialized on-chain state.

```rust,ignore
// Get a thread-safe guard to the deserialized Whirlpool state.
let whirlpool_data = pool.whirlpool.get();
println!("Current liquidity: {}", whirlpool_data.liquidity);
```

5.  **Refresh State:** To update the local state with the latest on-chain data, use the `.refresh()` method. This will efficiently re-fetch all accounts in a single RPC call.

```rust,ignore
use dex_tools::common::pool::Pool; // Bring the trait into scope

pool.refresh(&rpc_client).await?;
let latest_liquidity = pool.whirlpool.get().liquidity;
println!("Updated liquidity: {}", latest_liquidity);
```

## License

This crate is licensed under the MIT License.
