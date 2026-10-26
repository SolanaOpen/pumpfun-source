use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use crate::{PumpError,trade_types::Shareholder};
pub const FEES:Pubkey=anchor_lang::solana_program::pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
#[derive(AnchorSerialize,AnchorDeserialize,Clone,PartialEq)]pub enum ConfigStatus{Paused,Active}
#[derive(AnchorSerialize,AnchorDeserialize,Clone)]pub struct SharingConfig{pub bump:u8,pub version:u8,pub status:ConfigStatus,pub mint:Pubkey,pub admin:Pubkey,pub admin_revoked:bool,pub shareholders:Vec<Shareholder>}
impl Owner for SharingConfig{fn owner()->Pubkey{FEES}}
impl Discriminator for SharingConfig{const DISCRIMINATOR:&'static[u8]=&[216,74,9,0,56,140,93,75];}
impl AccountSerialize for SharingConfig{}
impl AccountDeserialize for SharingConfig{
 fn try_deserialize(buf:&mut &[u8])->Result<Self>{if buf.len()<8{return Err(error!(anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound));}if &buf[..8]!=Self::DISCRIMINATOR{return Err(error!(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch));}Self::try_deserialize_unchecked(buf)}
 fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{Self::deserialize(&mut &buf[8..]).map_err(|_|error!(anchor_lang::error::ErrorCode::AccountDidNotDeserialize))}
}
// Legacy curve layouts omit trailing fields. Keep this read-only view separate from trade state.
#[derive(Clone)]pub struct DistributionCurve{pub creator:Pubkey,pub quote_mint:Pubkey}
impl Owner for DistributionCurve{fn owner()->Pubkey{crate::ID}}
impl AccountSerialize for DistributionCurve{}
impl AccountDeserialize for DistributionCurve{
 fn try_deserialize(buf:&mut &[u8])->Result<Self>{Self::try_deserialize_unchecked(buf)}
 fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{
  if buf.len()<49 || buf[..8]!=[23,183,248,55,96,216,172,96]{return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());}
  for offset in [48usize,81,82,123,124]{if buf.len()>offset{let _=bool::deserialize(&mut &buf[offset..offset+1])?;}}
  let creator=if buf.len()>=81{Pubkey::new_from_array(buf[49..81].try_into().unwrap())}else{Pubkey::default()};
  let quote_mint=if buf.len()>=115{Pubkey::new_from_array(buf[83..115].try_into().unwrap())}else{Pubkey::default()};Ok(Self{creator,quote_mint})
 }
}
#[event]
#[derive(Clone)]pub struct DistributeCreatorFeesEvent{pub timestamp:i64,pub mint:Pubkey,pub bonding_curve:Pubkey,pub sharing_config:Pubkey,pub admin:Pubkey,pub shareholders:Vec<Shareholder>,pub distributed:u64,pub quote_mint:Pubkey}
#[derive(Accounts)]
pub struct DistributeCreatorFees<'info>{
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump)]pub bonding_curve:Account<'info,DistributionCurve>,
 #[account(seeds=[b"sharing-config",mint.key().as_ref()],bump,seeds::program=FEES,has_one=mint @ PumpError::MintDoesNotMatchBondingCurve)]pub sharing_config:Account<'info,SharingConfig>,
 #[account(mut,seeds=[b"creator-vault",sharing_config.key().as_ref()],bump)]pub creator_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::PumpRecovered>,
}
pub fn execute<'info>(ctx:Context<'_, '_, '_, 'info,DistributeCreatorFees<'info>>)->Result<DistributeCreatorFeesEvent>{
 let a=&ctx.accounts;
 require_keys_eq!(a.bonding_curve.creator,a.sharing_config.key(),PumpError::BondingCurveAndSharingConfigCreatorMismatch);
 require!(a.sharing_config.status==ConfigStatus::Active,PumpError::SharingConfigNotActive);
 require!(ctx.remaining_accounts.len()>=a.sharing_config.shareholders.len(),PumpError::NotEnoughRemainingAccounts);
 let available=a.creator_vault.lamports().saturating_sub(Rent::get()?.minimum_balance(0));let mut distributed=0u64;
 let eligible=native_eligible(&a.sharing_config.shareholders,ctx.remaining_accounts,available)?;
 let bump=[ctx.bumps.creator_vault];let seeds:&[&[u8]]=&[b"creator-vault",a.bonding_curve.creator.as_ref(),&bump];
 for (share,recipient) in a.sharing_config.shareholders.iter().zip(ctx.remaining_accounts){
  require_keys_eq!(share.address,recipient.key(),PumpError::ShareholdersAndRemainingAccountsMismatch);
  require!(!recipient.executable,PumpError::UnableToDistributeCreatorFeesToExecutableRecipient);
  require!(share.share_bps>0,PumpError::InvalidShareBps);
  let amount=((available as u128)*(share.share_bps as u128)/10000) as u64;
  if eligible && amount>0{anchor_lang::system_program::transfer(CpiContext::new_with_signer(a.system_program.to_account_info(),anchor_lang::system_program::Transfer{from:a.creator_vault.to_account_info(),to:recipient.clone()},&[seeds]),amount)?;distributed=distributed.checked_add(amount).ok_or(error!(PumpError::Overflow))?;}
 }
 let event=DistributeCreatorFeesEvent{timestamp:Clock::get()?.unix_timestamp,mint:a.mint.key(),bonding_curve:a.bonding_curve.key(),sharing_config:a.sharing_config.key(),admin:a.sharing_config.admin,shareholders:a.sharing_config.shareholders.clone(),distributed,quote_mint:Pubkey::default()};if eligible{emit!(event.clone());emit_cpi!(event.clone());}Ok(event)
}

