//! Recovered sharing reset, tested through the genuine captured Pump admin_cto caller.
use anchor_lang::prelude::*;
use crate::{FeeError, ID, pump_global};
use crate::types::Shareholder;
use crate::donation::{ConfigStatus, SharingConfig};
pub fn admin_cto_sharing_config(ctx:Context<AdminCtoSharingConfig>,new_admin:Option<Pubkey>)->Result<()> {
  let state=&mut ctx.accounts.sharing_config;
  let old_admin=state.admin;let old_shareholders=state.shareholders.clone();let old_version=state.version;
  match new_admin {
   Some(admin)=>{state.admin=admin;state.admin_revoked=false;state.status=ConfigStatus::Active;state.shareholders=vec![Shareholder{address:admin,share_bps:10000}];}
   None=>{state.admin=Pubkey::default();state.admin_revoked=true;state.status=ConfigStatus::Paused;state.shareholders=vec![];}
  }
  state.version=2;
  let event=ResetFeeSharingConfigEvent{timestamp:Clock::get()?.unix_timestamp,mint:ctx.accounts.mint.key(),sharing_config:state.key(),old_admin,old_shareholders,new_admin:state.admin,new_shareholders:state.shareholders.clone(),old_version,new_version:state.version};
  emit!(event);emit_cpi!(event);Ok(())
 }

#[derive(Accounts)]
pub struct AdminCtoSharingConfig<'info>{
 #[account(seeds=[b"pool-authority",mint.key().as_ref()],bump,seeds::program=pump_global::PUMP_ID)]pub pool_authority:Signer<'info>,
 pub authority:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=pump_global::PUMP_ID,constraint=global.admin_set_creator_authority==authority.key() @ FeeError::NotAuthorized)]pub global:Account<'info,pump_global::Global>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(mut,seeds=[b"sharing-config",mint.key().as_ref()],bump=sharing_config.bump,has_one=mint @ FeeError::InvalidSharingConfig)]pub sharing_config:Account<'info,SharingConfig>,
 /// CHECK: canonical event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: fixed self program used for event CPI.
 #[account(address=ID)]pub program:UncheckedAccount<'info>,
}
#[event]pub struct ResetFeeSharingConfigEvent{pub timestamp:i64,pub mint:Pubkey,pub sharing_config:Pubkey,pub old_admin:Pubkey,pub old_shareholders:Vec<Shareholder>,pub new_admin:Pubkey,pub new_shareholders:Vec<Shareholder>,pub old_version:u8,pub new_version:u8}
