use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};
use crate::checked_program::{CheckedSystemProgram, CheckedSystemProgramBumps, __client_accounts_checked_system_program, __cpi_client_accounts_checked_system_program};
use crate::state::GlobalParams;
use crate::{MayhemError, ID};

#[account]
pub struct FeeRecipient {
    pub bump: u8,
    pub index: u8,
    pub opaque: [u8; 32],
}

#[derive(Accounts)]
#[instruction(index: u8)]
pub struct InitializeFeeRecipient<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub wsol_mint: InterfaceAccount<'info, Mint>,
    #[account(init, payer = admin, space = 42, seeds = [b"fee-recipient".as_ref(), &[index]], bump)]
    pub fee_recipient: Account<'info, FeeRecipient>,
    #[account(init, payer = admin, associated_token::mint = wsol_mint, associated_token::authority = fee_recipient, associated_token::token_program = token_program)]
    pub fee_recipient_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, seeds = [b"global-params"], bump)]
    pub global_params: Account<'info, GlobalParams>,
    pub system_program: CheckedSystemProgram<'info>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[event(discriminator = [218,130,134,253,101,109,149,152])]
pub struct SweepEvent { pub amount: u64, pub timestamp: i64 }
fn sweep_remaining<'info>(remaining: &'info [AccountInfo<'info>], destination: &AccountInfo<'info>, destination_ata: &AccountInfo<'info>, token: &AccountInfo<'info>, mint: Pubkey, limit: u64, admin: bool) -> Result<u64> {
    require_eq!(remaining.len(),16,MayhemError::AccountTypeNotSupported);
    let rent = Rent::get()?;
    let mut total = 0u64;
    for pair in remaining.chunks_exact(2) {
        let mut recipient=Account::<FeeRecipient>::try_from(&pair[0])?;
        let ata=InterfaceAccount::<TokenAccount>::try_from(&pair[1])?;
        let before_total=total;
        let available = limit - total;
        let token_amount = ata.amount.min(available);
        if token_amount > 0 {
            let index=[recipient.index];let bump=[recipient.bump];let seeds: &[&[u8]]=&[b"fee-recipient", &index, &bump];
            anchor_spl::token::transfer(CpiContext::new_with_signer(*token.key, anchor_spl::token::Transfer { from: ata.to_account_info(), to: destination_ata.clone(), authority: recipient.to_account_info() }, &[seeds]),token_amount)?;
            total += token_amount;
        }
        if mint == anchor_spl::token::spl_token::native_mint::ID {
            let account = recipient.to_account_info();
            let amount = (account.lamports() - rent.minimum_balance(account.data_len())).min(limit-total);
            **account.try_borrow_mut_lamports()? -= amount;
            **destination.try_borrow_mut_lamports()? += amount;
            total += amount;
        }
        let (counter, timestamp) = if mint == anchor_spl::token::spl_token::native_mint::ID { (8, 0) } else { (16, 24) };
        let previous=u64::from_le_bytes(recipient.opaque[counter..counter+8].try_into().unwrap());
        recipient.opaque[counter..counter+8].copy_from_slice(&(previous + total-before_total).to_le_bytes());
        if !admin { recipient.opaque[timestamp..timestamp+8].copy_from_slice(&Clock::get()?.unix_timestamp.to_le_bytes()); }
        recipient.exit(&ID)?;
    }
    Ok(total)
}

#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)] pub trader: Signer<'info>,
    #[account(mut, seeds=[b"global-params"], bump, has_one=trader)] pub global_params: Account<'info, GlobalParams>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint=quote_mint, associated_token::authority=trader, associated_token::token_program=token_program)] pub trader_quote_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: CheckedSystemProgram<'info>,
    #[account(address=anchor_spl::token::ID)] pub token_program: Interface<'info,TokenInterface>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SweepFeesAdmin<'info> {
    #[account(mut)] pub admin: Signer<'info>,
    #[account(mut, seeds=[b"global-params"], bump, has_one=admin)] pub global_params: Account<'info, GlobalParams>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint=quote_mint, associated_token::authority=admin, associated_token::token_program=token_program)] pub admin_quote_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: CheckedSystemProgram<'info>,
    #[account(address=anchor_spl::token::ID)] pub token_program: Interface<'info,TokenInterface>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}

pub fn initialize_fee_recipient(ctx: Context<InitializeFeeRecipient>, index: u8) -> Result<()> {
    require!(index < 8, MayhemError::MaxFeeRecipientsExceeded);
    ctx.accounts.fee_recipient.bump = ctx.bumps.fee_recipient;
    ctx.accounts.fee_recipient.index = index;
    Ok(())
}
pub fn sweep_fees<'info>(ctx: Context<'info, SweepFees<'info>>) -> Result<()> {
    let limit = if ctx.accounts.quote_mint.key() == anchor_spl::token::spl_token::native_mint::ID { ctx.accounts.global_params.max_sol_per_sweep } else { ctx.accounts.global_params.max_stable_per_sweep };
    let amount = sweep_remaining(ctx.remaining_accounts, &ctx.accounts.trader.to_account_info(), &ctx.accounts.trader_quote_ata.to_account_info(), &ctx.accounts.token_program.to_account_info(), ctx.accounts.quote_mint.key(), limit, false)?;
    let event = SweepEvent { amount, timestamp: Clock::get()?.unix_timestamp }; emit!(event); emit_cpi!(event); Ok(())
}
pub fn sweep_fees_admin<'info>(ctx: Context<'info, SweepFeesAdmin<'info>>) -> Result<()> {
    let amount = sweep_remaining(ctx.remaining_accounts, &ctx.accounts.admin.to_account_info(), &ctx.accounts.admin_quote_ata.to_account_info(), &ctx.accounts.token_program.to_account_info(), ctx.accounts.quote_mint.key(), u64::MAX, true)?;
    let event = SweepEvent { amount, timestamp: Clock::get()?.unix_timestamp }; emit!(event); emit_cpi!(event); Ok(())
}
