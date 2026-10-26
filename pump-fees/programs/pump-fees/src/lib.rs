//! Partial reconstruction: all thirty-one instruction bodies implemented with bounded validation.
//! It is not the deployed program source.
use anchor_lang::prelude::*;

mod contexts;
mod donation;
mod social;
mod withdrawal;
mod buyback;
mod sharing;
mod sharing_update;
mod sharing_admin;
use sharing_update::*;
use sharing_admin::*;
use buyback::*;
use sharing::*;
use withdrawal::*;
use social::*;
use donation::*;
mod pump_global;
mod types;
mod state;
mod instructions;
use contexts::*;
use types::*;

declare_id!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");

declare_program!(pump);
declare_program!(pump_amm);

#[program]
pub mod pump_fees_candidate {
    use super::*;

    #[allow(unused_variables)]
    pub fn admin_cto_sharing_config(ctx: Context<AdminCtoSharingConfig>, new_admin: Option<Pubkey>) -> Result<()> {
        sharing_admin::admin_cto_sharing_config(ctx, new_admin)
    }

    #[allow(unused_variables)]
    pub fn claim_social_fee_pda(ctx: Context<ClaimSocialFeePda>, user_id: String, platform: u8) -> Result<Option<SocialFeePdaClaimed>> {
        social::claim(ctx, user_id, platform)
    }

    #[allow(unused_variables)]
    pub fn claim_social_fee_pda_v2(ctx: Context<ClaimSocialFeePdaV2>, user_id: String, platform: u8) -> Result<Option<SocialFeePdaClaimed>> {
        social::claim_v2(ctx, user_id, platform)
    }

    #[allow(unused_variables)]
    pub fn crank_donation_fee_pda(ctx: Context<CrankDonationFeePda>) -> Result<()> {
        donation::crank(ctx)
    }

    #[allow(unused_variables)]
    pub fn create_donation_fee_pda(ctx: Context<CreateDonationFeePda>) -> Result<()> {
        donation::create(ctx)
    }

    #[allow(unused_variables)]
    pub fn create_fee_sharing_config(ctx: Context<CreateFeeSharingConfig>) -> Result<()> {
        sharing::create_sharing(ctx)
    }

    #[allow(unused_variables)]
    pub fn create_social_fee_pda(ctx: Context<CreateSocialFeePda>, user_id: String, platform: u8) -> Result<()> {
        social::create_social(ctx, user_id, platform)
    }

    #[allow(unused_variables)]
    pub fn extend_fee_config(ctx: Context<ExtendFeeConfig>) -> Result<()> {
        let info = ctx.accounts.fee_config.to_account_info();
        let data = info.try_borrow_data()?;
        require!(info.owner == &ID && data.len() >= 9 && data[..8] == [143,52,146,187,219,123,76,155], FeeError::InvalidFeeConfigAccount);
        let bump = [data[8]];
        let config_program = ctx.accounts.config_program_id.key();
        let expected = Pubkey::create_program_address(&[b"fee_config", config_program.as_ref(), &bump], &ID)
            .map_err(|_| error!(FeeError::InvalidFeeConfigAccount))?;
        require_keys_eq!(expected, info.key(), FeeError::InvalidFeeConfigAccount);
        let current_size = data.len();
        drop(data);
        if current_size >= 4097 { return Ok(()); }
        let rent = Rent::get()?;
        let amount = rent.minimum_balance(4097) - rent.minimum_balance(current_size);
        anchor_lang::system_program::transfer(CpiContext::new(
            ctx.accounts.system_program.to_account_info(), anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(), to: info.clone(),
            }), amount)?;
        info.realloc(4097, true)?;
        let event = ExtendFeeConfigEvent { fee_config: info.key(), user: ctx.accounts.user.key(),
            current_size: current_size as u64, new_size: 4097, timestamp: Clock::get()?.unix_timestamp };
        emit!(event); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn get_fees(ctx: Context<GetFees>, is_pump_pool: bool, market_cap_lamports: u128, trade_size_lamports: u64, is_new_quote_mint: OptionBool) -> Result<Fees> {
        let config = &ctx.accounts.fee_config;
        if !is_pump_pool { return Ok(config.flat_fees.clone()); }
        Ok(select_fee_tier(if is_new_quote_mint.0 { &config.stable_fee_tiers } else { &config.fee_tiers }, market_cap_lamports))
    }

