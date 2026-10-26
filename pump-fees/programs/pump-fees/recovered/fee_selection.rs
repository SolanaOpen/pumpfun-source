//! Partial implementation recovery of the fee-tier selector.
//! Original SBF range: 0xab158..0xab2d0 (Pump Fees capture).
//! This is not a complete program or a byte-identical reconstruction.
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Fees {
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub creator_fee_bps: u64,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FeeTier {
    pub market_cap_lamports_threshold: u128,
    pub fees: Fees,
}

/// The first-tier comparison happens before the reverse scan, even for an
/// unsorted list. An empty slice panics. No validation or sorting occurs here.
#[no_mangle]
pub fn select_fee_tier(fee_tiers: &[FeeTier], market_cap_lamports: u128) -> Fees {
    if market_cap_lamports < fee_tiers[0].market_cap_lamports_threshold {
        return fee_tiers[0].fees;
    }
    for tier in fee_tiers.iter().rev() {
        if market_cap_lamports >= tier.market_cap_lamports_threshold {
            return tier.fees;
        }
    }
    fee_tiers[0].fees
}
