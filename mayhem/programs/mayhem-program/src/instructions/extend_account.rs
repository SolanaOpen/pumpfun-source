use anchor_lang::prelude::*;
use crate::checked_program::{CheckedSystemProgram, CheckedSystemProgramBumps, __client_accounts_checked_system_program, __cpi_client_accounts_checked_system_program};
use crate::state::GlobalParams;
use crate::instructions::fee_recipients::FeeRecipient;
use crate::{MayhemError, ID};

#[derive(Accounts)]
pub struct ExtendAccount<'info> {
    /// CHECK: Owner/mut checked here; recognized discriminator checked in handler.
    #[account(mut, owner = ID)]
    pub account: UncheckedAccount<'info>,
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}

#[event(discriminator = [0x61,0x61,0xd7,0x90,0x5d,0x92,0x16,0x7c])]
pub struct ExtendAccountEvent {
    pub account: Pubkey,
    pub user: Pubkey,
    pub old_size: u64,
    pub new_size: u64,
    pub timestamp: i64,
}

pub fn extend_account(ctx: Context<ExtendAccount>) -> Result<()> {
    let account = ctx.accounts.account.to_account_info();
    let old_size = account.data_len();
    let new_size = {
        let data = account.try_borrow_data()?;
        match data.get(..8) {
            Some(d) if d == GlobalParams::DISCRIMINATOR => 318,
            Some(d) if d == FeeRecipient::DISCRIMINATOR => 74,
            Some(d) if d == [0xb1,0xfd,0xbf,0x7d,0xcb,0x16,0x86,0x6b] => 170,
            _ => return err!(MayhemError::AccountTypeNotSupported),
        }
    };
    if old_size == new_size {
        msg!("Account already has the correct size");
        return Ok(());
    }
    if old_size > new_size {
        msg!("Account already has more than {} bytes: {}", new_size, old_size);
        return Ok(());
    }
    let required = Rent::get()?.minimum_balance(new_size).saturating_sub(account.lamports());
    if required > 0 {
        anchor_lang::system_program::transfer(
            CpiContext::new(ctx.accounts.system_program.key(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(), to: account.clone()
                }), required)?;
    }
    account.resize(new_size)?;
    let event = ExtendAccountEvent {
        account: account.key(), user: ctx.accounts.user.key(),
        old_size: old_size as u64, new_size: new_size as u64,
        timestamp: Clock::get()?.unix_timestamp,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
