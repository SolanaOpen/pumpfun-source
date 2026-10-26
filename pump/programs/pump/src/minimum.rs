use anchor_lang::prelude::*;
use crate::{PumpError,distribution::{FEES,SharingConfig,DistributionCurve}};
#[derive(Accounts)]
pub struct GetMinimumDistributableFee<'info>{
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump)]pub bonding_curve:Account<'info,DistributionCurve>,
 #[account(seeds=[b"sharing-config",mint.key().as_ref()],bump,seeds::program=FEES,has_one=mint @ PumpError::MintDoesNotMatchBondingCurve)]pub sharing_config:Account<'info,SharingConfig>,
 /// CHECK: only seeds and available lamports are used by this readonly query.
 #[account(seeds=[b"creator-vault",sharing_config.key().as_ref()],bump)]pub creator_vault:UncheckedAccount<'info>,
}
#[event]
#[derive(Clone)]
pub struct MinimumDistributableFeeEvent{pub minimum_required:u64,pub distributable_fees:u64,pub can_distribute:bool}
pub fn execute<'info>(ctx:Context<'_, '_, '_, 'info,GetMinimumDistributableFee<'info>>)->Result<MinimumDistributableFeeEvent>{
 let a=&ctx.accounts;require_keys_eq!(a.bonding_curve.creator,a.sharing_config.key(),PumpError::BondingCurveAndSharingConfigCreatorMismatch);
 require!(ctx.remaining_accounts.len()>=a.sharing_config.shareholders.len(),PumpError::NotEnoughRemainingAccounts);
 let rent=Rent::get()?;let reserve=rent.minimum_balance(0);let available=a.creator_vault.lamports().saturating_sub(reserve);let mut required=0u64;
 for (s,r) in a.sharing_config.shareholders.iter().zip(ctx.remaining_accounts){
  require_keys_eq!(s.address,r.key(),PumpError::ShareholdersAndRemainingAccountsMismatch);
  require!(!r.executable,PumpError::UnableToDistributeCreatorFeesToExecutableRecipient);
  require!(s.share_bps>0,PumpError::InvalidShareBps);
  let deficit=rent.minimum_balance(r.data_len()).saturating_sub(r.lamports());
  let threshold=((deficit as u128)*10000+(s.share_bps as u128)-1)/(s.share_bps as u128);
  required=required.max(u64::try_from(threshold).map_err(|_|error!(PumpError::Overflow))?);
 }
 let event=MinimumDistributableFeeEvent{minimum_required:reserve.checked_add(required).ok_or(error!(PumpError::Overflow))?,distributable_fees:available,can_distribute:available>=required};emit!(event.clone());Ok(event)
}
