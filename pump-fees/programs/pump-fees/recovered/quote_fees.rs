//! Partial recovery of Pump Fees function 0x3f748..0x3f9d8.
//! Field order represents the observed SBF memory layout, not historical source.
#![no_std]
extern crate alloc;
use alloc::vec::Vec;
extern crate pump_fees_math as fee_selection;
use fee_selection::{Fees, FeeTier};

#[repr(C)]
pub struct FeeConfig {
    pub fee_tiers: Vec<FeeTier>,
    pub stable_fee_tiers: Vec<FeeTier>,
    pub admin: [u8; 32],
    pub flat_fees: Fees,
    pub exotic_flat_fees: Fees,
    pub bump: u8,
}

pub const WRAPPED_SOL: [u8; 32] = [6,155,136,87,254,171,129,132,251,104,127,99,70,24,192,53,218,196,57,220,26,235,59,85,152,160,240,0,0,0,0,1];
/// Third normal-tier quote constant, observed at 0xc35a0. Identity not inferred.
pub const OTHER_NORMAL_QUOTE: [u8; 32] = [131,13,252,159,222,95,230,184,170,124,4,164,118,233,30,138,198,187,38,74,173,144,250,25,201,223,73,216,92,62,91,94];
pub const USDC: [u8; 32] = [198,250,122,243,190,219,173,58,61,101,243,106,171,201,116,49,177,187,228,194,210,246,224,228,124,166,2,3,69,47,93,97];

// The original calls the program's C memcmp shim with a 32-byte length.
// Both inputs are fixed-size arrays, so the call always reads valid bytes.
extern "Rust" { fn recovered_memcmp(a: *const u8, b: *const u8, len: usize) -> i32; }
#[inline(always)]
fn quote_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    unsafe { recovered_memcmp(a.as_ptr(), b.as_ptr(), 32) == 0 }
}

#[no_mangle]
pub fn get_fees_with_quote(config: &FeeConfig, is_pump_pool: bool,
                           market_cap: u128, quote_mint: &[u8; 32]) -> Fees {
    if !is_pump_pool { return config.flat_fees; }
    let default_quote = [0u8; 32];
    if quote_eq(quote_mint, &default_quote) || quote_eq(quote_mint, &WRAPPED_SOL) || quote_eq(quote_mint, &OTHER_NORMAL_QUOTE) {
        fee_selection::select_fee_tier(&config.fee_tiers, market_cap)
    } else if quote_eq(&USDC, quote_mint) {
        fee_selection::select_fee_tier(&config.stable_fee_tiers, market_cap)
    } else if config.exotic_flat_fees.lp_fee_bps == 0
        && config.exotic_flat_fees.protocol_fee_bps == 0
        && config.exotic_flat_fees.creator_fee_bps == 0 {
        config.flat_fees
    } else {
        config.exotic_flat_fees
    }
}
