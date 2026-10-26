//! Native differential harness for the recovered selector, not an SBF program.
#[path = "fee_selection.rs"]
mod recovered;
use recovered::{FeeTier, Fees, select_fee_tier};
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut fields = line.split(';');
        let cap: u128 = fields.next().unwrap().parse().unwrap();
        let tiers: Vec<FeeTier> = fields.map(|f| {
            let v: Vec<&str> = f.split(',').collect();
            FeeTier { market_cap_lamports_threshold: v[0].parse().unwrap(), fees: Fees {
                lp_fee_bps: v[1].parse().unwrap(), protocol_fee_bps: v[2].parse().unwrap(),
                creator_fee_bps: v[3].parse().unwrap() } }
        }).collect();
        let out = select_fee_tier(&tiers, cap);
        println!("{},{},{}", out.lp_fee_bps, out.protocol_fee_bps, out.creator_fee_bps);
    }
}