    #[allow(unused_variables)]
    pub fn get_fees_with_quote_mint(ctx: Context<GetFeesWithQuoteMint>, is_pump_pool: bool, market_cap_lamports: u128, quote_mint: Pubkey) -> Result<Fees> {
        let config = &ctx.accounts.fee_config;
        if !is_pump_pool { return Ok(config.flat_fees.clone()); }
        let quote = quote_mint.to_bytes();
        if quote == [0; 32] || quote == WRAPPED_SOL || quote == OTHER_NORMAL_QUOTE {
            Ok(select_fee_tier(&config.fee_tiers, market_cap_lamports))
        } else if quote == USDC {
            Ok(select_fee_tier(&config.stable_fee_tiers, market_cap_lamports))
        } else if config.exotic_flat_fees.lp_fee_bps == 0 && config.exotic_flat_fees.protocol_fee_bps == 0 && config.exotic_flat_fees.creator_fee_bps == 0 {
            Ok(config.flat_fees.clone())
        } else { Ok(config.exotic_flat_fees.clone()) }
    }

    #[allow(unused_variables)]
    pub fn initialize_buyback(ctx: Context<InitializeBuyback>, index: u8) -> Result<()> {
        buyback::initialize_buyback(ctx,index)
    }

    #[allow(unused_variables)]
    pub fn initialize_fee_config(ctx: Context<InitializeFeeConfig>) -> Result<()> {
        ctx.accounts.fee_config.bump = ctx.bumps.fee_config;
        ctx.accounts.fee_config.admin = ctx.accounts.admin.key();
        let event = InitializeFeeConfigEvent {
            timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.admin.key(),
            fee_config: ctx.accounts.fee_config.key(),
        };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn initialize_fee_program_global(ctx: Context<InitializeFeeProgramGlobal>, social_claim_authority: Pubkey, disable_flags: u8, claim_rate_limit: u64) -> Result<()> {
        let state = &mut ctx.accounts.fee_program_global;
        state.bump = ctx.bumps.fee_program_global;
        state.authority = ctx.accounts.authority.key();
        state.social_claim_authority = social_claim_authority;
        state.disable_flags = disable_flags;
        state.claim_rate_limit = claim_rate_limit;
        let event = InitializeFeeProgramGlobalEvent {
            timestamp: Clock::get()?.unix_timestamp, authority: ctx.accounts.authority.key(),
            social_claim_authority, disable_flags, claim_rate_limit,
        };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn revoke_fee_sharing_authority(_ctx: Context<RevokeFeeSharingAuthority>) -> Result<()> {
        Err(error!(FeeError::DeprecatedInstruction))
    }

    #[allow(unused_variables)]
    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        require!(new_authority != Pubkey::default(), FeeError::InvalidAdmin);
        let old_authority = ctx.accounts.fee_program_global.authority;
        ctx.accounts.fee_program_global.authority = new_authority;
        let event = SetAuthorityEvent { timestamp: Clock::get()?.unix_timestamp, old_authority, new_authority };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_claim_rate_limit(ctx: Context<SetClaimRateLimit>, claim_rate_limit: u64) -> Result<()> {
        ctx.accounts.fee_program_global.claim_rate_limit = claim_rate_limit;
        let event = SetClaimRateLimitEvent { timestamp: Clock::get()?.unix_timestamp, claim_rate_limit };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_disable_flags(ctx: Context<SetDisableFlags>, disable_flags: u8) -> Result<()> {
        ctx.accounts.fee_program_global.disable_flags = disable_flags;
        let event = SetDisableFlagsEvent { timestamp: Clock::get()?.unix_timestamp, disable_flags };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_exotic_flat_fees(ctx: Context<SetExoticFlatFees>, exotic_flat_fees: Fees) -> Result<()> {
        validate_fees(&exotic_flat_fees)?;
        let info = ctx.accounts.fee_config.to_account_info();
        if info.data_len() < 4097 {
            let rent = Rent::get()?.minimum_balance(4097);
            let extra = rent.saturating_sub(info.lamports());
            if extra > 0 {
                anchor_lang::system_program::transfer(CpiContext::new(
                    ctx.accounts.system_program.to_account_info(), anchor_lang::system_program::Transfer {
                        from: ctx.accounts.admin.to_account_info(), to: info.clone(),
                    }), extra)?;
            }
            info.realloc(4097, true)?;
        }
        ctx.accounts.fee_config.exotic_flat_fees = exotic_flat_fees.clone();
        let event = SetExoticFlatFeesEvent { timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.admin.key(), fee_config: ctx.accounts.fee_config.key(), exotic_flat_fees };
        emit!(event.clone()); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn set_social_claim_authority(ctx: Context<SetSocialClaimAuthority>, social_claim_authority: Pubkey) -> Result<()> {
        ctx.accounts.fee_program_global.social_claim_authority = social_claim_authority;
        let event = SetSocialClaimAuthorityEvent { timestamp: Clock::get()?.unix_timestamp, social_claim_authority };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn sweep_buyback(ctx: Context<SweepBuyback>, index: u8) -> Result<()> {
        buyback::sweep_buyback(ctx,index)
    }

    #[allow(unused_variables)]
    pub fn transfer_fee_sharing_authority(_ctx: Context<TransferFeeSharingAuthority>) -> Result<()> {
        Err(error!(FeeError::DeprecatedInstruction))
    }

    #[allow(unused_variables)]
    pub fn update_admin(ctx: Context<UpdateAdmin>) -> Result<()> {
        let old_admin = ctx.accounts.fee_config.admin;
        ctx.accounts.fee_config.admin = ctx.accounts.new_admin.key();
        let event = UpdateAdminEvent { timestamp: Clock::get()?.unix_timestamp, old_admin, new_admin: ctx.accounts.fee_config.admin };
        emit!(event); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn update_buyback_authority(ctx: Context<UpdateBuyback>, index: u8, new_authority: Pubkey) -> Result<()> {
        buyback::update_buyback_authority(ctx,index,new_authority)
    }

    #[allow(unused_variables)]
    pub fn update_buyback_claim_rate_limit(ctx: Context<UpdateBuyback>, index: u8, claim_rate_limit: i64) -> Result<()> {
        buyback::update_buyback_claim_rate_limit(ctx,index,claim_rate_limit)
    }

    #[allow(unused_variables)]
    pub fn update_fee_config(ctx: Context<UpdateFeeConfig>, fee_tiers: Vec<FeeTier>, flat_fees: Fees) -> Result<()> {
        validate_fees(&flat_fees)?;
        validate_tiers(&fee_tiers)?;
        ctx.accounts.fee_config.fee_tiers = fee_tiers.clone();
        ctx.accounts.fee_config.flat_fees = flat_fees.clone();
        let event = UpdateFeeConfigEvent { timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.admin.key(), fee_config: ctx.accounts.fee_config.key(), fee_tiers, flat_fees };
        emit!(event.clone()); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn update_fee_shares<'info>(ctx: Context<'_, '_, '_, 'info, UpdateFeeShares<'info>>, shareholders: Vec<Shareholder>) -> Result<()> {
        sharing_update::update_fee_shares(ctx, shareholders)
    }

    #[allow(unused_variables)]
    pub fn update_fee_shares_v2<'info>(ctx: Context<'_, '_, '_, 'info, UpdateFeeSharesV2<'info>>, shareholders: Vec<Shareholder>) -> Result<()> {
        sharing_update::update_fee_shares_v2(ctx, shareholders)
    }

    #[allow(unused_variables)]
    pub fn update_stable_fee_config(ctx: Context<UpdateStableFeeConfig>, fee_tiers: Vec<FeeTier>) -> Result<()> {
        validate_tiers(&fee_tiers)?;
        ctx.accounts.fee_config.stable_fee_tiers = fee_tiers.clone();
        let event = UpdateStableFeeConfigEvent { timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.admin.key(), fee_config: ctx.accounts.fee_config.key(),
            stable_fee_tiers: fee_tiers, flat_fees: ctx.accounts.fee_config.flat_fees.clone() };
        emit!(event.clone()); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn upsert_fee_tiers(ctx: Context<UpsertFeeTiers>, fee_tiers: Vec<FeeTier>, offset: u8) -> Result<()> {
        upsert_tiers(&mut ctx.accounts.fee_config.fee_tiers, &fee_tiers, offset)?;
        let event = UpsertFeeTiersEvent { timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.admin.key(), fee_config: ctx.accounts.fee_config.key(), fee_tiers, offset };
        emit!(event.clone()); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn upsert_stable_fee_tiers(ctx: Context<UpsertStableFeeTiers>, fee_tiers: Vec<FeeTier>, offset: u8) -> Result<()> {
        upsert_tiers(&mut ctx.accounts.fee_config.stable_fee_tiers, &fee_tiers, offset)?;
        let event = UpsertStableFeeTiersEvent { timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.admin.key(), fee_config: ctx.accounts.fee_config.key(), stable_fee_tiers: fee_tiers, offset };
        emit!(event.clone()); emit_cpi!(event); Ok(())
    }

    #[allow(unused_variables)]
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        withdrawal::execute(ctx)
    }

}

#[error_code]
pub enum CandidateError {
    #[msg("Research candidate handler not reconstructed")]
    Unsupported,
}

fn select_fee_tier(tiers: &[FeeTier], cap: u128) -> Fees {
    if cap < tiers[0].market_cap_lamports_threshold { return tiers[0].fees.clone(); }
    for tier in tiers.iter().rev() {
        if cap >= tier.market_cap_lamports_threshold { return tier.fees.clone(); }
    }
    tiers[0].fees.clone()
}
pub const WRAPPED_SOL: [u8; 32] = [6,155,136,87,254,171,129,132,251,104,127,99,70,24,192,53,218,196,57,220,26,235,59,85,152,160,240,0,0,0,0,1];
pub const OTHER_NORMAL_QUOTE: [u8; 32] = [131,13,252,159,222,95,230,184,170,124,4,164,118,233,30,138,198,187,38,74,173,144,250,25,201,223,73,216,92,62,91,94];
pub const USDC: [u8; 32] = [198,250,122,243,190,219,173,58,61,101,243,106,171,201,116,49,177,187,228,194,210,246,224,228,124,166,2,3,69,47,93,97];

#[error_code]
pub enum FeeError {
    #[msg("Only Pump and PumpSwap programs can call this instruction")]
    UnauthorizedProgram = 0,
    #[msg("Invalid admin")]
    InvalidAdmin = 1,
    #[msg("No fee tiers provided")]
    NoFeeTiers = 2,
    #[msg("format")]
    TooManyFeeTiers = 3,
    #[msg("The offset should be <= fee_config.fee_tiers.len()")]
    OffsetNotContinuous = 4,
    #[msg("Fee tiers must be sorted by market cap threshold (ascending)")]
    FeeTiersNotSorted = 5,
    #[msg("Fee total must not exceed 10_000bps")]
    InvalidFeeTotal = 6,
    #[msg("Invalid Sharing Config")]
    InvalidSharingConfig = 7,
    #[msg("Invalid Pool")]
    InvalidPool = 8,
    #[msg("Sharing config authority has been revoked - sharing config can only be updated once")]
    SharingConfigAdminRevoked = 9,
    #[msg("No shareholders provided")]
    NoShareholders = 10,
    #[msg("format")]
    TooManyShareholders = 11,
    #[msg("Duplicate shareholder address")]
    DuplicateShareholder = 12,
    #[msg("Not enough remaining accounts")]
    NotEnoughRemainingAccounts = 13,
    #[msg("Invalid share total - must equal 10_000 basis points")]
    InvalidShareTotal = 14,
    #[msg("Share calculation overflow")]
    ShareCalculationOverflow = 15,
    #[msg("The given account is not authorized to execute this instruction.")]
    NotAuthorized = 16,
    #[msg("Shareholder cannot have zero share")]
    ZeroShareNotAllowed = 17,
    #[msg("Fee sharing config is not active")]
    SharingConfigNotActive = 18,
    #[msg("AMM accounts are required for graduated coins")]
    AmmAccountsRequiredForGraduatedCoin = 19,
    #[msg("Remaining account key doesn't match shareholder address")]
    ShareholderAccountMismatch = 20,
    #[msg("Feature is currently deactivated")]
    FeatureDeactivated = 21,
    #[msg("User ID exceeds maximum length")]
    UserIdTooLong = 22,
    #[msg("Instruction is deprecated")]
    DeprecatedInstruction = 23,
    #[msg("Reward split can only be updated once")]
    FeeSharesAlreadyUpdated = 24,
    #[msg("Math overflow")]
    MathOverflow = 25,
    #[msg("Invalid buybackindex")]
    InvalidBuybackIndex = 26,
    #[msg("Claim rate limit exceeded")]
    ClaimRateLimitExceeded = 27,
    #[msg("Account is not a valid FeeConfig for this instruction")]
    InvalidFeeConfigAccount = 28,
    #[msg("Account type not supported")]
    AccountTypeNotSupported = 29,
    #[msg("Mint does not match quote mint")]
    InvalidMint = 30,
    #[msg("Unsupported quote mint")]
    UnsupportedQuoteMint = 31,
    #[msg("Invalid remaining accounts")]
    InvalidRemainingAccounts = 32,
    #[msg("Pool creator_fees must be swept before the shares change")]
    PoolCreatorFeesNotSwept = 33,
    #[msg("quote_mint is not the bonding curve's quote mint")]
    InvalidQuoteMint = 34,
    #[msg("Token withdrawals need quote_mint, vault_ata, user_ata and token_program")]
    IncompleteTokenAccounts = 35,
    #[msg("Nothing to withdraw")]
    NothingToWithdraw = 36,
}

fn validate_fees(fees: &Fees) -> Result<()> {
    // The original aborts on addition overflow before comparing the total.
    let total = fees.lp_fee_bps.checked_add(fees.protocol_fee_bps).unwrap()
        .checked_add(fees.creator_fee_bps).unwrap();
    require!(total <= 10_000, FeeError::InvalidFeeTotal);
    Ok(())
}
fn validate_tiers(tiers: &[FeeTier]) -> Result<()> {
    require!(!tiers.is_empty(), FeeError::NoFeeTiers);
    require!(tiers.len() <= 50, FeeError::TooManyFeeTiers);
    for tier in tiers { validate_fees(&tier.fees)?; }
    require!(tiers.windows(2).all(|pair| pair[0].market_cap_lamports_threshold < pair[1].market_cap_lamports_threshold), FeeError::FeeTiersNotSorted);
    Ok(())
}

fn upsert_tiers(current: &mut Vec<FeeTier>, incoming: &[FeeTier], offset: u8) -> Result<()> {
    require!(!incoming.is_empty(), FeeError::NoFeeTiers);
    let offset = offset as usize;
    require!(offset <= current.len(), FeeError::OffsetNotContinuous);
    require!(current.len().max(offset + incoming.len()) <= 50, FeeError::TooManyFeeTiers);
    for (index, tier) in incoming.iter().enumerate() {
        if offset + index < current.len() { current[offset + index] = tier.clone(); }
        else { current.push(tier.clone()); }
    }
    validate_tiers(current)
}
