//! Recovered referral withdrawal; original executable remains the comparison target.
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::FeeError;
 pub fn execute(ctx:Context<Withdraw>)->Result<()> {
  let user=ctx.accounts.user.key();let bump=[ctx.bumps.referral_vault];
  let seeds:&[&[u8]]=&[b"referral-vault",user.as_ref(),&bump];
  let amount;let mint;
  if let Some(quote)=ctx.accounts.quote_mint.as_ref() {
   let vault=ctx.accounts.vault_ata.as_ref().ok_or(FeeError::IncompleteTokenAccounts)?;
   let destination=ctx.accounts.user_ata.as_ref().ok_or(FeeError::IncompleteTokenAccounts)?;
   let token=ctx.accounts.token_program.as_ref().ok_or(FeeError::IncompleteTokenAccounts)?;
   amount=vault.amount;require!(amount>0,FeeError::NothingToWithdraw);mint=quote.key();
   anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(token.to_account_info(),anchor_spl::token_interface::TransferChecked{from:vault.to_account_info(),mint:quote.to_account_info(),to:destination.to_account_info(),authority:ctx.accounts.referral_vault.to_account_info()},&[seeds]),amount,quote.decimals)?;
  }else{
   amount=ctx.accounts.referral_vault.lamports().saturating_sub(Rent::get()?.minimum_balance(0));
   require!(amount>0,FeeError::NothingToWithdraw);mint=Pubkey::default();
   anchor_lang::system_program::transfer(CpiContext::new_with_signer(ctx.accounts.system_program.to_account_info(),anchor_lang::system_program::Transfer{from:ctx.accounts.referral_vault.to_account_info(),to:ctx.accounts.user.to_account_info()},&[seeds]),amount)?;
  }
  let event=WithdrawEvent{timestamp:Clock::get()?.unix_timestamp,user,quote_mint:mint,amount};
  emit!(event);emit_cpi!(event);Ok(())
 }
#[derive(Accounts)]
pub struct Withdraw<'info>{
 #[account(mut)]pub user:Signer<'info>,
 #[account(mut,seeds=[b"referral-vault",user.key().as_ref()],bump)]pub referral_vault:SystemAccount<'info>,
 pub quote_mint:Option<InterfaceAccount<'info,Mint>>,
 #[account(mut,token::mint=quote_mint,token::authority=referral_vault,token::token_program=token_program)]pub vault_ata:Option<InterfaceAccount<'info,TokenAccount>>,
 #[account(mut,token::mint=quote_mint,token::authority=user,token::token_program=token_program)]pub user_ata:Option<InterfaceAccount<'info,TokenAccount>>,
 pub token_program:Option<Interface<'info,TokenInterface>>,
 pub system_program:Program<'info,System>,
 /// CHECK: canonical event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: CPI receives this account; no typed program constraint observed.
 pub program:UncheckedAccount<'info>,
}
#[event]
pub struct WithdrawEvent{pub timestamp:i64,pub user:Pubkey,pub quote_mint:Pubkey,pub amount:u64}
