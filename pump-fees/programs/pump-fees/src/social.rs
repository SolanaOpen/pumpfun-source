//! Partial social-fee behavior integrated from the independently validated draft.
use anchor_lang::prelude::*;
use crate::FeeError;
use crate::types::FeeProgramGlobal;
 pub fn claim_v2(ctx:Context<ClaimSocialFeePdaV2>,_user_id:String,_platform:u8)->Result<Option<SocialFeePdaClaimed>> {
  require!(ctx.accounts.fee_program_global.disable_flags&2==0,FeeError::FeatureDeactivated);
  let timestamp=Clock::get()?.unix_timestamp;let now=timestamp as u64;
  let state=&mut ctx.accounts.social_fee_pda;
  if state.last_claimed!=0 && now.saturating_sub(state.last_claimed)<ctx.accounts.fee_program_global.claim_rate_limit{return Ok(None);}
  let native=ctx.accounts.quote_mint.key()==anchor_spl::token::spl_token::native_mint::ID;
  let amount;let before;let after;
  if native {
   amount=state.to_account_info().lamports().saturating_sub(Rent::get()?.minimum_balance(179));
   if amount==0 || ctx.accounts.recipient.executable{return Ok(None);}
   before=ctx.accounts.recipient.lamports();
   {let info=state.to_account_info();let mut from=info.try_borrow_mut_lamports()?;let mut to=ctx.accounts.recipient.try_borrow_mut_lamports()?;**from-=amount;**to+=amount;}
   after=ctx.accounts.recipient.lamports();state.total_claimed=state.total_claimed.saturating_add(amount);
  }else{
   crate::instructions::fee_sharing::admit_associated_token_account(
    &ctx.accounts.associated_social_fee_pda,&state.to_account_info(),&ctx.accounts.quote_mint.key(),
    &ctx.accounts.quote_token_program.key(),&ctx.accounts.quote_mint.to_account_info(),&ctx.accounts.quote_token_program,
    false,&ctx.accounts.social_claim_authority,&ctx.accounts.system_program)?;
   crate::instructions::fee_sharing::admit_associated_token_account(
    &ctx.accounts.associated_recipient,&ctx.accounts.recipient.to_account_info(),&ctx.accounts.quote_mint.key(),
    &ctx.accounts.quote_token_program.key(),&ctx.accounts.quote_mint.to_account_info(),&ctx.accounts.quote_token_program,
    true,&ctx.accounts.social_claim_authority,&ctx.accounts.system_program)?;
   amount={
    let data=ctx.accounts.associated_social_fee_pda.try_borrow_data()?;
    anchor_spl::token_interface::TokenAccount::try_deserialize(&mut &data[..])?.amount
   };
   if amount==0{return Ok(None);}
   before={
    let data=ctx.accounts.associated_recipient.try_borrow_data()?;
    anchor_spl::token_interface::TokenAccount::try_deserialize(&mut &data[..]).map(|account|account.amount).unwrap_or_default()
   };
   let platform=[state.platform];let bump=[state.bump];let seeds:&[&[u8]]=&[b"social-fee-pda",state.user_id.as_bytes(),&platform,&bump];
   anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(ctx.accounts.quote_token_program.to_account_info(),anchor_spl::token_interface::TransferChecked{from:ctx.accounts.associated_social_fee_pda.to_account_info(),mint:ctx.accounts.quote_mint.to_account_info(),to:ctx.accounts.associated_recipient.to_account_info(),authority:state.to_account_info()},&[seeds]),amount,ctx.accounts.quote_mint.decimals)?;
   after=before.saturating_add(amount);
   state.total_stable_claimed=state.total_stable_claimed.saturating_add(amount);
  }
  state.last_claimed=now;
  let event=SocialFeePdaClaimed{timestamp,user_id:state.user_id.clone(),platform:state.platform,social_fee_pda:state.key(),recipient:ctx.accounts.recipient.key(),social_claim_authority:ctx.accounts.social_claim_authority.key(),amount_claimed:amount,claimable_before:amount,lifetime_claimed:state.total_claimed,recipient_balance_before:before,recipient_balance_after:after,quote_mint:if native{Pubkey::default()}else{ctx.accounts.quote_mint.key()},lifetime_stable_claimed:state.total_stable_claimed};
  emit!(event);emit_cpi!(event.clone());Ok(Some(event))
 }

 pub fn create_social(ctx:Context<CreateSocialFeePda>,user_id:String,platform:u8)->Result<()> {
  require!(ctx.accounts.fee_program_global.disable_flags & 1 == 0,FeeError::FeatureDeactivated);
  require!(user_id.len()<=20,FeeError::UserIdTooLong);
  if ctx.accounts.social_fee_pda.version!=0{return Ok(());}
  let state=&mut ctx.accounts.social_fee_pda;
  state.bump=ctx.bumps.social_fee_pda;state.version=1;state.user_id=user_id.clone();state.platform=platform;
  state.total_claimed=0;state.last_claimed=0;state.total_stable_claimed=0;state._reserved=[0;120];
  let event=SocialFeePdaCreated{timestamp:Clock::get()?.unix_timestamp,user_id,platform,social_fee_pda:state.key(),created_by:ctx.accounts.payer.key()};
  emit!(event);emit_cpi!(event);Ok(())
 } pub fn claim(ctx:Context<ClaimSocialFeePda>,user_id:String,platform:u8)->Result<Option<SocialFeePdaClaimed>> {
  require!(ctx.accounts.fee_program_global.disable_flags&2==0,FeeError::FeatureDeactivated);
  let state=&mut ctx.accounts.social_fee_pda;
  let timestamp=Clock::get()?.unix_timestamp;let now=timestamp as u64;
  if state.last_claimed!=0 && now.saturating_sub(state.last_claimed)<ctx.accounts.fee_program_global.claim_rate_limit {return Ok(None);}
  let amount=state.to_account_info().lamports().saturating_sub(Rent::get()?.minimum_balance(179));
  if amount==0 || ctx.accounts.recipient.executable{return Ok(None);}
  let before=ctx.accounts.recipient.lamports();
  {let info=state.to_account_info();let mut from=info.try_borrow_mut_lamports()?;let mut to=ctx.accounts.recipient.try_borrow_mut_lamports()?;**from-=amount;**to+=amount;}
  state.total_claimed=state.total_claimed.saturating_add(amount);state.last_claimed=now;
  let event=SocialFeePdaClaimed {timestamp,user_id:state.user_id.clone(),platform:state.platform,social_fee_pda:state.key(),recipient:ctx.accounts.recipient.key(),social_claim_authority:ctx.accounts.social_claim_authority.key(),amount_claimed:amount,claimable_before:amount,lifetime_claimed:state.total_claimed,recipient_balance_before:before,recipient_balance_after:ctx.accounts.recipient.lamports(),quote_mint:Pubkey::default(),lifetime_stable_claimed:state.total_stable_claimed};
  emit!(event);emit_cpi!(event.clone());Ok(Some(event))
 }
