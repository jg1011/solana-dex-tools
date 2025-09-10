# solana-dex-tools: A High-Performance Solana DEX Toolkit

## 1 - Mission Statement

`solana-dex-tools` is a high-performance, thread-safe, generic Rust library designed for developers building sophisticated high-frequency-trading (HFT) applications on the Solana blockchain. We provide utilities to use the `solana-sdk` and `tokio` ecosystems, but consumers using specialised toolkits are given the freedom to do so with our abstractions.

## 2 - Core Purpose & Use Case

In the world of high-frequency trading, every microsecond counts. Applications must manage real-time data from multiple sources concurrently without compromising state integrity. `solana_dex_tools` is engineered from the ground up to solve this problem.

This crate is ideal for:
-   **High Frequency Trading Firms** Our architecture is liberating enough to utilise internal sockets (e.g. a local validator), data structures and hardware optimisations all within our ecosystem. We also provide blanket implementations for the `solana-sdk` data structures and `solana-client` rpc client for the casual user. Further, our thread-safe architecture is well suited to running several algorithmic trading strategies from one central server, minimising data-execution latency. 
-   **Arbitrage Bots:** Easily monitor multiple hundreds of liquidity pools across several DEXs to identify arbitrage opportunities.
-   **Market Making:** Receive websocket streams with live order-book data across several DEXs, aiding in solving the optimal liquidity provisioning problem.
-   **Data Analytics Platforms:** Ingest and process all sorts of real-time on-chain DEX data with high throughput.


## 3 - Key Features

-   **Unified DEX Abstraction:** Generic `Pool` and `AccountState` traits create a standardized interface for different DEX implementations, with an easy pattern for adding new DEX implementations, allowing for a huge reduction in boilerplate code in multi-DEX applications.
-   **Flexible, Generic RPC Abstraction:** The library is generic over a new `RpcProvider` trait, decoupling it from any specific RPC client implementation or account data structure. For convenience, a default implementation for the standard nonblocking `solana-client` RPC client is provided out-of-the-box, with out of the box gRPC support coming in a future version.
-   **High-Performance & Thread-Safe State Management:** `ManagedAccount` instances use `ArcSwap` for lock-free, atomic updates via the pointer swap trick, perfect for a broadcast styled application with one provider and several consumers (e.g. a HFT firm, by consumers here we mean threads, though this would usually be abstracted with `tokio` tasks). Each account tracks its own `update_slot` counter and `last_update_time` timestamp (in unix nanoseconds) to help consumers track data freshness, along with swap-ready pointers to the raw byte data and the DEX-dependent deserialized data.

## 4 - Roadmap 

- Add more DEXs (immediate future, on the ol todo list. Raydium and Meteora have essentially been done, are just experimental)
- Pubkey level agnostics, allowing for Pool trait-objects to be initialized without prior knowledge of the associated DEX
    - I remark that this is logically intense. One approach is to attempt deserialization and if it matches a "pool account" (lead account for a pool) on a given DEX, then this is the DEX. Probably works....
- Out of the box gRPC functionality.
- Event streaming utilities for transaction level data.
- Shredstream deserialisation strategies and incorporation with the aforementioned event streaming utilities.

## 5 - Usage

Simply add `solana_dex_tools` to your `Cargo.toml`:
```toml
[dependencies]
solana_dex_tools = "1.0.3" # check crates.io page for latest version
```

## 6 - Example

Here is a comprehensive example demonstrating how to initialize a pool, store it as an object-safe `dyn Pool`, refresh its state abstractly, and downcast it to access implementation-specific data.