#[derive(Accounts)]
pub struct DistributeCreatorFeesV2<'info>{
 #[account(mut)]pub payer:Signer<'info>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump)]pub bonding_curve:Account<'info,DistributionCurve>,
 #[account(seeds=[b"sharing-config",mint.key().as_ref()],bump,seeds::program=FEES,has_one=mint @ PumpError::MintDoesNotMatchBondingCurve)]pub sharing_config:Account<'info,SharingConfig>,
 /// CHECK: Native vault address validated by PDA seeds.
 #[account(mut,seeds=[b"creator-vault",sharing_config.key().as_ref()],bump)]pub creator_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 /// CHECK: Event authority.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::PumpRecovered>,
 /// CHECK: Nonlegacy branch parses token data manually.
 #[account(mut,seeds=[creator_vault.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub creator_vault_quote_token_account:UncheckedAccount<'info>,
 pub quote_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 pub quote_token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
}
pub fn execute_v2<'info>(ctx:Context<'_, '_, '_, 'info,DistributeCreatorFeesV2<'info>>,initialize_ata:bool)->Result<DistributeCreatorFeesEvent>{
 let a=&ctx.accounts;
 require_keys_eq!(a.bonding_curve.creator,a.sharing_config.key(),PumpError::BondingCurveAndSharingConfigCreatorMismatch);
 require!(a.sharing_config.status==ConfigStatus::Active,PumpError::SharingConfigNotActive);
 let native=a.bonding_curve.quote_mint==Pubkey::default() || a.bonding_curve.quote_mint==anchor_spl::token::spl_token::native_mint::ID;
 let count=a.sharing_config.shareholders.len();
 require_keys_eq!(a.quote_mint.key(),if native{anchor_spl::token::spl_token::native_mint::ID}else{a.bonding_curve.quote_mint},PumpError::MintDoesNotMatchBondingCurve);
 let available=if native{a.creator_vault.lamports().saturating_sub(Rent::get()?.minimum_balance(0))}else{
  let data=a.creator_vault_quote_token_account.try_borrow_data()?;
  anchor_spl::token_interface::TokenAccount::try_deserialize(&mut data.as_ref())?.amount
 };
 require!(ctx.remaining_accounts.len()>=count*if native{1}else{2},PumpError::NotEnoughRemainingAccounts);
 let eligible=if native{native_eligible(&a.sharing_config.shareholders,ctx.remaining_accounts,available)?}else{true};
 let mut distributed=0u64;let bump=[ctx.bumps.creator_vault];let seeds:&[&[u8]]=&[b"creator-vault",a.bonding_curve.creator.as_ref(),&bump];
 for (i,share) in a.sharing_config.shareholders.iter().enumerate(){
  let recipient=&ctx.remaining_accounts[i];
  require_keys_eq!(share.address,recipient.key(),PumpError::ShareholdersAndRemainingAccountsMismatch);
  require!(!recipient.executable,PumpError::UnableToDistributeCreatorFeesToExecutableRecipient);
  require!(share.share_bps>0,PumpError::InvalidShareBps);
  let amount=((available as u128)*(share.share_bps as u128)/10000) as u64;
  if native{
   if eligible && amount>0{anchor_lang::system_program::transfer(CpiContext::new_with_signer(a.system_program.to_account_info(),anchor_lang::system_program::Transfer{from:a.creator_vault.to_account_info(),to:recipient.clone()},&[seeds]),amount)?;}
  }else{
   let ata=&ctx.remaining_accounts[count+i];
   let expected=anchor_spl::associated_token::get_associated_token_address_with_program_id(&recipient.key(),&a.quote_mint.key(),&a.quote_token_program.key());
   require_keys_eq!(ata.key(),expected,PumpError::ShareholdersAndRemainingAccountsMismatch);
   if ata.data_is_empty(){
    require!(initialize_ata,PumpError::UnableToDistributeCreatorFeesToUninitializedAccount);
    anchor_spl::associated_token::create(CpiContext::new(a.associated_token_program.to_account_info(),anchor_spl::associated_token::Create{payer:a.payer.to_account_info(),associated_token:ata.clone(),authority:recipient.clone(),mint:a.quote_mint.to_account_info(),system_program:a.system_program.to_account_info(),token_program:a.quote_token_program.to_account_info()}))?;
   }
   if amount>0{anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(a.quote_token_program.to_account_info(),anchor_spl::token_interface::TransferChecked{from:a.creator_vault_quote_token_account.to_account_info(),mint:a.quote_mint.to_account_info(),to:ata.clone(),authority:a.creator_vault.to_account_info()},&[seeds]),amount,a.quote_mint.decimals)?;}
  }
  if eligible{distributed=distributed.checked_add(amount).ok_or(error!(PumpError::Overflow))?;}
 }
 let event=DistributeCreatorFeesEvent{timestamp:Clock::get()?.unix_timestamp,mint:a.mint.key(),bonding_curve:a.bonding_curve.key(),sharing_config:a.sharing_config.key(),admin:a.sharing_config.admin,shareholders:a.sharing_config.shareholders.clone(),distributed,quote_mint:if native{Pubkey::default()}else{a.quote_mint.key()}};if eligible{emit!(event.clone());emit_cpi!(event.clone());}Ok(event)
}

fn native_eligible(shares:&[Shareholder],recipients:&[AccountInfo],available:u64)->Result<bool>{
 require!(recipients.len()>=shares.len(),PumpError::NotEnoughRemainingAccounts);
 let rent=Rent::get()?;let mut minimum=0u128;
 for (share,recipient) in shares.iter().zip(recipients){
  require_keys_eq!(share.address,recipient.key(),PumpError::ShareholdersAndRemainingAccountsMismatch);
  require!(!recipient.executable,PumpError::UnableToDistributeCreatorFeesToExecutableRecipient);
  require!(share.share_bps>0,PumpError::InvalidShareBps);
  let deficit=rent.minimum_balance(recipient.data_len()).saturating_sub(recipient.lamports()) as u128;
  minimum=minimum.max((deficit*10000+share.share_bps as u128-1)/share.share_bps as u128);
 }Ok(available as u128>=minimum)
}
