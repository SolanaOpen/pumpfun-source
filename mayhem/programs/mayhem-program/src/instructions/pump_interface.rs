pub mod buy {
//! Legacy buy recovered from original dispatch 0x813f8, parser 0x525f8,
//! handler 0x1bc08 and signed-counter/risk helper 0x2c598.
//! Fixture validation is synthetic and ordinary-signed; no historical prestate claim.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction,program::invoke_signed};
use anchor_spl::token_interface::{Mint,TokenAccount};
use anchor_spl::token_2022::Token2022;
use crate::{GlobalParams,MayhemError};
use crate::state::{MayhemState,MayhemStatus,PUMP_ID};
use super::sell::{SellCurve,SellEvent,SellError};
use crate::checked_program::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};

#[derive(Accounts)]
pub struct LegacyBuy<'info> {
 #[account(mut)]pub trader:Signer<'info>,
 #[account(mut,seeds=[b"global-params"],bump,has_one=trader)]pub global_params:Box<Account<'info,GlobalParams>>,
 #[account(mut,seeds=[b"mayhem-state",mint.key().as_ref()],bump)]pub mayhem_state:Box<Account<'info,MayhemState>>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=sol_vault,associated_token::token_program=mint.to_account_info().owner)]pub mayhem_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Original writable PDA constraint.
 #[account(mut,seeds=[b"sol-vault"],bump)]pub sol_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 pub token_program:Program<'info,Token2022>,
 /// CHECK: Passed through to Pump; original unchecked account.
 pub global:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP_ID)]pub bonding_curve:Box<Account<'info,SellCurve>>,
 /// CHECK: Pump validates fee recipient.
 #[account(mut)]pub fee_recipient:UncheckedAccount<'info>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=bonding_curve,associated_token::token_program=mint.to_account_info().owner)]pub associated_bonding_curve:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Pump validates creator vault.
 #[account(mut)]pub creator_vault:UncheckedAccount<'info>,
 /// CHECK: Pump event authority passed through.
 pub pump_event_authority:UncheckedAccount<'info>,
 /// CHECK: Original address constraint, without executable Program wrapper.
 #[account(address=PUMP_ID)]pub pump_program:UncheckedAccount<'info>,
 /// CHECK: Pump validates global volume state.
 #[account(mut)]pub global_volume_accumulator:UncheckedAccount<'info>,
 /// CHECK: Pump validates the sol-vault user volume account.
 #[account(mut)]pub user_volume_accumulator:UncheckedAccount<'info>,
 /// CHECK: Pump validates its fee config.
 pub fee_config:UncheckedAccount<'info>,
 /// CHECK: Pump validates its fee program.
 pub fee_program:UncheckedAccount<'info>,
 /// CHECK: Original runtime event PDA constraint.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::MayhemProgram>,
 /// CHECK: Original derives this under Mayhem, not the Pump program.
 #[account(seeds=[b"bonding-curve-v2",mint.key().as_ref()],bump)]pub bonding_curve_v2:UncheckedAccount<'info>,
 /// CHECK: Pump validates the recipient when used.
 #[account(mut)]pub buyback_recipient:UncheckedAccount<'info>,
}
fn math()->Error {MayhemError::MathOverflow.into()}
pub fn handler(ctx:Context<LegacyBuy>,reference:u128,tolerance_bps:u16,retained_reserve_bps:u16)->Result<()> {
 let a=&mut *ctx.accounts;
 require!(!a.global_params.paused,MayhemError::TradingPaused);
 let now=Clock::get().unwrap().unix_timestamp;
 let last=a.mayhem_state.field_89 as i64;
 let interval=i64::from_le_bytes(a.global_params.reset_middle[9..17].try_into().unwrap());
 if last!=0&&interval!=0 {require!(now>=last+interval,SellError::TradeRateLimit);}
 a.mayhem_state.field_89=now as u64;
 require!(Clock::get().unwrap().unix_timestamp<a.mayhem_state.end_timestamp,SellError::MayhemModeEnded);
 if a.mayhem_state.status!=MayhemStatus::Variant1 {
  require!(a.bonding_curve.real_quote_reserves>=u64::from_le_bytes(a.global_params.preserved_middle_a),SellError::SolReservesTooLow);
  a.mayhem_state.status=MayhemStatus::Variant1;
 }
 // Price-band check precedes quote construction and the real Pump CPI.
 let c=&a.bonding_curve;
 let vt=c.virtual_token_reserves as u128;let vs=c.virtual_quote_reserves as u128;
 if vt==0{return Err(math());}
 let scaled=reference.checked_mul(tolerance_bps as u128).ok_or_else(math)?;
 let price=vs*1_000_000_000_000_000u128/vt;let delta=scaled/10_000;
 let low=reference.checked_sub(delta).ok_or_else(math)?;let high=reference.checked_add(delta).ok_or_else(math)?;
 if price<low||price>high{return Err(SellError::PumpCpiSlippage.into());}
 let retained=10_000u16-retained_reserve_bps;
 let tokens=a.mayhem_token_vault.amount as u128;
 // The SOL budget is derived from current vault tokens, then constrained by
 // the optional global cap and retained real SOL reserves.
 let mut output=u64::try_from(vs*tokens/(vt+tokens)).unwrap();
 let cap=u64::from_le_bytes(a.global_params.reset_prefix[8..16].try_into().unwrap());
 if cap!=0 {output=output.min(cap);}
 output=output.min((c.real_quote_reserves as u128*retained as u128/10_000) as u64);
 let input=u64::try_from((output as u128*vt).checked_div(vs+output as u128).unwrap()).unwrap().min(c.real_token_reserves);
 let before_sol=a.sol_vault.lamports();let before_tokens=a.mayhem_token_vault.amount;
 let accounts=crate::pump::cpi::accounts::Buy{global:a.global.to_account_info(),fee_recipient:a.fee_recipient.to_account_info(),mint:a.mint.to_account_info(),bonding_curve:a.bonding_curve.to_account_info(),associated_bonding_curve:a.associated_bonding_curve.to_account_info(),associated_user:a.mayhem_token_vault.to_account_info(),user:a.sol_vault.to_account_info(),system_program:a.system_program.to_account_info(),token_program:a.token_program.to_account_info(),creator_vault:a.creator_vault.to_account_info(),event_authority:a.pump_event_authority.to_account_info(),program:a.pump_program.to_account_info(),global_volume_accumulator:a.global_volume_accumulator.to_account_info(),user_volume_accumulator:a.user_volume_accumulator.to_account_info(),fee_config:a.fee_config.to_account_info(),fee_program:a.fee_program.to_account_info()};
 let bump=[ctx.bumps.sol_vault];let seeds:&[&[u8]]=&[b"sol-vault",&bump];let signer=&[seeds];
 let cpi=CpiContext::new_with_signer(crate::pump::ID,accounts,signer).with_remaining_accounts(vec![a.bonding_curve_v2.to_account_info(),a.buyback_recipient.to_account_info()]);
 require!(crate::pump::cpi::buy(cpi,input,output,crate::pump::types::OptionBool(false)).is_ok(),BuyError::PumpCpiFailed);
 a.mayhem_token_vault.reload()?;a.bonding_curve.reload()?;
 let sol=before_sol-a.sol_vault.lamports();let tokens=a.mayhem_token_vault.amount-before_tokens;
 a.global_params.sol_deployed=a.global_params.sol_deployed.checked_add(sol as i128).ok_or_else(math)?;
 a.mayhem_state.field_56=a.mayhem_state.field_56.checked_add(sol as i128).ok_or_else(math)?;
 a.mayhem_state.field_72=a.mayhem_state.field_72.checked_sub(tokens as i128).ok_or_else(math)?;
 // Original tests signed positivity, then compares the low u64 limb with
 // a strict upper bound. Preserve that truncation even for counters > u64::MAX.
 if a.global_params.sol_deployed>0 {require_gt!(u64::from_le_bytes(a.global_params.reset_prefix[24..32].try_into().unwrap()),a.global_params.sol_deployed as u64,BuyError::OverallRiskExceeded);}
 if a.mayhem_state.field_56>0 {require_gt!(u64::from_le_bytes(a.global_params.reset_prefix[32..40].try_into().unwrap()),a.mayhem_state.field_56 as u64,BuyError::CoinRiskExceeded);}
 let event=SellEvent{side:0,sol_vault:a.sol_vault.key(),mint:a.mint.key(),sol_amount:sol,token_amount:tokens,sol_vault_balance:a.sol_vault.lamports(),token_vault_balance:a.mayhem_token_vault.amount,state_sol_deployed:a.mayhem_state.field_56,state_token_net:a.mayhem_state.field_72,virtual_sol:a.bonding_curve.virtual_quote_reserves,virtual_tokens:a.bonding_curve.virtual_token_reserves,global_sol_deployed:a.global_params.sol_deployed,timestamp:Clock::get()?.unix_timestamp,end_timestamp:a.mayhem_state.end_timestamp,real_sol:a.bonding_curve.real_quote_reserves,real_tokens:a.bonding_curve.real_token_reserves,version:a.global_params.version,retained_reserve_bps,quote_amount:sol,quote_vault_balance:a.sol_vault.lamports(),state_quote_deployed:a.mayhem_state.field_56,virtual_quote:a.bonding_curve.virtual_quote_reserves,global_quote_deployed:a.global_params.sol_deployed,real_quote:a.bonding_curve.real_quote_reserves,quote_mint:Pubkey::default()};
 emit!(event);emit_cpi!(event);Ok(())
}
#[error_code]
pub enum BuyError {
 #[msg("Overall risk tolerance exceeded")]OverallRiskExceeded=6,
 #[msg("Per coin risk tolerance exceeded")]CoinRiskExceeded=7,
 #[msg("Pump CPI call failed")]PumpCpiFailed=8,
}
}

