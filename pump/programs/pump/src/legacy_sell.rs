use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self,Mint,TokenAccount,TokenInterface,TransferChecked};
use crate::{Global,PumpError};
use crate::trade_types::{BondingCurve,TradeEvent,Fees};
const FEE_PROGRAM:Pubkey=anchor_lang::solana_program::pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");

#[derive(Clone)]pub struct LegacyCurve(pub BondingCurve,pub usize);
impl std::ops::Deref for LegacyCurve{type Target=BondingCurve;fn deref(&self)->&BondingCurve{&self.0}}
impl std::ops::DerefMut for LegacyCurve{fn deref_mut(&mut self)->&mut BondingCurve{&mut self.0}}
impl Owner for LegacyCurve{fn owner()->Pubkey{crate::ID}}
impl AccountDeserialize for LegacyCurve{fn try_deserialize(buf:&mut &[u8])->Result<Self>{crate::DistributionCurve::try_deserialize(buf)?;let len=buf.len();let mut bytes=buf.to_vec();bytes.resize(166,0);if len<141{bytes[125..141].fill(0);}if len<166{bytes[141..166].fill(0);}Ok(Self(BondingCurve::deserialize(&mut &bytes[8..])?,len))}fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{Self::try_deserialize(buf)}}
impl AccountSerialize for LegacyCurve{fn try_serialize<W:std::io::Write>(&self,w:&mut W)->Result<()>{let mut b=Vec::new();self.0.try_serialize(&mut b)?;w.write_all(&b[..self.1.min(b.len())])?;Ok(())}}
#[derive(Clone)]pub struct LegacyFeeConfig;
impl Owner for LegacyFeeConfig{fn owner()->Pubkey{FEE_PROGRAM}}
impl AccountSerialize for LegacyFeeConfig{}
impl AccountDeserialize for LegacyFeeConfig{fn try_deserialize(buf:&mut &[u8])->Result<Self>{if buf.len()<8{return err!(anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound)}if buf[..8]!=[143,52,146,187,219,123,76,155]{return err!(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch)}Self::try_deserialize_unchecked(buf)}fn try_deserialize_unchecked(buf:&mut &[u8])->Result<Self>{crate::trade_types::FeeConfig::deserialize(&mut &buf[8..]).map_err(|_|error!(anchor_lang::error::ErrorCode::AccountDidNotDeserialize))?;Ok(Self)}}
#[derive(Clone)]pub struct LegacyFeesProgram;
impl Id for LegacyFeesProgram{fn id()->Pubkey{FEE_PROGRAM}}

