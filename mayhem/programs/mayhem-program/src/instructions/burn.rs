use anchor_lang::prelude::*;
use crate::checked_program::{CheckedSystemProgram, CheckedSystemProgramBumps, __client_accounts_checked_system_program, __cpi_client_accounts_checked_system_program};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use crate::state::{BondingCurve, MayhemState, MayhemStatus, PUMP_ID};

// Burn instruction: burns the vault's remaining tokens once mayhem mode is over.
//
// Event and error definitions come first.

#[event]
pub struct BurnEvent {
    pub amount: u64,
    pub mint: Pubkey,
    pub mayhem_state: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum BurnError {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Invalid mint")]
    InvalidMint,
    #[msg("Invalid state")]
    InvalidState,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Mayhem mode ended")]
    MayhemModeEnded,
    #[msg("Mayhem mode not ended")]
    MayhemModeNotEnded,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut, seeds = [b"mayhem-state", mint.key().as_ref()], bump)]
    pub mayhem_state: Account<'info, MayhemState>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = sol_vault,
        associated_token::token_program = mint.to_account_info().owner)]
    pub mayhem_token_vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Address and writable flag are checked below; original parser uses an unchecked account.
    #[account(mut, seeds = [b"sol-vault"], bump)]
    pub sol_vault: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"bonding-curve", mint.key().as_ref()], bump, seeds::program = PUMP_ID)]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,
    /// CHECK: The captured burn parser consumes this account without further constraints.
    pub global: UncheckedAccount<'info>,
    /// CHECK: The captured parser checks its address; it does not require an executable Program wrapper.
    #[account(address = PUMP_ID)]
    pub pump_program: UncheckedAccount<'info>,
    /// CHECK: The captured burn parser consumes this account without further constraints.
    pub pump_event_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Original parser derives this PDA at runtime and returns ConstraintSeeds.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Original parser only consumes the program account.
    pub program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<BurnTokens>) -> Result<()> {
    // Burn flow:
    //  1. mayhem mode must have reached its end timestamp (6005 otherwise)
    //  2. a second burn is rejected (6004)
    //  3. the full vault balance is burned with the sol-vault PDA as signer
    //  4. the burned amount is recorded
    //  5. a BurnEvent is emitted via log and via self-CPI
    //  6. the state is marked as ended
    //
    // The binary reads Clock independently for each use.
    //
    let current_unix_timestamp: i64 = Clock::get().unwrap().unix_timestamp;
    require!(ctx.accounts.mayhem_state.end_timestamp <= current_unix_timestamp, BurnError::MayhemModeNotEnded);
    //
    // A second burn is rejected.
    //
    require!(ctx.accounts.mayhem_state.status != MayhemStatus::Ended, BurnError::MayhemModeEnded);
    let amount = ctx.accounts.mayhem_token_vault.amount;
    token_interface::burn(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        token_interface::Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.mayhem_token_vault.to_account_info(),
            authority: ctx.accounts.sol_vault.to_account_info(),
        }, &[&[b"sol-vault", &[ctx.bumps.sol_vault]]]), amount)?;
    ctx.accounts.mayhem_state.burned_amount = amount;
    //
    // The binary reads Clock independently for the two event paths.
    //
    // First the event is logged via sol_log_data,
    // then it is emitted through the event authority
    // self-CPI.
    //
    //
    emit!(BurnEvent {
        amount,
        mint: ctx.accounts.mint.key(),
        mayhem_state: ctx.accounts.mayhem_state.key(),
        timestamp: Clock::get().unwrap().unix_timestamp,
    });
    emit_cpi!(BurnEvent {
        amount,
        mint: ctx.accounts.mint.key(),
        mayhem_state: ctx.accounts.mayhem_state.key(),
        timestamp: Clock::get().unwrap().unix_timestamp,
    });
    ctx.accounts.mayhem_state.status = MayhemStatus::Ended;
    Ok(())
}
