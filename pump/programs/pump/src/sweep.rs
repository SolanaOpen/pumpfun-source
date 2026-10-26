use anchor_lang::prelude::*;
use crate::{Global,PumpError};
#[derive(Clone)]pub struct SweepCurve{pub data:Vec<u8>}
impl Owner for SweepCurve{fn owner()->Pubkey{crate::ID}}
impl AccountSerialize for SweepCurve{fn try_serialize<W:std::io::Write>(&self,w:&mut W)->Result<()>{w.write_all(&self.data)?;Ok(())}}
impl AccountDeserialize for SweepCurve{
 fn try_deserialize(buf:&mut &[u8])->Result<Self>{Self::try_deserialize_unchecked(buf)}
 fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{
  if buf.len()<49 || buf[..8]!=[23,183,248,55,96,216,172,96]{return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());}
  for off in [48usize,81,82,123,124]{if buf.len()>off{let _=bool::deserialize(&mut &buf[off..off+1])?;}}
  let len=buf.len();let mut data=buf.to_vec();data.resize(len.max(166),0);
  for (a,b) in [(49,81),(83,115),(115,125),(125,141),(142,150),(150,158),(158,166)]{if len<b{data[a..b].fill(0);}}
  Ok(Self{data})
 }
}
impl SweepCurve{
 fn field_key(&self,n:usize)->Pubkey{Pubkey::new_from_array(self.data[n..n+32].try_into().unwrap())}
 fn number(&self,n:usize)->u64{u64::from_le_bytes(self.data[n..n+8].try_into().unwrap())}
}
#[event_cpi]
#[derive(Accounts)]
pub struct SweepFees<'info>{
 #[account(mut)]pub payer:Signer<'info>,
 #[account(seeds=[b"global"],bump)]pub global:Account<'info,Global>,
 pub base_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 pub quote_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 pub quote_token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 pub system_program:Program<'info,System>,
 #[account(mut,seeds=[b"bonding-curve",base_mint.key().as_ref()],bump,constraint=(if bonding_curve.field_key(83)==Pubkey::default(){anchor_spl::token::spl_token::native_mint::ID}else{bonding_curve.field_key(83)})==quote_mint.key() @ PumpError::MintDoesNotMatchBondingCurve)]pub bonding_curve:Account<'info,SweepCurve>,
 /// CHECK: checked as token data only in the positive token branch.
 #[account(mut,seeds=[bonding_curve.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_quote_bonding_curve:UncheckedAccount<'info>,
 /// CHECK: handler verifies fee recipient or creator vault.
 #[account(mut)]pub recipient:UncheckedAccount<'info>,
 /// CHECK: address checked; token branch creates or parses.
 #[account(mut,seeds=[recipient.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub associated_quote_recipient:UncheckedAccount<'info>,
}
#[event]
#[derive(Clone)]pub struct SweepBondingCurveFeeEvent{pub timestamp:i64,pub mint:Pubkey,pub bonding_curve:Pubkey,pub quote_mint:Pubkey,pub recipient:Pubkey,pub amount:u64,pub bucket:u8}
pub fn execute(ctx:Context<SweepFees>,creator:bool)->Result<()>{
 let a=&mut *ctx.accounts;let native=a.bonding_curve.field_key(83)==Pubkey::default();let cf=a.bonding_curve.number(125);let pf=a.bonding_curve.number(133);let info=a.bonding_curve.to_account_info();
 let reserve=if native{cf.checked_add(pf).ok_or(error!(PumpError::Overflow))?}else{0};let available=info.lamports().saturating_sub(reserve);let rent=Rent::get()?.minimum_balance(info.data_len().max(166));let deficit=rent.saturating_sub(available);
 if deficit>0{anchor_lang::system_program::transfer(CpiContext::new(a.system_program.to_account_info(),anchor_lang::system_program::Transfer{from:a.payer.to_account_info(),to:info.clone()}),deficit)?;}
 if info.data_len()<166{info.realloc(166,false)?;}
 if creator{let expected=Pubkey::find_program_address(&[b"creator-vault",a.bonding_curve.field_key(49).as_ref()],&crate::ID).0;require_keys_eq!(a.recipient.key(),expected,anchor_lang::error::ErrorCode::ConstraintSeeds);}
 else{let k=a.recipient.key();let valid=if a.bonding_curve.data[81]!=0{a.global.reserved_fee_recipient==k||a.global.reserved_fee_recipients.contains(&k)}else{a.global.fee_recipient==k||classic_search(&a.global.fee_recipients,&k)};require!(valid,PumpError::NotAuthorized);}
 let amount=if creator{cf}else{pf};if amount==0{return Ok(());}
 if native{a.bonding_curve.sub_lamports(amount)?;a.recipient.add_lamports(amount)?;}
 else{

  for (index,account) in [a.associated_quote_bonding_curve.to_account_info(),a.associated_quote_recipient.to_account_info()].into_iter().enumerate(){
  if index==1 && a.associated_quote_recipient.owner==&anchor_lang::system_program::ID{anchor_spl::associated_token::create_idempotent(CpiContext::new(a.associated_token_program.to_account_info(),anchor_spl::associated_token::Create{payer:a.payer.to_account_info(),associated_token:a.associated_quote_recipient.to_account_info(),authority:a.recipient.to_account_info(),mint:a.quote_mint.to_account_info(),system_program:a.system_program.to_account_info(),token_program:a.quote_token_program.to_account_info()}))?;}
   if account.owner==&anchor_lang::system_program::ID{return Err(error!(anchor_lang::error::ErrorCode::AccountNotInitialized));}
   if account.owner!=&anchor_spl::token::ID&&account.owner!=&anchor_spl::token_2022::ID{return Err(error!(anchor_lang::error::ErrorCode::AccountOwnedByWrongProgram));}
   let d=account.try_borrow_data()?;let token=anchor_spl::token_interface::TokenAccount::try_deserialize(&mut &d[..])?;require_keys_eq!(token.mint,a.quote_mint.key(),PumpError::MintDoesNotMatchBondingCurve);
  }
  let mint=a.base_mint.key();let bump=[ctx.bumps.bonding_curve];let seeds:&[&[u8]]=&[b"bonding-curve",mint.as_ref(),&bump];
  anchor_spl::token_interface::transfer_checked(CpiContext::new_with_signer(a.quote_token_program.to_account_info(),anchor_spl::token_interface::TransferChecked{from:a.associated_quote_bonding_curve.to_account_info(),mint:a.quote_mint.to_account_info(),to:a.associated_quote_recipient.to_account_info(),authority:info},&[seeds]),amount,a.quote_mint.decimals)?;
 }
 let offset=if creator{125}else{133};a.bonding_curve.data[offset..offset+8].fill(0);
 let event=SweepBondingCurveFeeEvent{timestamp:Clock::get()?.unix_timestamp,mint:a.base_mint.key(),bonding_curve:a.bonding_curve.key(),quote_mint:a.bonding_curve.field_key(83),recipient:a.recipient.key(),amount,bucket:if creator{1}else{0}};emit!(event.clone());emit_cpi!(event);Ok(())
}

fn classic_search(values:&[Pubkey],key:&Pubkey)->bool{let(mut lo,mut hi)=(0,values.len());while lo<hi{let mid=lo+(hi-lo)/2;match values[mid].cmp(key){std::cmp::Ordering::Equal=>return true,std::cmp::Ordering::Less=>lo=mid+1,std::cmp::Ordering::Greater=>hi=mid}}false}