#[derive(Accounts)]
pub struct LegacySell<'info>{
 #[account(seeds=[b"global"],bump)]pub global:Box<Account<'info,Global>>,
 /// CHECK: handler validates recipient.
 #[account(mut)]pub fee_recipient:UncheckedAccount<'info>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,seeds=[b"bonding-curve",mint.key().as_ref()],bump,constraint=bonding_curve.quote_mint==Pubkey::default() @ PumpError::UnsupportedQuoteMint)]pub bonding_curve:Box<Account<'info,LegacyCurve>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=bonding_curve,associated_token::token_program=token_program)]pub associated_bonding_curve:InterfaceAccount<'info,TokenAccount>,
 #[account(mut,token::mint=mint,token::authority=user,token::token_program=token_program)]pub associated_user:InterfaceAccount<'info,TokenAccount>,
 #[account(mut)]pub user:Signer<'info>,
 pub system_program:Program<'info,System>,
 /// CHECK: PDA checked.
 #[account(mut,seeds=[b"creator-vault",bonding_curve.creator.as_ref()],bump)]pub creator_vault:UncheckedAccount<'info>,
 pub token_program:Interface<'info,TokenInterface>,
 /// CHECK: event authority.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: self CPI.
 pub program:Program<'info,crate::program::PumpRecovered>,
 /// CHECK: fee program validates config.
 #[account(seeds=[b"fee_config",crate::ID.as_ref()],bump,seeds::program=FEE_PROGRAM)]
 pub fee_config:Account<'info,LegacyFeeConfig>,
 /// CHECK: canonical dependency.
 pub fee_program:Program<'info,LegacyFeesProgram>,
}
fn add(a:u64,b:u64)->Result<u64>{a.checked_add(b).ok_or_else(||error!(PumpError::Overflow))}
fn sub(a:u64,b:u64)->Result<u64>{a.checked_sub(b).ok_or_else(||error!(PumpError::Overflow))}
fn narrow(a:u128)->Result<u64>{u64::try_from(a).map_err(|_|error!(PumpError::Truncation))}
pub fn execute(ctx:Context<LegacySell>,amount:u64,minimum:u64)->Result<()>{
 let a=&mut *ctx.accounts;
 {let info=a.bonding_curve.to_account_info();let reserve=add(add(a.bonding_curve.real_quote_reserves,a.bonding_curve.creator_fee)?,a.bonding_curve.protocol_fees)?;let target=add(Rent::get()?.minimum_balance(info.data_len().max(166)),reserve)?;let deficit=target.saturating_sub(info.lamports());if deficit>0{anchor_lang::system_program::transfer(CpiContext::new(a.system_program.to_account_info(),anchor_lang::system_program::Transfer{from:a.user.to_account_info(),to:info.clone()}),deficit)?;}if info.data_len()<166{info.realloc(166,false)?;a.bonding_curve.1=166;}}

let c=&a.bonding_curve;
 require!(if c.is_mayhem_mode{a.fee_recipient.key()==a.global.reserved_fee_recipient||a.global.reserved_fee_recipients.contains(&a.fee_recipient.key())}else{a.fee_recipient.key()==a.global.fee_recipient||a.global.fee_recipients.contains(&a.fee_recipient.key())},PumpError::NotAuthorized);
 require!(!c.complete,PumpError::BondingCurveComplete);require!(amount>0,PumpError::SellZeroAmount);require!(a.associated_user.amount>=amount,PumpError::NotEnoughTokensToSell);
 let quote=narrow(amount as u128*c.virtual_quote_reserves as u128/(c.virtual_token_reserves as u128+amount as u128))?;
 let cap=c.token_total_supply as u128*c.virtual_quote_reserves as u128/c.virtual_token_reserves as u128;
 let mut data=vec![154,237,138,92,162,2,162,187,1];data.extend_from_slice(&cap.to_le_bytes());data.extend_from_slice(Pubkey::default().as_ref());
 let ix=anchor_lang::solana_program::instruction::Instruction{program_id:FEE_PROGRAM,accounts:vec![anchor_lang::solana_program::instruction::AccountMeta::new_readonly(a.fee_config.key(),false),anchor_lang::solana_program::instruction::AccountMeta::new_readonly(crate::ID,false)],data};
 anchor_lang::solana_program::program::invoke(&ix,&[a.fee_config.to_account_info(),a.program.to_account_info(),a.fee_program.to_account_info()])?;
 let (_,ret)=anchor_lang::solana_program::program::get_return_data().unwrap();let mut fees=Fees::deserialize(&mut ret.as_slice())?;if c.creator_fee_bps>0{fees.creator_fee_bps=c.creator_fee_bps;}
 let cb=if c.creator==Pubkey::default(){0}else{fees.creator_fee_bps};let pf=narrow((quote as u128*fees.protocol_fee_bps as u128+9999)/10000)?;let cf=narrow((quote as u128*cb as u128+9999)/10000)?;let output=sub(sub(quote,pf)?,cf)?;require!(output>=minimum,PumpError::TooLittleSolReceived);
 let vb=add(c.virtual_token_reserves,amount)?;let vq=sub(c.virtual_quote_reserves,quote)?;let rb=add(c.real_token_reserves,amount)?;let rq=sub(c.real_quote_reserves,quote)?;
 token_interface::transfer_checked(CpiContext::new(a.token_program.to_account_info(),TransferChecked{from:a.associated_user.to_account_info(),mint:a.mint.to_account_info(),to:a.associated_bonding_curve.to_account_info(),authority:a.user.to_account_info()}),amount,a.mint.decimals)?;
 let rem=ctx.remaining_accounts;let index=if c.is_cashback_coin {require!(rem.len()>=2,PumpError::MissingCashbackAccounts);let acc=&rem[0];require!(acc.key()==Pubkey::find_program_address(&[b"user_volume_accumulator",a.user.key().as_ref()],&crate::ID).0&&acc.owner==&crate::ID&&acc.data_len()>=82,PumpError::InvalidCashbackAccumulator);1}else{0};
 let v2=rem.get(index).ok_or_else(||error!(PumpError::InvalidBondingCurveV2))?;require!(v2.key()==Pubkey::find_program_address(&[b"bonding-curve-v2",a.mint.key().as_ref()],&crate::ID).0,PumpError::InvalidBondingCurveV2);
 let bb=rem.get(index+1).ok_or_else(||error!(PumpError::BuybackFeeRecipientMissing))?;require!(a.global.buyback_fee_recipients.contains(bb.key),PumpError::BuybackFeeRecipientNotAuthorized);
 let bbps=if c.is_mayhem_mode{0}else{a.global.buyback_basis_points};let bf=narrow(pf as u128*bbps as u128/10000)?;
 let ev=TradeEvent{mint:a.mint.key(),sol_amount:quote,token_amount:amount,is_buy:false,user:a.user.key(),timestamp:Clock::get()?.unix_timestamp,virtual_sol_reserves:vq,virtual_token_reserves:vb,real_sol_reserves:rq,real_token_reserves:rb,fee_recipient:a.fee_recipient.key(),fee_basis_points:fees.protocol_fee_bps,fee:pf,creator:c.creator,creator_fee_basis_points:if c.is_cashback_coin{0}else{fees.creator_fee_bps},creator_fee:if c.is_cashback_coin{0}else{cf},track_volume:false,total_unclaimed_tokens:0,total_claimed_tokens:0,current_sol_volume:0,last_update_timestamp:0,ix_name:"sell".into(),mayhem_mode:c.is_mayhem_mode,cashback_fee_basis_points:if c.is_cashback_coin{cb}else{0},cashback:if c.is_cashback_coin{cf}else{0},buyback_fee_basis_points:bbps,buyback_fee:bf,shareholders:vec![],quote_mint:Pubkey::default(),quote_amount:quote,virtual_quote_reserves:vq,real_quote_reserves:rq,holder_rewards_bps:if c.is_holder_reward{cb}else{0},holder_rewards:if c.is_holder_reward{cf}else{0},creator_fee_unclaimed:c.creator_fee,unparsed_trailing_field:0};
 let cashback_coin=c.is_cashback_coin;


 a.bonding_curve.sub_lamports(quote)?;a.user.add_lamports(output)?;a.fee_recipient.add_lamports(sub(pf,bf)?)?;if cashback_coin{rem[0].add_lamports(cf)?;let mut d=rem[0].try_borrow_mut_data()?;let n=u64::from_le_bytes(d[74..82].try_into().unwrap());d[74..82].copy_from_slice(&add(n,cf)?.to_le_bytes());}else{a.creator_vault.add_lamports(cf)?;}bb.add_lamports(bf)?;
 let c=&mut a.bonding_curve;c.virtual_token_reserves=vb;c.virtual_quote_reserves=vq;c.real_token_reserves=rb;c.real_quote_reserves=rq;emit!(ev.clone());emit_cpi!(ev);Ok(())
}
