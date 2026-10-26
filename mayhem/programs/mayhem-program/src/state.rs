use anchor_lang::prelude::*;
pub const PUMP_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

/// Serialized offsets are recovered from 0x685f0 and 0x68ce0.
/// Neutral field names avoid claiming unavailable historical semantics.
#[account]
pub struct MayhemState {
    pub field_8: u64,             // 8
    pub end_timestamp: i64,        // 16: signed Clock comparison in burn
    pub mint: Pubkey,              // 24
    pub field_56: i128,            // 56: exact width; signedness not proven here
    pub field_72: i128,            // 72: exact width; signedness not proven here
    pub flag_88: bool,             // 88: decoder accepts only 0 or 1
    pub field_89: u64,             // 89
    pub status: MayhemStatus,      // 97: decoder accepts only 0, 1 or 2
    pub burned_amount: u64,        // 98
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MayhemStatus { Variant0, Variant1, Ended }

/// A separate namespace suppresses the automatic Mayhem Owner implementation.
/// The codec at 0x6dc00 writes 82 bytes including this account discriminator.
#[account("account")]
pub struct BondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
    pub is_mayhem_mode: bool,
}
impl Owner for BondingCurve { fn owner() -> Pubkey { PUMP_ID } }

/// Exact observed Borsh positions; opaque bytes remain named by behavior.
/// This is not an assertion of historical field names or Rust memory layout.
#[account]
pub struct GlobalParams {
    pub reset_prefix: [u8; 40], // serialized offset 8
    pub paused: bool, // 48
    pub admin: Pubkey, // 49
    pub trader: Pubkey, // 81
    pub sol_deployed: i128, // 113
    pub preserved_middle_a: [u8; 8], // 129
    pub reset_middle: [u8; 25], // 137
    pub version: u32, // 162, incremented by update_global_params
    pub field_10: u64, // 166
    pub max_sol_per_sweep: u64, // 174
    pub reset_stable_prefix: [u8; 24], // 182
    pub stable_deployed: i128, // 206
    pub reset_stable_suffix: [u8; 24], // 222
    pub max_stable_per_sweep: u64, // 246
}
