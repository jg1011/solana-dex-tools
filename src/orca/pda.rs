/// crates/dex_tools/src/orca/pda.rs ///

use solana_sdk::pubkey::Pubkey;
use crate::common::types::AnyResult;
use std::str::FromStr;
use anyhow::anyhow;
use orca_whirlpools_core::{
    TICK_ARRAY_SIZE, 
};
use num_integer::Integer;

/// Quick fn to get the whirlpool master pubkey
/// 
/// Returns:
///     - The pubkey that owns all whirlpools and associated accounts, e.g. tick arrays, oracles, etc.
pub fn parse_whirlpool_master_pubkey() -> Pubkey {
    Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap()
}

/// Given a whirlpool pubkey, returns all the corresponding tick array pubkeys.
/// 
/// The supported price range on whirlpools is [-2^64, 2^64], so noting p(i) = 1.0001^i for a tick idx 
/// i, we set p(i) = 2^64 to get i = 64 * ln(2) / ln(1.0001) = 443636.3759. Taking floors and repeating 
/// for p(i) = -2^64, we deduce the viable tick range is [-443636, 443636].
/// 
/// The start tick idx for a the tick array containing a given idx, i, is given by largest solution x
/// to x * TICK_ARRAY_SIZE * whirlpool.tick_spacing <= i, which is easily computed as 
///     - floor(i / (TICK_ARRAY_SIZE * whirlpool.tick_spacing)) * (TICK_ARRAY_SIZE * whirlpool.tick_spacing)
/// 
/// Given this context, the methodology becomes clear: 
///     - 1. Iterate over steps of size TICK_ARRAY_SIZE * whirlpool.tick_spacing started from 
///          - floor(-443636 / (TICK_ARRAY_SIZE * whirlpool.tick_spacing)) * (TICK_ARRAY_SIZE * whirlpool.tick_spacing)
///          to 
///          - floor(443636 / (TICK_ARRAY_SIZE * whirlpool.tick_spacing)) * (TICK_ARRAY_SIZE * whirlpool.tick_spacing)
///     - 2. Invoke get_tick_array_address fn to get the tick array pubkey for each start tick idx. 
/// 
/// Note: TICK_ARRAY_SIZE is 88, but we import it incase the adhd devs change it later. 
/// 
/// Note: There's a name collision with num-integer::Integer::div_floor, may be added to std lib later. 
///     - Warning resolved by keeping Integer prefix.
/// 
/// Note: Many of these tick arrays are uninitialized, but theres no way to check without 
/// an RPC call. We begin by trying all, and it's reasonable to assume any that failed 
/// are simply uninitialized.
/// 
/// Parameters: 
///     - pool_pubkey: Pointer to the pool's pubkey
///     - tick_spacing: A pointer to the space between ticks, pool dependent
/// 
/// Returns: 
///     - A vector of Pubkeys if successful, otherwise an (anyhow) error
pub fn get_tick_array_addresses(
    whirlpool_pubkey: &Pubkey,
    tick_spacing: &u16,
) -> AnyResult<Vec<Pubkey>> {
    let abs_max_tick_idx: i32 = 443636;
    let tick_array_width: i32 = TICK_ARRAY_SIZE as i32 * *tick_spacing as i32;
    let mut tick_array_pubkeys: Vec<Pubkey> = Vec::new();
    let mut curr_start_tick_idx: i32 = Integer::div_floor(&(-abs_max_tick_idx), &tick_array_width) * tick_array_width;
    let largest_start_tick_idx: i32 = Integer::div_floor(&abs_max_tick_idx, &tick_array_width) * tick_array_width;
    while curr_start_tick_idx <= largest_start_tick_idx {
        let tick_array_address = get_tick_array_address(
            whirlpool_pubkey,
            curr_start_tick_idx
        )?;
        tick_array_pubkeys.push(tick_array_address);
        curr_start_tick_idx += tick_array_width;
    }
    Ok(tick_array_pubkeys)
}

/// Given a whirlpool pubkey, returns the corresponding oracle pubkey 
/// 
/// Note oracle usually doesn't exist, only for new variable fee pools. We get the 
/// same error whether there was an issue or no oracle pubkey 
/// 
/// TODO:
///     - seperate errors for failing and for no pubkey
///     - remove irrelevant discriminant, applying necessary refactor to OrcaWhirlpool::new impl
/// 
/// Parameters: 
///     - pool_pubkey: Pointer to the pool's pubkey
/// 
/// Returns: 
///     - A tuple containing the oracle's pubkey and the discriminant or an (anyhow)error
pub fn get_oracle_address(pool_pubkey: &Pubkey) -> AnyResult<(Pubkey, u8)> {
    let seeds = &[b"oracle", pool_pubkey.as_ref()];
    let whirlpool_master_pubkey: Pubkey = parse_whirlpool_master_pubkey(); 
    let oracle_address_result:Option<(Pubkey, u8)> =  Pubkey::try_find_program_address(
        seeds, &whirlpool_master_pubkey);
    if oracle_address_result.is_none() {
        return Err(anyhow!("Failed to get oracle address"));
    }
    let oracle_address = oracle_address_result.unwrap();
    Ok(oracle_address)  
}

/// Given a whirlpool pubkey and a start tick index, derives the corresponding tick array pubkey.
/// 
/// Methodology given in orca rust sdk, we just minorly adapt their code here. 
///     - By minorly adapt, I mean we just swap program pubkeys for sdk pubkeys
/// 
/// NOTE: If start_tick_index is invalid, we still get a tick array pubkey
/// 
/// Parameters: 
///     - pool_pubkey: Pointer to the pool's pubkey
///     - start_tick_index: The first tick in the tick array
/// 
/// Returns: 
///     - The tick array's pubkey or an (anyhow) error
pub fn get_tick_array_address(
    pool_pubkey: &Pubkey,
    start_tick_index: i32,
) -> AnyResult<Pubkey> {
    let start_tick_index_str = start_tick_index.to_string();
    let seeds = &[
        b"tick_array",
        pool_pubkey.as_ref(),
        start_tick_index_str.as_bytes(),
    ];
    let whirlpool_master_pubkey: Pubkey = parse_whirlpool_master_pubkey(); 
    let tick_array_address_result:Option<(Pubkey, u8)> =  Pubkey::try_find_program_address(
        seeds, &whirlpool_master_pubkey);
    if tick_array_address_result.is_none() {
        return Err(anyhow!("Failed to get tick array address"));
    }
    // Unwrap the pubkey from the tuple w/ .0 as we don't need the discriminant
    Ok(tick_array_address_result.unwrap().0)
}