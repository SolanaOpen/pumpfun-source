use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self,Mint,TokenAccount,TokenInterface,TransferChecked};
use crate::{Global,PumpError};
use crate::trade_types::{TradeEvent,Fees};
use crate::legacy_sell::{LegacyCurve,LegacyFeeConfig,LegacyFeesProgram};
const FEE_PROGRAM:Pubkey=anchor_lang::solana_program::pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
const WSOL:Pubkey=anchor_lang::solana_program::pubkey!("So11111111111111111111111111111111111111112");
#[derive(Accounts)]
pub struct SellV2<'info>{
 #[account(seeds=[b"global"],bump)]pub global:Box<Account<'info,Global>>,
 pub base_mint:Box<InterfaceAccount<'info,Mint>>,
 pub quote_mint:Box<InterfaceAccount<'info,Mint>>,
 pub base_token_program:Interface<'info,TokenInterface>,
 pub quote_token_program:Interface<'info,TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 /// CHECK: handler checks membership.
 #[account(mut)]pub fee_recipient:UncheckedAccount<'info>,
 /// CHECK: quote ATA.
 #[account(mut,seeds=[fee_recipient.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_quote_fee_recipient:UncheckedAccount<'info>,
 /// CHECK: handler checks membership.
 #[account(mut)]pub buyback_fee_recipient:UncheckedAccount<'info>,
 /// CHECK: quote ATA.
 #[account(mut,seeds=[buyback_fee_recipient.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_quote_buyback_fee_recipient:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"bonding-curve",base_mint.key().as_ref()],bump,constraint=quote_mint.key()==if bonding_curve.quote_mint==Pubkey::default(){WSOL}else{bonding_curve.quote_mint} @ PumpError::UnsupportedQuoteMint)]pub bonding_curve:Box<Account<'info,LegacyCurve>>,
 #[account(mut,associated_token::mint=base_mint,associated_token::authority=bonding_curve,associated_token::token_program=base_token_program)]pub associated_base_bonding_curve:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: quote ATA.
 #[account(mut,seeds=[bonding_curve.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_quote_bonding_curve:UncheckedAccount<'info>,
 #[account(mut)]pub user:Signer<'info>,
 #[account(mut,token::mint=base_mint,token::authority=user,token::token_program=base_token_program)]pub associated_base_user:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: quote ATA.
 #[account(mut)]pub associated_quote_user:UncheckedAccount<'info>,
 /// CHECK: PDA.
 #[account(mut,seeds=[b"creator-vault",bonding_curve.creator.as_ref()],bump)]pub creator_vault:UncheckedAccount<'info>,
 /// CHECK: quote ATA.
 #[account(mut,seeds=[creator_vault.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_creator_vault:UncheckedAccount<'info>,
 /// CHECK: sharing validation later.
 #[account(seeds=[b"sharing-config",base_mint.key().as_ref()],bump,seeds::program=FEE_PROGRAM)]pub sharing_config:UncheckedAccount<'info>,
 /// CHECK: volume state validated when used.
 #[account(init_if_needed,payer=user,space=137,seeds=[b"user_volume_accumulator",user.key().as_ref()],bump)]pub user_volume_accumulator:Box<Account<'info,crate::UserVolumeAccumulator>>,
 /// CHECK: quote ATA.
 #[account(mut,seeds=[user_volume_accumulator.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_user_volume_accumulator:UncheckedAccount<'info>,
 #[account(seeds=[b"fee_config",crate::ID.as_ref()],bump,seeds::program=FEE_PROGRAM)]pub fee_config:Account<'info,LegacyFeeConfig>,
 pub fee_program:Program<'info,LegacyFeesProgram>,
 pub system_program:Program<'info,System>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 pub program:Program<'info,crate::program::PumpRecovered>,
}
fn add(a:u64,b:u64)->Result<u64>{a.checked_add(b).ok_or_else(||error!(PumpError::Overflow))}
fn sub(a:u64,b:u64)->Result<u64>{a.checked_sub(b).ok_or_else(||error!(PumpError::Overflow))}
fn narrow(a:u128)->Result<u64>{u64::try_from(a).map_err(|_|error!(PumpError::Truncation))}
pub fn execute(ctx:Context<SellV2>,amount:u64,minimum:u64)->Result<()>{
 let a=&mut *ctx.accounts;
 if a.user_volume_accumulator.user==Pubkey::default(){a.user_volume_accumulator.user=a.user.key();}
 if a.bonding_curve.quote_mint!=Pubkey::default(){
  require!(a.associated_quote_bonding_curve.owner!=&anchor_lang::system_program::ID,anchor_lang::error::ErrorCode::AccountNotInitialized);
  check_token(&a.associated_quote_bonding_curve,a.quote_mint.key(),a.bonding_curve.key())?;
  require!(a.associated_quote_user.owner!=&anchor_lang::system_program::ID,PumpError::AccountTypeNotSupported);
  check_token(&a.associated_quote_user,a.quote_mint.key(),a.user.key())?;{let d=a.associated_quote_user.try_borrow_data()?;let t=TokenAccount::try_deserialize(&mut &d[..])?;require!(t.owner==a.user.key(),PumpError::AccountTypeNotSupported);}
  for (destination,authority) in [(a.associated_quote_fee_recipient.to_account_info(),a.fee_recipient.to_account_info()),(a.associated_quote_buyback_fee_recipient.to_account_info(),a.buyback_fee_recipient.to_account_info()),(a.associated_creator_vault.to_account_info(),a.creator_vault.to_account_info()),(a.associated_user_volume_accumulator.to_account_info(),a.user_volume_accumulator.to_account_info())]{
   if destination.owner==&anchor_lang::system_program::ID{anchor_spl::associated_token::create_idempotent(CpiContext::new(a.associated_token_program.to_account_info(),anchor_spl::associated_token::Create{payer:a.user.to_account_info(),associated_token:destination.clone(),authority:authority.clone(),mint:a.quote_mint.to_account_info(),system_program:a.system_program.to_account_info(),token_program:a.quote_token_program.to_account_info()}))?;}
   check_token(&destination,a.quote_mint.key(),authority.key())?;
  }
 }

 {let info=a.bonding_curve.to_account_info();let reserve=if a.bonding_curve.quote_mint==Pubkey::default(){add(add(a.bonding_curve.real_quote_reserves,a.bonding_curve.creator_fee)?,a.bonding_curve.protocol_fees)?}else{0};let target=add(Rent::get()?.minimum_balance(info.data_len().max(166)),reserve)?;let deficit=target.saturating_sub(info.lamports());if deficit>0{anchor_lang::system_program::transfer(CpiContext::new(a.system_program.to_account_info(),anchor_lang::system_program::Transfer{from:a.user.to_account_info(),to:info.clone()}),deficit)?;}if info.data_len()<166{info.realloc(166,false)?;a.bonding_curve.1=166;}}

let c=&a.bonding_curve;let native=c.quote_mint==Pubkey::default();
 require!(if c.is_mayhem_mode{a.fee_recipient.key()==a.global.reserved_fee_recipient||a.global.reserved_fee_recipients.contains(&a.fee_recipient.key())}else{a.fee_recipient.key()==a.global.fee_recipient||a.global.fee_recipients.contains(&a.fee_recipient.key())},PumpError::NotAuthorized);
 require!(!c.complete,PumpError::BondingCurveComplete);require!(amount>0,PumpError::SellZeroAmount);require!(a.associated_base_user.amount>=amount,PumpError::NotEnoughTokensToSell);
 let quote=narrow(amount as u128*c.virtual_quote_reserves as u128/(c.virtual_token_reserves as u128+amount as u128))?;
 let cap=c.token_total_supply as u128*c.virtual_quote_reserves as u128/c.virtual_token_reserves as u128;
 let mut data=vec![154,237,138,92,162,2,162,187,1];data.extend_from_slice(&cap.to_le_bytes());data.extend_from_slice(c.quote_mint.as_ref());
 let ix=anchor_lang::solana_program::instruction::Instruction{program_id:FEE_PROGRAM,accounts:vec![anchor_lang::solana_program::instruction::AccountMeta::new_readonly(a.fee_config.key(),false),anchor_lang::solana_program::instruction::AccountMeta::new_readonly(crate::ID,false)],data};
 anchor_lang::solana_program::program::invoke(&ix,&[a.fee_config.to_account_info(),a.program.to_account_info(),a.fee_program.to_account_info()])?;
 let (_,ret)=anchor_lang::solana_program::program::get_return_data().unwrap();let mut fees=Fees::deserialize(&mut ret.as_slice())?;if c.creator_fee_bps>0{fees.creator_fee_bps=c.creator_fee_bps;}
 let cb=if c.creator==Pubkey::default(){0}else{fees.creator_fee_bps};let pf=narrow((quote as u128*fees.protocol_fee_bps as u128+9999)/10000)?;let cf=narrow((quote as u128*cb as u128+9999)/10000)?;let output=sub(sub(quote,pf)?,cf)?;require!(output>=minimum,PumpError::TooLittleSolReceived);
 let vb=add(c.virtual_token_reserves,amount)?;let vq=sub(c.virtual_quote_reserves,quote)?;let rb=add(c.real_token_reserves,amount)?;let rq=sub(c.real_quote_reserves,quote)?;
 token_interface::transfer_checked(CpiContext::new(a.base_token_program.to_account_info(),TransferChecked{from:a.associated_base_user.to_account_info(),mint:a.base_mint.to_account_info(),to:a.associated_base_bonding_curve.to_account_info(),authority:a.user.to_account_info()}),amount,a.base_mint.decimals)?;
 let rem=ctx.remaining_accounts;
 let bb=&a.buyback_fee_recipient;require!(a.global.buyback_fee_recipients.contains(bb.key),PumpError::BuybackFeeRecipientNotAuthorized);
 let bbps=if c.is_mayhem_mode{0}else{a.global.buyback_basis_points};let bf=narrow(pf as u128*bbps as u128/10000)?;
 let shareholders={let d=a.sharing_config.try_borrow_data()?;crate::SharingConfig::try_deserialize(&mut &d[..]).map(|s|s.shareholders).unwrap_or_default()};
 let ev=TradeEvent{mint:a.base_mint.key(),sol_amount:if native{quote}else{0},token_amount:amount,is_buy:false,user:a.user.key(),timestamp:Clock::get()?.unix_timestamp,virtual_sol_reserves:if native{vq}else{0},virtual_token_reserves:vb,real_sol_reserves:if native{rq}else{0},real_token_reserves:rb,fee_recipient:a.fee_recipient.key(),fee_basis_points:fees.protocol_fee_bps,fee:pf,creator:c.creator,creator_fee_basis_points:if c.is_cashback_coin{0}else{fees.creator_fee_bps},creator_fee:if c.is_cashback_coin{0}else{cf},track_volume:false,total_unclaimed_tokens:0,total_claimed_tokens:0,current_sol_volume:0,last_update_timestamp:0,ix_name:"sell".into(),mayhem_mode:c.is_mayhem_mode,cashback_fee_basis_points:if c.is_cashback_coin{cb}else{0},cashback:if c.is_cashback_coin{cf}else{0},buyback_fee_basis_points:bbps,buyback_fee:bf,shareholders,quote_mint:c.quote_mint,quote_amount:quote,virtual_quote_reserves:vq,real_quote_reserves:rq,holder_rewards_bps:if c.is_holder_reward{cb}else{0},holder_rewards:if c.is_holder_reward{cf}else{0},creator_fee_unclaimed:c.creator_fee,unparsed_trailing_field:0};
 let cashback_coin=c.is_cashback_coin;


 if native{
 a.bonding_curve.sub_lamports(quote)?;a.user.add_lamports(output)?;a.fee_recipient.add_lamports(sub(pf,bf)?)?;if cashback_coin{a.user_volume_accumulator.add_lamports(cf)?;a.user_volume_accumulator.cashback_earned=add(a.user_volume_accumulator.cashback_earned,cf)?;}else{a.creator_vault.add_lamports(cf)?;}bb.add_lamports(bf)?;
 }else{
 let mint=a.base_mint.key();let bump=[ctx.bumps.bonding_curve];let seeds:&[&[u8]]=&[b"bonding-curve",mint.as_ref(),&bump];
 for (to,n) in [(if cashback_coin{a.associated_user_volume_accumulator.to_account_info()}else{a.associated_creator_vault.to_account_info()},cf),(a.associated_quote_user.to_account_info(),output),(a.associated_quote_buyback_fee_recipient.to_account_info(),bf),(a.associated_quote_fee_recipient.to_account_info(),sub(pf,bf)?)]{if n>0{token_interface::transfer_checked(CpiContext::new_with_signer(a.quote_token_program.to_account_info(),TransferChecked{from:a.associated_quote_bonding_curve.to_account_info(),mint:a.quote_mint.to_account_info(),to,authority:a.bonding_curve.to_account_info()},&[seeds]),n,a.quote_mint.decimals)?;}}
 if cashback_coin{a.user_volume_accumulator.stable_cashback_earned=add(a.user_volume_accumulator.stable_cashback_earned,cf)?;}
 }

 let c=&mut a.bonding_curve;c.virtual_token_reserves=vb;c.virtual_quote_reserves=vq;c.real_token_reserves=rb;c.real_quote_reserves=rq;emit!(ev.clone());emit_cpi!(ev);Ok(())
}

fn check_token(info:&AccountInfo,mint:Pubkey,owner:Pubkey)->Result<()>{let d=info.try_borrow_data()?;let t=TokenAccount::try_deserialize(&mut &d[..])?;require!(t.mint==mint,PumpError::MintDoesNotMatchBondingCurve);let _=owner;Ok(())}