#[derive(Accounts)]
#[instruction(user_id:String,platform:u8)]
pub struct CreateSocialFeePda<'info>{
 #[account(mut)]pub payer:Signer<'info>,
 #[account(init_if_needed,payer=payer,space=179,seeds=[b"social-fee-pda",user_id.as_bytes(),&[platform]],bump)]pub social_fee_pda:Account<'info,SocialFeePda>,
 pub system_program:Program<'info,System>,
 #[account(seeds=[b"fee-program-global"],bump=fee_program_global.bump)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
 /// CHECK: event authority PDA validated.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: event-CPI program account; original imposes no typed-program constraint.
 pub program:UncheckedAccount<'info>,
}
pub use crate::state::SocialFeePda;
#[event]
pub struct SocialFeePdaCreated{pub timestamp:i64,pub user_id:String,pub platform:u8,pub social_fee_pda:Pubkey,pub created_by:Pubkey}

#[derive(Accounts)]
#[instruction(user_id:String,platform:u8)]
pub struct ClaimSocialFeePda<'info>{
 /// CHECK: arbitrary recipient validated in handler.
 #[account(mut)]pub recipient:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"social-fee-pda",user_id.as_bytes(),&[platform]],bump=social_fee_pda.bump)]pub social_fee_pda:Account<'info,SocialFeePda>,
 #[account(seeds=[b"fee-program-global"],bump=fee_program_global.bump,has_one=social_claim_authority @ FeeError::NotAuthorized)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
 pub social_claim_authority:Signer<'info>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: event CPI target.
 pub program:UncheckedAccount<'info>,
}
#[event]
#[derive(Clone)]
pub struct SocialFeePdaClaimed{pub timestamp:i64,pub user_id:String,pub platform:u8,pub social_fee_pda:Pubkey,pub recipient:Pubkey,pub social_claim_authority:Pubkey,pub amount_claimed:u64,pub claimable_before:u64,pub lifetime_claimed:u64,pub recipient_balance_before:u64,pub recipient_balance_after:u64,pub quote_mint:Pubkey,pub lifetime_stable_claimed:u64}

#[derive(Accounts)]
#[instruction(user_id:String,platform:u8)]
pub struct ClaimSocialFeePdaV2<'info>{
 /// CHECK: recipient.
 #[account(mut)]pub recipient:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"social-fee-pda",user_id.as_bytes(),&[platform]],bump=social_fee_pda.bump)]pub social_fee_pda:Account<'info,SocialFeePda>,
 #[account(mut)]pub quote_mint:Box<InterfaceAccount<'info,anchor_spl::token_interface::Mint>>,
 /// CHECK: canonical ATA; data validated on token branch.
 #[account(mut,seeds=[social_fee_pda.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_social_fee_pda:UncheckedAccount<'info>,
 /// CHECK: canonical ATA; data validated on token branch.
 #[account(mut,seeds=[recipient.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_recipient:UncheckedAccount<'info>,
 pub quote_token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 #[account(seeds=[b"fee-program-global"],bump=fee_program_global.bump,has_one=social_claim_authority @ FeeError::NotAuthorized)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
 pub social_claim_authority:Signer<'info>,
 pub system_program:Program<'info,System>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: event CPI target.
 pub program:AccountInfo<'info>,
}
