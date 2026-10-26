#[path = "memory_compare.rs"]
mod memory_compare;
extern crate pump_quote_fees;
extern crate pump_fees_math;
use pump_quote_fees::{FeeConfig, get_fees_with_quote};
use pump_fees_math::{FeeTier, Fees};
use std::io::{self, BufRead};
fn fees(s: &str) -> Fees {
    let v: Vec<u64> = s.split(',').map(|x| x.parse().unwrap()).collect();
    Fees { lp_fee_bps: v[0], protocol_fee_bps: v[1], creator_fee_bps: v[2] }
}
fn tiers(s: &str) -> Vec<FeeTier> {
    if s.is_empty() { return vec![]; }
    s.split('|').map(|row| {
        let (threshold, rest) = row.split_once(',').unwrap();
        FeeTier { market_cap_lamports_threshold: threshold.parse().unwrap(), fees: fees(rest) }
    }).collect()
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for line in io::stdin().lock().lines() {
        let line = line.unwrap(); let f: Vec<&str> = line.split(';').collect();
        let mut quote = [0u8; 32];
        for (i, b) in quote.iter_mut().enumerate() { *b = u8::from_str_radix(&f[2][2*i..2*i+2],16).unwrap(); }
        let config = FeeConfig { fee_tiers: tiers(f[5]), stable_fee_tiers: tiers(f[6]),
            admin: [0; 32], flat_fees: fees(f[3]), exotic_flat_fees: fees(f[4]), bump: 0 };
        let result = std::panic::catch_unwind(|| get_fees_with_quote(&config, f[1]=="1", f[0].parse().unwrap(), &quote));
        match result {
            Ok(out) => println!("{},{},{}",out.lp_fee_bps,out.protocol_fee_bps,out.creator_fee_bps),
            Err(_) => println!("PANIC"),
        }
    }
}
