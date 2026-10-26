//! Partial buyback reconstruction from signed local differential evidence.
use anchor_lang::prelude::*;
use crate::FeeError;
use crate::types::FeeProgramGlobal;
 pub fn initialize_buyback(_ctx:Context<InitializeBuyback>,index:u8)->Result<()>{require_gt!(8u8,index,FeeError::InvalidBuybackIndex);Ok(())}
 pub fn update_buyback_authority(ctx:Context<UpdateBuyback>,index:u8,new_authority:Pubkey)->Result<()>{require_gt!(8u8,index,FeeError::InvalidBuybackIndex);ctx.accounts.buyback_vault.authority=new_authority;Ok(())}
 pub fn update_buyback_claim_rate_limit(ctx:Context<UpdateBuyback>,index:u8,claim_rate_limit:i64)->Result<()>{require_gt!(8u8,index,FeeError::InvalidBuybackIndex);ctx.accounts.buyback_vault.claim_rate_limit=claim_rate_limit;Ok(())}
 pub fn sweep_buyback(ctx:Context<SweepBuyback>,index:u8)->Result<()>{
  require!(index<8,FeeError::InvalidBuybackIndex);
  let now=Clock::get()?.unix_timestamp;let vault=&mut ctx.accounts.buyback_vault;
  require!(vault.claim_rate_limit==0 || now-vault.last_claimed>vault.claim_rate_limit,FeeError::ClaimRateLimitExceeded);
  let sol_amount=vault.to_account_info().lamports().saturating_sub(Rent::get()?.minimum_balance(vault.to_account_info().data_len()));let token_amount=ctx.accounts.buyback_vault_ata.amount;
  let index_seed=[index];let bump=[ctx.bumps.buyback_vault];let seeds:&[&[u8]]=&[b"buyback-vault",&index_seed,&bump];
  anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(),anchor_spl::token_interface::TransferChecked{from:ctx.accounts.buyback_vault_ata.to_account_info(),mint:ctx.accounts.mint.to_account_info(),to:ctx.accounts.destination_ata.to_account_info(),authority:vault.to_account_info()},&[seeds]),token_amount,ctx.accounts.mint.decimals)?;
  vault.total_claimed=vault.total_claimed.checked_add(sol_amount).ok_or(error!(FeeError::MathOverflow))?;
  if ctx.accounts.mint.key()==anchor_spl::token::spl_token::native_mint::ID{vault.total_claimed_token1=vault.total_claimed_token1.checked_add(token_amount).ok_or(error!(FeeError::MathOverflow))?;}else{vault.total_claimed_token2=vault.total_claimed_token2.checked_add(token_amount).ok_or(error!(FeeError::MathOverflow))?;}
  **vault.to_account_info().try_borrow_mut_lamports()?-=sol_amount;
  **ctx.accounts.destination.try_borrow_mut_lamports()?+=sol_amount;
  vault.last_claimed=now;
  let event=SweepBuybackEvent{index,sol_amount,token_amount,destination:ctx.accounts.destination.key(),buyback_vault:vault.key(),mint:ctx.accounts.mint.key()};emit!(event);emit_cpi!(event);Ok(())
 }
#[derive(Accounts)]
#[instruction(index:u8)]
pub struct InitializeBuyback<'info>{
 #[account(mut)]pub payer:Signer<'info>,
 #[account(init,payer=payer,space=208,seeds=[b"buyback-vault".as_ref(),&[index]],bump)]pub buyback_vault:Account<'info,BuybackVault>,
 #[account(init_if_needed,payer=payer,associated_token::mint=mint,associated_token::authority=buyback_vault,associated_token::token_program=token_program)]pub buyback_vault_ata:InterfaceAccount<'info,anchor_spl::token_interface::TokenAccount>,
 pub system_program:Program<'info,System>,pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: event target.
 pub program:UncheckedAccount<'info>,
}
#[derive(Accounts)]
#[instruction(index:u8)]
pub struct UpdateBuyback<'info>{
 #[account(mut)]pub authority:Signer<'info>,
 #[account(mut,seeds=[b"fee-program-global"],bump=fee_program_global.bump,has_one=authority @ FeeError::NotAuthorized)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
 #[account(mut,seeds=[b"buyback-vault".as_ref(),&[index]],bump)]pub buyback_vault:Box<Account<'info,BuybackVault>>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: event target.
 pub program:UncheckedAccount<'info>,
}
#[derive(Accounts)]
#[instruction(index:u8)]
pub struct SweepBuyback<'info>{
 /// CHECK: arbitrary destination.
 #[account(mut)]pub destination:UncheckedAccount<'info>,
 #[account(mut)]pub authority:Signer<'info>,
 #[account(mut,seeds=[b"buyback-vault".as_ref(),&[index]],bump,has_one=authority @ FeeError::NotAuthorized)]pub buyback_vault:Account<'info,BuybackVault>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=buyback_vault,associated_token::token_program=token_program)]pub buyback_vault_ata:InterfaceAccount<'info,anchor_spl::token_interface::TokenAccount>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=destination,associated_token::token_program=token_program)]pub destination_ata:InterfaceAccount<'info,anchor_spl::token_interface::TokenAccount>,
 pub system_program:Program<'info,System>,pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: event target.
 pub program:UncheckedAccount<'info>,
}
pub use crate::state::BuybackVault;
#[event]
pub struct SweepBuybackEvent{pub index:u8,pub sol_amount:u64,pub token_amount:u64,pub destination:Pubkey,pub buyback_vault:Pubkey,pub mint:Pubkey}