pub mod sell {
//! Legacy sell recovered from the original dispatch, parsers and arithmetic.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction,program::invoke_signed};
use anchor_spl::token_interface::{Mint,TokenAccount};
use anchor_spl::token_2022::Token2022;
use crate::{GlobalParams,MayhemError};
use crate::state::{MayhemState,MayhemStatus,PUMP_ID};
use crate::checked_program::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};

#[derive(Accounts)]
pub struct LegacySell<'info> {
 #[account(mut)]pub trader:Signer<'info>,
 #[account(mut,seeds=[b"global-params"],bump,has_one=trader)]pub global_params:Box<Account<'info,GlobalParams>>,
 #[account(mut,seeds=[b"mayhem-state",mint.key().as_ref()],bump)]pub mayhem_state:Box<Account<'info,MayhemState>>,
 /// CHECK: Original unchecked PDA and mutable constraints.
 #[account(mut,seeds=[b"sol-vault"],bump)]pub sol_vault:UncheckedAccount<'info>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=sol_vault,associated_token::token_program=mint.to_account_info().owner)]pub mayhem_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 pub token_program:Program<'info,Token2022>,
 pub system_program:Program<'info,System>,
 /// CHECK: Passed through to Pump; original unchecked account.
 pub global:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP_ID)]pub bonding_curve:Box<Account<'info,SellCurve>>,
 /// CHECK: Pump validates fee recipient.
 #[account(mut)]pub fee_recipient:UncheckedAccount<'info>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=bonding_curve,associated_token::token_program=mint.to_account_info().owner)]pub associated_bonding_curve:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Pump validates creator vault.
 #[account(mut)]pub creator_vault:UncheckedAccount<'info>,
 /// CHECK: Pump event authority passed through.
 pub pump_event_authority:UncheckedAccount<'info>,
 /// CHECK: Original address constraint, without executable Program wrapper.
 #[account(address=PUMP_ID)]pub pump_program:UncheckedAccount<'info>,
 /// CHECK: Pump validates its fee config.
 pub fee_config:UncheckedAccount<'info>,
 /// CHECK: Pump validates its fee program.
 pub fee_program:UncheckedAccount<'info>,
 /// CHECK: Original runtime event PDA constraint.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::MayhemProgram>,
 /// CHECK: Original derives this under Mayhem, not the Pump program.
 #[account(seeds=[b"bonding-curve-v2",mint.key().as_ref()],bump)]pub bonding_curve_v2:UncheckedAccount<'info>,
 /// CHECK: Pump validates the recipient when used.
 #[account(mut)]pub buyback_recipient:UncheckedAccount<'info>,
}
/// Pump's 141-byte `BondingCurve` as `pump-cpi`'s `declare_program!(pump)` generates
/// it, read through `pump-cpi`'s zero-padding account wrapper (legacy and v2 trades).
/// The older, strict 82-byte layout used by burn is `crate::pump::accounts::BondingCurve`.
pub type SellCurve=pump_cpi::account_wrapper::AccountWrapper<pump_cpi::pump_v2::pump::accounts::BondingCurve>;
fn math()->Error {MayhemError::MathOverflow.into()}
pub fn handler(ctx:Context<LegacySell>,reference:u128,tolerance_bps:u16,retained_reserve_bps:u16)->Result<()> {
 let a=&mut *ctx.accounts;
 require!(!a.global_params.paused,MayhemError::TradingPaused);
 let now=Clock::get().unwrap().unix_timestamp;
 let last=a.mayhem_state.field_89 as i64;
 let interval=i64::from_le_bytes(a.global_params.reset_middle[9..17].try_into().unwrap());
 if last!=0&&interval!=0 {require!(now>=last+interval,SellError::TradeRateLimit);}
 a.mayhem_state.field_89=now as u64;
 require!(Clock::get().unwrap().unix_timestamp<a.mayhem_state.end_timestamp,SellError::MayhemModeEnded);
 if a.mayhem_state.status!=MayhemStatus::Variant1 {
  require!(a.bonding_curve.real_quote_reserves>=u64::from_le_bytes(a.global_params.preserved_middle_a),SellError::SolReservesTooLow);
  a.mayhem_state.status=MayhemStatus::Variant1;
 }
 let c=&a.bonding_curve;
 let vt=c.virtual_token_reserves as u128;let vs=c.virtual_quote_reserves as u128;
 if vt==0{return Err(math());}
 let scaled=reference.checked_mul(tolerance_bps as u128).ok_or_else(math)?;
 let price=vs*1_000_000_000_000_000u128/vt;let delta=scaled/10_000;
 let low=reference.checked_sub(delta).ok_or_else(math)?;let high=reference.checked_add(delta).ok_or_else(math)?;
 if price<low||price>high{return Err(SellError::PumpCpiSlippage.into());}
 let retained=10_000u16-retained_reserve_bps;
 let tokens=a.mayhem_token_vault.amount as u128;
 let mut output=u64::try_from(vs*tokens/(vt+tokens)).unwrap();
 let cap=u64::from_le_bytes(a.global_params.reset_prefix[8..16].try_into().unwrap());
 if cap!=0 {output=output.min(cap);}
 output=output.min((c.real_quote_reserves as u128*retained as u128/10_000) as u64);
 let input=u64::try_from((output as u128*vt).checked_div(vs-output as u128).unwrap()).unwrap();
 let minimum=output.saturating_sub(1);
 let before_sol=a.sol_vault.lamports();let before_tokens=a.mayhem_token_vault.amount;
 let accounts=crate::pump::cpi::accounts::Sell{global:a.global.to_account_info(),fee_recipient:a.fee_recipient.to_account_info(),mint:a.mint.to_account_info(),bonding_curve:a.bonding_curve.to_account_info(),associated_bonding_curve:a.associated_bonding_curve.to_account_info(),associated_user:a.mayhem_token_vault.to_account_info(),user:a.sol_vault.to_account_info(),system_program:a.system_program.to_account_info(),creator_vault:a.creator_vault.to_account_info(),token_program:a.token_program.to_account_info(),event_authority:a.pump_event_authority.to_account_info(),program:a.pump_program.to_account_info(),fee_config:a.fee_config.to_account_info(),fee_program:a.fee_program.to_account_info()};
 let bump=[ctx.bumps.sol_vault];let seeds:&[&[u8]]=&[b"sol-vault",&bump];let signer=&[seeds];
 let cpi=CpiContext::new_with_signer(crate::pump::ID,accounts,signer).with_remaining_accounts(vec![a.bonding_curve_v2.to_account_info(),a.buyback_recipient.to_account_info()]);
 crate::pump::cpi::sell(cpi,input,minimum)?;
 a.mayhem_token_vault.reload()?;a.bonding_curve.reload()?;
 let sol=a.sol_vault.lamports()-before_sol;let tokens=before_tokens-a.mayhem_token_vault.amount;
 a.global_params.sol_deployed=a.global_params.sol_deployed.checked_sub(sol as i128).ok_or_else(math)?;
 a.mayhem_state.field_56=a.mayhem_state.field_56.checked_sub(sol as i128).ok_or_else(math)?;
 a.mayhem_state.field_72=a.mayhem_state.field_72.checked_add(tokens as i128).ok_or_else(math)?;
 let event=SellEvent{side:1,sol_vault:a.sol_vault.key(),mint:a.mint.key(),sol_amount:sol,token_amount:tokens,sol_vault_balance:a.sol_vault.lamports(),token_vault_balance:a.mayhem_token_vault.amount,state_sol_deployed:a.mayhem_state.field_56,state_token_net:a.mayhem_state.field_72,virtual_sol:a.bonding_curve.virtual_quote_reserves,virtual_tokens:a.bonding_curve.virtual_token_reserves,global_sol_deployed:a.global_params.sol_deployed,timestamp:Clock::get()?.unix_timestamp,end_timestamp:a.mayhem_state.end_timestamp,real_sol:a.bonding_curve.real_quote_reserves,real_tokens:a.bonding_curve.real_token_reserves,version:a.global_params.version,retained_reserve_bps,quote_amount:sol,quote_vault_balance:a.sol_vault.lamports(),state_quote_deployed:a.mayhem_state.field_56,virtual_quote:a.bonding_curve.virtual_quote_reserves,global_quote_deployed:a.global_params.sol_deployed,real_quote:a.bonding_curve.real_quote_reserves,quote_mint:Pubkey::default()};
 emit!(event);emit_cpi!(event);Ok(())
}
#[event(discriminator=[0x31,0x48,0x7b,0x2d,0x6e,0x40,0xb0,0x85])]
pub struct SellEvent {
 pub side:u8,pub sol_vault:Pubkey,pub mint:Pubkey,
 pub sol_amount:u64,pub token_amount:u64,pub sol_vault_balance:u64,pub token_vault_balance:u64,
 pub state_sol_deployed:i128,pub state_token_net:i128,pub virtual_sol:u64,pub virtual_tokens:u64,
 pub global_sol_deployed:i128,pub timestamp:i64,pub end_timestamp:i64,pub real_sol:u64,pub real_tokens:u64,
 pub version:u32,pub retained_reserve_bps:u16,pub quote_amount:u64,pub quote_vault_balance:u64,
 pub state_quote_deployed:i128,pub virtual_quote:u64,pub global_quote_deployed:i128,pub real_quote:u64,pub quote_mint:Pubkey,
}
#[error_code]
pub enum SellError {
 #[msg("Too little sol reserves")]SolReservesTooLow=3,
 #[msg("Mayhem mode ended")]MayhemModeEnded=4,
 #[msg("Price impact exceeds slippage tolerance in Pump CPI call")]PumpCpiSlippage=12,
 #[msg("Trade Rate limit")]TradeRateLimit=15,
}
}
