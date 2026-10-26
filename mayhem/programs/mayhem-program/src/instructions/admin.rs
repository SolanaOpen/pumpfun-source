use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};
use crate::checked_program::{CheckedSystemProgram, CheckedSystemProgramBumps, __client_accounts_checked_system_program, __cpi_client_accounts_checked_system_program};
use crate::state::GlobalParams;
use crate::MayhemError;

#[derive(Accounts)]
pub struct Admin<'info> { #[account(mut)] pub admin: Signer<'info>, #[account(mut, seeds = [b"global-params"], bump, has_one = admin)] pub global_params: Account<'info, GlobalParams>, pub system_program: Program<'info, System>, #[doc = " CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error."] #[account(seeds = [b"__event_authority"], bump)] pub event_authority: UncheckedAccount<'info>, #[doc = " CHECK: Captured parser consumes this account without an address constraint."] pub program: UncheckedAccount<'info>, }
#[derive(Accounts)]
pub struct InitializeGlobalParams<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 254, seeds = [b"global-params"], bump)]
    pub global_params: Account<'info, GlobalParams>,
    pub system_program: Program<'info, System>,
}

// (layout padding)
// (layout padding)

// The discriminator is observed from original emitted data. Historical event
// type name remains unresolved; overriding it avoids pretending to know it.
// (layout padding)
// (layout padding)
#[event(discriminator = [0x63,0xa7,0x5c,0x98,0x98,0xe3,0x26,0x29])]
pub struct GlobalParamsUpdateEvent {
    pub old_version: u32,
    pub new_version: u32,
    pub field_0: u32,
    pub field_1: u32,
    pub field_2: u64,
    pub field_3: u64,
    pub field_4: u64,
    pub field_5: u64,
    pub field_6: u8,
    pub field_7: u64,
    pub field_8: u64,
    pub field_9: u64,
    pub field_10: u64,
    pub field_11: u64,
    pub field_12: u64,
    pub field_13: u64,
    pub field_14: u64,
    pub field_15: u64,
}

/// Recovered serialized widths/order. Semantic names remain unresolved.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateGlobalParamsArgs {
    pub field_0: Option<u32>,
    pub field_1: Option<u32>,
    pub field_2: Option<u64>,
    pub field_3: Option<u64>,
    pub field_4: Option<u64>,
    pub field_5: Option<u64>,
    pub field_6: Option<u8>,
    pub field_7: Option<u64>,
    pub field_8: Option<u64>,
    pub field_9: Option<u64>,
    pub field_10: Option<u64>,
    pub field_11: Option<u64>,
    pub field_12: Option<u64>,
    pub field_13: Option<u64>,
    pub field_14: Option<u64>,
    pub field_15: Option<u64>,
}

// (layout padding)
// (layout padding)
// (layout padding)
// (layout padding)
// (layout padding)
// (layout padding)