```rust,ignore
use solana_dex_tools::{
    common::pool::Pool,
    orca::pool::OrcaWhirlpool,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, account::Account};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

// This code must be run in an async context, e.g. #[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize a concrete, non-blocking RPC client.
    // The library provides an out-of-the-box RpcProvider implementation for this client.
    let rpc_client = Arc::new(RpcClient::new("YOUR_RPC_URL".to_string()));

    // A sample Orca Whirlpool public key (replace with a real one).
    let pool_pubkey = Pubkey::new_from_array([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
    ]);

    // 2. Initialize the concrete pool type.
    let (orca_pool, _failures) = OrcaWhirlpool::new_initialized_from_rpc(&pool_pubkey, &*rpc_client).await?;

    // 3. Store the concrete pool in a `Box<dyn Pool>`. This is where type erasure happens.
    // We must specify the `AccountType` that this pool requires from its provider.
    let pool: Box<dyn Pool<AccountType = Account>> = Box::new(orca_pool);

    println!("Successfully initialized pool {}", pool.pubkey());

    // To access Orca-specific data, we must downcast the trait object.
    if let Some(concrete_pool) = pool.as_any().downcast_ref::<OrcaWhirlpool>() {
        let whirlpool_data = concrete_pool.whirlpool.get();
        println!("Initial liquidity: {}", whirlpool_data.liquidity);
    }

    // 4. Wait for a bit...
    println!("\nWaiting 5 seconds before refreshing...");
    sleep(Duration::from_secs(5)).await;

    // 5. Refresh the pool's state abstractly using the trait method.
    // We don't need to know what kind of pool it is, only that it implements `Pool`.
    println!("Refreshing pool state...");
    pool.refresh(&*rpc_client).await?;
    println!("Refresh complete.");

    // 6. Downcast again to see the updated state.
    if let Some(concrete_pool) = pool.as_any().downcast_ref::<OrcaWhirlpool>() {
        let whirlpool_data = concrete_pool.whirlpool.get();
        println!("Updated liquidity: {}", whirlpool_data.liquidity);
    }

    Ok(())
}
```
## 7 - Advanced Usage: Custom RPC clients / Account types

For advanced use cases, such as using your own custom RPC-client/account-type pair or wrapping the blocking `solana-client`, you can: 

1 - implement the `AccountData` trait for your custom account-type (not necessary if this type is `solana_sdk::account::Account`)
2 - Implement the `RpcProvider` trait for your custom RPC-client type (necessary even if the associated account type is `solana_sdk::account::Account`). 
3 - Implement the `Pool` trait for your logical collection of accounts representing a liquidity pool (not necessary if you use the existing collections, which are collections of `solana_sdk::account::Account` types.)

Once steps 1 through 3 are complete, development using the abstracted `ManagedAccount<T>` (provided you're using a type `T` we've already implemented, if not you'll need to implement `AccountState` for `ManagedAccount<T>` too) struct and the object safe `Pool` trait (via the `dyn Pool` pattern). 

## 8 - Providing To `solana-dex-tools` 

#### 8.1 - Providing Utilities for Alternate RPC Clients and Account Types 

See section 7. 

#### 8.2 - Providing Functionality for Alternate DEXs 

If you want to add another DEX, at least one following the usual liquidity pool pattern, you just do the following: 

1. Identify the DEX by its name and give it its own folder (in `src/`). 
2. Identify the associated account types (post-deserialization) and implement the `Deserializable` trait for these. 
    - These are usually found in the DEX's program source code, though custom types can be used if these are well justified. 
3. Make sure the `Clone`, `Send` and `Sync` traits are implemented for `T` so that the blanket implementation of the  `AccountState` trait for `ManagedAccount<T>` can be used. Careful with the orphan rule here! 
4. Identify a liquidity pool by a logical collection of associated account types `T` (or `Vec<T>` for e.g. tick arrays, where there are multiple accounts of identical type) and define your concreate pool struct with `Arc<ManagedAccount<T>>` wrappers of these types `T`. 
5. Implement the `Pool` trait for your concrete pool struct with either the `solana_sdk::account::Account` account type chosen, or your own custom account type if this is a better fit (the latter being a rare case). 

In general, any DEX is just a logical collection of accounts on the blockchain, so this development pattern can be used for DEXs outside of the usual liquidity pool swap system, though this may be harder for e.g. markets utilitising a Serum orderbook. Feel free to reach out! Also, given they're just logical collections of on-chain accounts, we can almost certainly just use the `solana-sdk::account::Account` type, though a particularly thorough user may wish to use their own streamlined account type, hence we provide this flexibility at minimal cost to the average consumer. 

## License

This crate is licensed under the MIT License.
