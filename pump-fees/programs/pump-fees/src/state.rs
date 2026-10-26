use anchor_lang::prelude::*;

#[account]
pub struct BuybackVault{pub authority:Pubkey,pub total_claimed:u64,pub total_claimed_token1:u64,pub total_claimed_token2:u64,pub last_claimed:i64,pub claim_rate_limit:i64,pub _reserved:[u8;128]}

#[account]
pub struct SocialFeePda{pub bump:u8,pub version:u8,pub user_id:String,pub platform:u8,pub total_claimed:u64,pub last_claimed:u64,pub total_stable_claimed:u64,pub _reserved:[u8;120]}

#[account]
pub struct DonationFeePda {pub bump:u8,pub version:u8,pub config_id:Pubkey,pub base_mint:Pubkey,pub quote_mint:Pubkey,pub creator:Pubkey,pub total_donated:u64,pub last_crank_ts:i64,pub reserved:[u8;64]}

#[account]
pub struct FeeProgramGlobal {
    pub bump: u8,
    pub authority: Pubkey,
    pub disable_flags: u8,
    pub social_claim_authority: Pubkey,
    pub claim_rate_limit: u64,
    pub _reserved: [u8; 256],
}