#[derive(Accounts)]
pub struct WithdrawAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"global-params"], bump, has_one = admin)]
    pub global_params: Account<'info, GlobalParams>,
    /// CHECK: PDA/writability enforced here; System CPI enforces spending owner.
    #[account(mut, seeds = [b"sol-vault"], bump)]
    pub sol_vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawQuoteAdmin<'info> {
    #[account(mut)] pub admin: Signer<'info>,
    #[account(mut, seeds = [b"global-params"], bump, has_one = admin)]
    pub global_params: Account<'info, GlobalParams>,
    /// CHECK: PDA authorizes its token accounts.
    #[account(mut, seeds = [b"sol-vault"], bump)] pub sol_vault: UncheckedAccount<'info>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint = quote_mint, associated_token::authority = sol_vault, associated_token::token_program = token_program)]
    pub sol_vault_quote_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(init_if_needed, payer = admin, associated_token::mint = quote_mint, associated_token::authority = admin, associated_token::token_program = token_program)]
    pub admin_quote_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_global_params(ctx: Context<InitializeGlobalParams>) -> Result<()> {
    ctx.accounts.global_params.admin = ctx.accounts.admin.key();
    Ok(())
}
pub fn update_global_params(ctx: Context<Admin>, field_0: Option<u32>, field_1: Option<u32>, field_2: Option<u64>, field_3: Option<u64>, field_4: Option<u64>, field_5: Option<u64>, field_6: Option<u8>, field_7: Option<u64>, field_8: Option<u64>, field_9: Option<u64>, field_10: Option<u64>, field_11: Option<u64>, field_12: Option<u64>, field_13: Option<u64>, field_14: Option<u64>, field_15: Option<u64>) -> Result<()> {
    let state = &mut ctx.accounts.global_params;
    let old_version = state.version;
    apply_update(state, field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7, field_8, field_9, field_10, field_11, field_12, field_13, field_14, field_15)?;
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    let event = GlobalParamsUpdateEvent {
        old_version, new_version: state.version,
        field_0: u32::from_le_bytes(state.reset_prefix[0..4].try_into().unwrap()),
        field_1: u32::from_le_bytes(state.reset_prefix[4..8].try_into().unwrap()),
        field_2: u64::from_le_bytes(state.reset_prefix[8..16].try_into().unwrap()),
        field_3: u64::from_le_bytes(state.reset_prefix[16..24].try_into().unwrap()),
        field_4: u64::from_le_bytes(state.reset_prefix[24..32].try_into().unwrap()),
        field_5: u64::from_le_bytes(state.reset_prefix[32..40].try_into().unwrap()),
        field_6: state.reset_middle[0],
        field_7: u64::from_le_bytes(state.reset_middle[9..17].try_into().unwrap()),
        field_8: u64::from_le_bytes(state.reset_middle[17..25].try_into().unwrap()),
        field_9: u64::from_le_bytes(state.preserved_middle_a),
        field_10: state.field_10,
        field_11: u64::from_le_bytes(state.reset_stable_prefix[0..8].try_into().unwrap()),
        field_12: u64::from_le_bytes(state.reset_stable_prefix[8..16].try_into().unwrap()),
        field_13: u64::from_le_bytes(state.reset_stable_prefix[16..24].try_into().unwrap()),
        field_14: u64::from_le_bytes(state.reset_stable_suffix[0..8].try_into().unwrap()),
        field_15: u64::from_le_bytes(state.reset_stable_suffix[8..16].try_into().unwrap()),
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
pub fn update_admin(ctx: Context<Admin>, new_admin: Pubkey) -> Result<()> {
    ctx.accounts.global_params.admin = new_admin;
    Ok(())
}
pub fn update_trader(ctx: Context<Admin>, new_trader: Pubkey) -> Result<()> {
    ctx.accounts.global_params.trader = new_trader;
    Ok(())
}
pub fn update_max_sol_per_sweep(ctx: Context<Admin>, value: u64) -> Result<()> {
    ctx.accounts.global_params.max_sol_per_sweep = value;
    Ok(())
}
pub fn update_max_stable_per_sweep(ctx: Context<Admin>, value: u64) -> Result<()> {
    ctx.accounts.global_params.max_stable_per_sweep = value;
    Ok(())
}
pub fn reset_sol_deployed(ctx: Context<Admin>) -> Result<()> {
    ctx.accounts.global_params.sol_deployed = 0;
    Ok(())
}
pub fn reset_stable_deployed(ctx: Context<Admin>) -> Result<()> {
    ctx.accounts.global_params.stable_deployed = 0;
    Ok(())
}
pub fn reset_state(ctx: Context<Admin>) -> Result<()> {
    let state = &mut ctx.accounts.global_params;
    state.reset_prefix = [0; 40];
    state.paused = false;
    state.sol_deployed = 0;
    state.reset_middle = [0; 25];
    state.reset_stable_prefix = [0; 24];
    state.stable_deployed = 0;
    state.reset_stable_suffix = [0; 24];
    Ok(())
}
pub fn stop_trading(ctx: Context<Admin>) -> Result<()> {
    require_eq!(ctx.accounts.global_params.paused, false, MayhemError::TradingPaused);
    ctx.accounts.global_params.paused = true;
    Ok(())
}
pub fn resume_trading(ctx: Context<Admin>) -> Result<()> {
    // The original also uses 6000/TradingPaused when already active.
    require_eq!(ctx.accounts.global_params.paused, true, MayhemError::TradingPaused);
    ctx.accounts.global_params.paused = false;
    Ok(())
}
pub fn withdraw_quote_admin(ctx: Context<WithdrawQuoteAdmin>, amount: u64) -> Result<()> {
    quote_withdraw_rate_limit(&mut ctx.accounts.global_params)?;
    anchor_spl::token::transfer(CpiContext::new_with_signer(ctx.accounts.token_program.key(), anchor_spl::token::Transfer {
            from: ctx.accounts.sol_vault_quote_ata.to_account_info(),
            to: ctx.accounts.admin_quote_ata.to_account_info(),
            authority: ctx.accounts.sol_vault.to_account_info(),
        }, &[&[b"sol-vault", &[ctx.bumps.sol_vault]]]), amount)?;
    Ok(())
}
pub fn withdraw_admin(ctx: Context<WithdrawAdmin>, amount: u64) -> Result<()> {
    sol_withdraw_rate_limit(&mut ctx.accounts.global_params)?;
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    // (layout padding)
    let bump = [ctx.bumps.sol_vault];
    let seeds: &[&[u8]] = &[b"sol-vault", &bump];
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(ctx.accounts.system_program.key(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.sol_vault.to_account_info(),
                to: ctx.accounts.admin.to_account_info(),
            }, &[seeds]), amount)
}

#[inline(never)]
fn sol_withdraw_rate_limit(state: &mut GlobalParams) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let last = i64::from_le_bytes(state.reset_middle[1..9].try_into().unwrap());
    let interval = i64::from_le_bytes(state.reset_middle[17..25].try_into().unwrap());
    if last != 0 && interval != 0 {
        require_gte!(now, last + interval, MayhemError::WithdrawRateLimit);
    }
    state.reset_middle[1..9].copy_from_slice(&now.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn quote_withdraw_rate_limit(state: &mut GlobalParams) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let last = i64::from_le_bytes(state.reset_stable_suffix[16..24].try_into().unwrap());
    let interval = i64::from_le_bytes(state.reset_stable_suffix[8..16].try_into().unwrap());
    GlobalParams::quote_withdraw_check(now, last, interval)?;
    state.reset_stable_suffix[16..24].copy_from_slice(&now.to_le_bytes());
    Ok(())
}

#[inline(never)]
fn apply_update(state: &mut GlobalParams, field_0: Option<u32>, field_1: Option<u32>, field_2: Option<u64>, field_3: Option<u64>, field_4: Option<u64>, field_5: Option<u64>, field_6: Option<u8>, field_7: Option<u64>, field_8: Option<u64>, field_9: Option<u64>, field_10: Option<u64>, field_11: Option<u64>, field_12: Option<u64>, field_13: Option<u64>, field_14: Option<u64>, field_15: Option<u64>) -> Result<()> { let args = UpdateGlobalParamsArgs { field_0, field_1, field_2, field_3, field_4, field_5, field_6, field_7, field_8, field_9, field_10, field_11, field_12, field_13, field_14, field_15 };
    if let Some(v) = args.field_0 { state.reset_prefix[0..4].copy_from_slice(&v.to_le_bytes()); }
    if let Some(v) = args.field_1 { state.reset_prefix[4..8].copy_from_slice(&v.to_le_bytes()); }
    for (value, offset) in [(args.field_2, 8), (args.field_3, 16), (args.field_4, 24), (args.field_5, 32)] {
        if let Some(v) = value { state.reset_prefix[offset..offset+8].copy_from_slice(&v.to_le_bytes()); }
    }
    if let Some(v) = args.field_6 { state.reset_middle[0] = v; }
    if let Some(v) = args.field_7 { state.reset_middle[9..17].copy_from_slice(&v.to_le_bytes()); }
    if let Some(v) = args.field_8 { state.reset_middle[17..25].copy_from_slice(&v.to_le_bytes()); }
    if let Some(v) = args.field_9 { state.preserved_middle_a.copy_from_slice(&v.to_le_bytes()); }
    if let Some(v) = args.field_10 { state.field_10 = v; }
    for (value, offset) in [(args.field_11, 0), (args.field_12, 8), (args.field_13, 16)] {
        if let Some(v) = value { state.reset_stable_prefix[offset..offset+8].copy_from_slice(&v.to_le_bytes()); }
    }
    if let Some(v) = args.field_14 { state.reset_stable_suffix[0..8].copy_from_slice(&v.to_le_bytes()); }
    if let Some(v) = args.field_15 { state.reset_stable_suffix[8..16].copy_from_slice(&v.to_le_bytes()); }
    state.version = state.version.checked_add(1).ok_or(MayhemError::MathOverflow)?;
    Ok(())
}
