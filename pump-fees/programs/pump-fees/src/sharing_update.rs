//! Recovered sharing updates; captured Pump/AMM dependencies execute their own CPIs.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::{Instruction,AccountMeta},program::invoke};
use crate::{FeeError, pump_global};
use crate::types::Shareholder;
use crate::donation::{PUMP, AMM, BondingCurve, ConfigStatus, SharingConfig};
use crate::sharing::{Pump, Amm};
// Static legacy source hypothesis, bound to the frozen 8ff12 source. Not compiled.
pub fn update_fee_shares<'info>(mut ctx:Context<'_, '_, '_, 'info,UpdateFeeShares<'info>>,shareholders:Vec<Shareholder>)->Result<()> {
  // The temporary AccountInfo and its data borrow end before version/auth checks.
  let quote_is_zero = legacy_curve_quote_is_zero(&ctx.accounts.bonding_curve.to_account_info())?;
  require!(quote_is_zero,FeeError::UnsupportedQuoteMint);
  require!(ctx.accounts.sharing_config.version!=1,FeeError::FeeSharesAlreadyUpdated);
  legacy_authorize(&ctx.accounts.sharing_config,ctx.accounts.authority.key())?;
  ctx.remaining_accounts = preprocess_remaining(
      ctx.remaining_accounts, &ctx.accounts.bonding_curve,
      &ctx.accounts.mint.key(), &anchor_spl::token::spl_token::native_mint::ID,
  )?;
  // Own the fixed accounts before the adapter's reserve/owner guard.
  legacy_transfer_creator_fees_to_pump(
      &ctx.accounts.bonding_curve,
      crate::pump_amm::cpi::accounts::TransferCreatorFeesToPump {
          program: ctx.accounts.pump_amm_program.to_account_info(),
          wsol_mint: ctx.accounts.wsol_mint.to_account_info(),
          token_program: ctx.accounts.token_program.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
          coin_creator: ctx.accounts.sharing_config.to_account_info(),
          coin_creator_vault_authority: ctx.accounts.coin_creator_vault_authority.to_account_info(),
          coin_creator_vault_ata: ctx.accounts.coin_creator_vault_ata.to_account_info(),
          pump_creator_vault: ctx.accounts.pump_creator_vault.to_account_info(),
          event_authority: ctx.accounts.amm_event_authority.to_account_info(),
      },
  )?;
  // Existing shareholder validation belongs here, after the AMM call.
  let payout_accounts = legacy_payout_accounts(
      &mut ctx.accounts.sharing_config, ctx.remaining_accounts,
  )?;
  if !payout_accounts.is_empty() {
  legacy_distribute_creator_fees(
      crate::pump::cpi::accounts::DistributeCreatorFees {
          program: ctx.accounts.pump_program.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          bonding_curve: ctx.accounts.bonding_curve.to_account_info(),
          sharing_config: ctx.accounts.sharing_config.to_account_info(),
          creator_vault: ctx.accounts.pump_creator_vault.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          event_authority: ctx.accounts.pump_event_authority.to_account_info(),
      },
      payout_accounts,
  )?;
  }
  legacy_validate_shares(&shareholders)?;
  let state=&mut ctx.accounts.sharing_config;state.shareholders=shareholders.clone();state.admin_revoked=true;
  let event=UpdateFeeSharesEvent{timestamp:Clock::get()?.unix_timestamp,mint:ctx.accounts.mint.key(),sharing_config:state.key(),admin:state.admin,new_shareholders:shareholders,version:state.version};emit!(event);emit_cpi!(event);Ok(())
 }
 pub fn update_fee_shares_v2<'info>(mut ctx:Context<'_, '_, '_, 'info,UpdateFeeSharesV2<'info>>,shareholders:Vec<Shareholder>)->Result<()> {
  require!(ctx.accounts.sharing_config.version!=1,FeeError::FeeSharesAlreadyUpdated);
  // Same complete authorization target [234096,235304) as legacy.
  legacy_authorize(&ctx.accounts.sharing_config,ctx.accounts.authority.key())?;
  let quote = ctx.accounts.quote_mint.key();
  let expected = v2_expected_quote(&ctx.accounts.bonding_curve.to_account_info());
  if quote != expected {
      return Err(error!(FeeError::InvalidQuoteMint).with_pubkeys((quote, expected)));
  }
  ctx.remaining_accounts = preprocess_remaining(
      ctx.remaining_accounts, &ctx.accounts.bonding_curve,
      &ctx.accounts.mint.key(), &expected,
  )?;
  v2_transfer_creator_fees_to_pump(
      &ctx.accounts.bonding_curve,
      crate::pump_amm::cpi::accounts::TransferCreatorFeesToPumpV2 {
          payer: ctx.accounts.authority.to_account_info(),
          quote_mint: ctx.accounts.quote_mint.to_account_info(),
          token_program: ctx.accounts.token_program.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
          coin_creator: ctx.accounts.sharing_config.to_account_info(),
          coin_creator_vault_authority: ctx.accounts.coin_creator_vault_authority.to_account_info(),
          coin_creator_vault_ata: ctx.accounts.coin_creator_vault_ata.to_account_info(),
          pump_creator_vault: ctx.accounts.pump_creator_vault.to_account_info(),
          pump_creator_vault_ata: ctx.accounts.pump_creator_vault_ata.to_account_info(),
          event_authority: ctx.accounts.amm_event_authority.to_account_info(),
          program: ctx.accounts.pump_amm_program.to_account_info(),
      },
  )?;
  // The cloned AccountInfo/borrow die before payout preparation.
  let zero_quote = legacy_curve_quote_is_zero(&ctx.accounts.bonding_curve.to_account_info())?;
  let prepared = if zero_quote {
      prepare_v2_zero_payout(ctx.remaining_accounts, &ctx.accounts.sharing_config.shareholders)?
  } else {
      prepare_v2_quote_payout(ctx.remaining_accounts, &ctx.accounts.sharing_config.shareholders)?
  };
  if prepared.changed {
      ctx.accounts.sharing_config.shareholders = prepared.shareholders;
      ctx.accounts.sharing_config.exit(&crate::ID)?;
  }
  if !prepared.accounts.is_empty() {
      v2_distribute_creator_fees(
          crate::pump::cpi::accounts::DistributeCreatorFeesV2 {
              payer: ctx.accounts.authority.to_account_info(),
              mint: ctx.accounts.mint.to_account_info(),
              bonding_curve: ctx.accounts.bonding_curve.to_account_info(),
              sharing_config: ctx.accounts.sharing_config.to_account_info(),
              creator_vault: ctx.accounts.pump_creator_vault.to_account_info(),
              system_program: ctx.accounts.system_program.to_account_info(),
              event_authority: ctx.accounts.pump_event_authority.to_account_info(),
              program: ctx.accounts.pump_program.to_account_info(),
              creator_vault_quote_token_account: ctx.accounts.pump_creator_vault_ata.to_account_info(),
              quote_mint: ctx.accounts.quote_mint.to_account_info(),
              quote_token_program: ctx.accounts.token_program.to_account_info(),
              associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
          },
          prepared.accounts,
      )?;
  }
  v2_validate_shares(&shareholders)?;
  let state=&mut ctx.accounts.sharing_config;state.shareholders=shareholders.clone();state.admin_revoked=true;
  let event=UpdateFeeSharesEvent{timestamp:Clock::get()?.unix_timestamp,mint:ctx.accounts.mint.key(),sharing_config:state.key(),admin:state.admin,new_shareholders:shareholders,version:state.version};emit!(event);emit_cpi!(event);Ok(())
 }

fn legacy_curve_quote_is_zero(info: &AccountInfo<'_>) -> Result<bool> {
  if info.data_len() < 115 { return Ok(true); }
  let data = info.data.borrow();
  Ok(Pubkey::new_from_array(data[83..115].try_into().unwrap()) == Pubkey::default())
}

fn authorize(state:&SharingConfig,authority:Pubkey)->Result<()>{
 require_keys_eq!(state.admin,authority,FeeError::NotAuthorized);
 require!(!state.admin_revoked,FeeError::SharingConfigAdminRevoked);
 require!(state.status==ConfigStatus::Active,FeeError::SharingConfigNotActive);Ok(())
}
fn validate_remaining(state:&SharingConfig,remaining:&[AccountInfo],stride:usize)->Result<()>{
 require!(remaining.len()>=state.shareholders.len()*stride,FeeError::NotEnoughRemainingAccounts);
 for (i,share) in state.shareholders.iter().enumerate(){require_keys_eq!(remaining[i].key(),share.address,FeeError::ShareholderAccountMismatch);}Ok(())
}
fn validate_shares(shares:&[Shareholder])->Result<()>{
 require!(!shares.is_empty(),FeeError::NoShareholders);require!(shares.len()<=10,FeeError::TooManyShareholders);
 let mut sum=0u16;let mut seen=Vec::new();
 for share in shares {require!(share.share_bps>0,FeeError::ZeroShareNotAllowed);sum=sum.checked_add(share.share_bps).ok_or(error!(FeeError::ShareCalculationOverflow))?;require!(!seen.contains(&share.address),FeeError::DuplicateShareholder);seen.push(share.address);}
 require!(sum==10000,FeeError::InvalidShareTotal);Ok(())
}
fn call<'info>(program_id:Pubkey,mut infos:Vec<AccountInfo<'info>>,flags:&[(bool,bool)],data:Vec<u8>,remaining:&[AccountInfo<'info>])->Result<()>{
 let mut accounts:Vec<AccountMeta>=infos.iter().zip(flags).map(|(a,(w,s))|if *w{AccountMeta::new(a.key(),*s)}else{AccountMeta::new_readonly(a.key(),*s)}).collect();
 accounts.extend(remaining.iter().map(|a|if a.is_writable{AccountMeta::new(a.key(),a.is_signer)}else{AccountMeta::new_readonly(a.key(),a.is_signer)}));infos.extend_from_slice(remaining);invoke(&Instruction{program_id,accounts,data},&infos)?;Ok(())
}
#[event]
pub struct UpdateFeeSharesEvent{pub timestamp:i64,pub mint:Pubkey,pub sharing_config:Pubkey,pub admin:Pubkey,pub new_shareholders:Vec<Shareholder>,pub version:u8}
#[derive(Accounts)]
pub struct UpdateFeeShares<'info>{
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::PumpFeesCandidate>,
 pub authority:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=PUMP)]pub global:Box<Account<'info,pump_global::Global>>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(mut,seeds=[b"sharing-config",mint.key().as_ref()],bump=sharing_config.bump,has_one=mint @ FeeError::InvalidSharingConfig)]pub sharing_config:Account<'info,SharingConfig>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP)]pub bonding_curve:Account<'info,BondingCurve>,
 /// CHECK: creator vault PDA.
 #[account(mut,seeds=[b"creator-vault",sharing_config.key().as_ref()],bump,seeds::program=PUMP)]pub pump_creator_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 pub pump_program:Program<'info,Pump>,
 /// CHECK: Pump event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=PUMP)]pub pump_event_authority:UncheckedAccount<'info>,
 pub pump_amm_program:Program<'info,Amm>,
 /// CHECK: AMM event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=AMM)]pub amm_event_authority:UncheckedAccount<'info>,
 #[account(address=anchor_spl::token::spl_token::native_mint::ID)]pub wsol_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(address=anchor_spl::token::ID)]pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 /// CHECK: AMM creator vault PDA.
 #[account(mut,seeds=[b"creator_vault",sharing_config.key().as_ref()],bump,seeds::program=AMM)]pub coin_creator_vault_authority:UncheckedAccount<'info>,
 /// CHECK: canonical token account address; dependency validates data.
 #[account(mut,address=anchor_spl::associated_token::get_associated_token_address_with_program_id(&coin_creator_vault_authority.key(),&wsol_mint.key(),&token_program.key()))]pub coin_creator_vault_ata:UncheckedAccount<'info>,
}
#[derive(Accounts)]
pub struct UpdateFeeSharesV2<'info>{
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::PumpFeesCandidate>,
 #[account(mut)]pub authority:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=PUMP)]pub global:Box<Account<'info,pump_global::Global>>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(mut,seeds=[b"sharing-config",mint.key().as_ref()],bump=sharing_config.bump,has_one=mint @ FeeError::InvalidSharingConfig)]pub sharing_config:Account<'info,SharingConfig>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP)]pub bonding_curve:Account<'info,BondingCurve>,
 /// CHECK: creator vault PDA.
 #[account(mut,seeds=[b"creator-vault",sharing_config.key().as_ref()],bump,seeds::program=PUMP)]pub pump_creator_vault:UncheckedAccount<'info>,
 /// CHECK: canonical token address, validated by dependency.
 #[account(mut,seeds=[pump_creator_vault.key().as_ref(),token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub pump_creator_vault_ata:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 pub pump_program:Program<'info,Pump>,
 /// CHECK: Pump event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=PUMP)]pub pump_event_authority:UncheckedAccount<'info>,
 pub pump_amm_program:Program<'info,Amm>,
 /// CHECK: AMM event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=AMM)]pub amm_event_authority:UncheckedAccount<'info>,
pub quote_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 /// CHECK: AMM creator vault PDA.
 #[account(mut,seeds=[b"creator_vault",sharing_config.key().as_ref()],bump,seeds::program=AMM)]pub coin_creator_vault_authority:UncheckedAccount<'info>,
 /// CHECK: canonical token account address; dependency validates data.
 #[account(mut,seeds=[coin_creator_vault_authority.key().as_ref(),token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub coin_creator_vault_ata:UncheckedAccount<'info>,
}

// Original [234096,235304): no compared-key payload on authorization failure.
fn legacy_authorize(state: &SharingConfig, authority: Pubkey) -> Result<()> {
    require!(authority == state.admin, FeeError::NotAuthorized);
    if state.admin_revoked { return Err(FeeError::SharingConfigAdminRevoked.into()); }
    require!(state.status == ConfigStatus::Active, FeeError::SharingConfigNotActive);
    Ok(())
}

struct PreparedLegacyPayout<'info> {
    shareholders: Vec<Shareholder>,
    changed: bool,
    accounts: Vec<AccountInfo<'info>>,
}

// Original [199072,204080). Per-entry availability precedes key and executable.
fn prepare_legacy_payout<'info>(
    remaining: &[AccountInfo<'info>], shareholders: &[Shareholder],
) -> Result<PreparedLegacyPayout<'info>> {
    let mut accounts = Vec::with_capacity(remaining.len());
    let mut retained = Vec::with_capacity(shareholders.len());
    let mut sum = 0u16;
    let mut max_weight = 0u16;
    let mut max_index = 0usize;
    for (i, share) in shareholders.iter().enumerate() {
        require!(i < remaining.len(), FeeError::NotEnoughRemainingAccounts);
        let account = &remaining[i];
        if account.key() != share.address {
            return Err(error!(FeeError::ShareholderAccountMismatch)
                .with_values((account.key(), share.address)));
        }
        if account.executable { continue; }
        retained.push(share.clone());
        accounts.push(account.clone());
        if share.share_bps > max_weight {
            max_weight = share.share_bps;
            max_index = retained.len() - 1;
        }
        sum = sum.saturating_add(share.share_bps);
    }
    if retained.len() == shareholders.len() {
        // The unchanged result includes trailing accounts, with their real flags.
        return Ok(PreparedLegacyPayout {
            shareholders: retained, changed: false, accounts: remaining.to_vec(),
        });
    }
    if retained.is_empty() {
        return Ok(PreparedLegacyPayout {
            shareholders: Vec::new(), changed: true, accounts: Vec::new(),
        });
    }
    let mut normalized_sum = 0u16;
    for share in &mut retained {
        share.share_bps = if sum == 0 { 0 } else {
            ((u64::from(share.share_bps) * 10_000 / u64::from(sum)).min(10_000)) as u16
        };
        normalized_sum = normalized_sum.saturating_add(share.share_bps);
    }
    retained[max_index].share_bps = retained[max_index].share_bps
        .saturating_add(10_000u16.saturating_sub(normalized_sum));
    Ok(PreparedLegacyPayout { shareholders: retained, changed: true, accounts })
}

// Original [204080,205768). This is a legacy adapter, not the V2 caller.
fn legacy_payout_accounts<'info>(
    state: &mut Account<'info, SharingConfig>, remaining: &[AccountInfo<'info>],
) -> Result<Vec<AccountInfo<'info>>> {
    let prepared = prepare_legacy_payout(remaining, &state.shareholders)?;
    if prepared.changed {
        state.shareholders = prepared.shareholders;
        state.exit(&crate::ID)?;
        Ok(prepared.accounts)
    } else {
        // The original adapter makes a second copy before dropping prepared.accounts.
        Ok(remaining.to_vec())
    }
}

// Original [205768,211408): scalar validation, then total, then duplicates.
fn legacy_validate_shares(shares: &[Shareholder]) -> Result<()> {
    require!(!shares.is_empty(), FeeError::NoShareholders);
    require!(shares.len() <= 10, FeeError::TooManyShareholders);
    let mut sum = 0u16;
    let mut keys = Vec::with_capacity(shares.len());
    for share in shares {
        require!(share.share_bps > 0, FeeError::ZeroShareNotAllowed);
        sum = sum.checked_add(share.share_bps).ok_or(FeeError::ShareCalculationOverflow)?;
        keys.push(share.address);
    }
    keys.sort_unstable();
    keys.dedup();
    require!(sum == 10_000, FeeError::InvalidShareTotal);
    require!(keys.len() == shares.len(), FeeError::DuplicateShareholder);
    Ok(())
}

// Original adapters own fixed accounts; generated CPI owns their final cleanup.
fn legacy_transfer_creator_fees_to_pump<'info>(
    curve: &BondingCurve,
    accounts: crate::pump_amm::cpi::accounts::TransferCreatorFeesToPump<'info>,
) -> Result<()> {
    if curve.real_sol_reserves == 0
        && curve.virtual_sol_reserves == 0
        && curve.real_token_reserves == 0
        && curve.virtual_token_reserves == 0
        && accounts.coin_creator_vault_ata.owner != &System::id()
    {
        let program = accounts.program.clone();
        crate::pump_amm::cpi::transfer_creator_fees_to_pump(
            CpiContext::new(program, accounts),
        )?;
    }
    Ok(())
}

fn legacy_distribute_creator_fees<'info>(
    accounts: crate::pump::cpi::accounts::DistributeCreatorFees<'info>,
    remaining: Vec<AccountInfo<'info>>,
) -> Result<()> {
    let program = accounts.program.clone();
    crate::pump::cpi::distribute_creator_fees(
        CpiContext::new(program, accounts).with_remaining_accounts(remaining),
    )?;
    Ok(())
}

fn preprocess_remaining<'remaining, 'info>(
    remaining: &'remaining [AccountInfo<'info>],
    curve: &BondingCurve,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
) -> Result<&'remaining [AccountInfo<'info>]> {
    if !curve.complete {
        return Ok(remaining);
    }
    require!(!remaining.is_empty(), FeeError::NotEnoughRemainingAccounts);

    let (authority, _) = Pubkey::find_program_address(
        &[b"pool-authority", base_mint.as_ref()], &PUMP,
    );
    let (expected, _) = Pubkey::find_program_address(
        &[b"pool", &0u16.to_le_bytes(), authority.as_ref(),
          base_mint.as_ref(), quote_mint.as_ref()], &AMM,
    );

    // The nonempty check dominates this subtraction and both slice accesses.
    let last_index = remaining.len() - 1;
    let last = &remaining[last_index];
    require_keys_eq!(*last.key, expected, anchor_lang::error::ErrorCode::ConstraintSeeds);

    // A different owner is deliberately accepted after the key check.
    if last.owner == &AMM {
        let data = last.try_borrow_data()?;
        if data.len() >= 287 {
            let word = u64::from_le_bytes(data[279..287].try_into().unwrap());
            require!(word == 0, FeeError::PoolCreatorFeesNotSwept);
        }
    }
    Ok(&remaining[..last_index])
}

// Static V2 source fragment. Historical diagnostics/build identity remain open.
struct PreparedV2Payout<'info> {
    shareholders: Vec<Shareholder>,
    changed: bool,
    accounts: Vec<AccountInfo<'info>>,
}

// [219376,225920): two account blocks, not interleaved pairs.
fn prepare_v2_quote_payout<'info>(
    remaining: &[AccountInfo<'info>], shareholders: &[Shareholder],
) -> Result<PreparedV2Payout<'info>> {
    let n = shareholders.len();
    let mut accounts = Vec::with_capacity(n);
    let mut partners = Vec::with_capacity(n);
    let mut retained = Vec::with_capacity(n);
    let mut sum = 0u16;
    let mut max_weight = 0u16;
    let mut max_index = 0usize;
    for (i, share) in shareholders.iter().enumerate() {
        require!(i < remaining.len(), FeeError::NotEnoughRemainingAccounts);
        let j = n + i;
        require!(j < remaining.len(), FeeError::NotEnoughRemainingAccounts);
        let account = &remaining[i];
        if account.key() != share.address {
            return Err(error!(FeeError::ShareholderAccountMismatch)
                .with_values((account.key(), share.address)));
        }
        if account.executable { continue; }
        retained.push(share.clone());
        accounts.push(account.clone());
        partners.push(remaining[j].clone());
        if share.share_bps > max_weight {
            max_weight = share.share_bps;
            max_index = retained.len() - 1;
        }
        sum = sum.saturating_add(share.share_bps);
    }
    if retained.len() == n {
        // Clone the complete slice BEFORE dropping either temporary buffer.
        let all = remaining.to_vec();
        drop(partners);
        drop(accounts);
        return Ok(PreparedV2Payout { shareholders: retained, changed: false, accounts: all });
    }
    if retained.is_empty() {
        return Ok(PreparedV2Payout {
            shareholders: Vec::new(), changed: true, accounts: Vec::new(),
        });
    }
    v2_rescale_retained(&mut retained, sum, max_index);
    accounts.append(&mut partners);
    Ok(PreparedV2Payout { shareholders: retained, changed: true, accounts })
}

// Same owned zero-quote contract [199072,204080), without changing legacy code.
fn prepare_v2_zero_payout<'info>(
    remaining: &[AccountInfo<'info>], shareholders: &[Shareholder],
) -> Result<PreparedV2Payout<'info>> {
    let mut accounts = Vec::with_capacity(remaining.len());
    let mut retained = Vec::with_capacity(shareholders.len());
    let mut sum = 0u16;
    let mut max_weight = 0u16;
    let mut max_index = 0usize;
    for (i, share) in shareholders.iter().enumerate() {
        require!(i < remaining.len(), FeeError::NotEnoughRemainingAccounts);
        let account = &remaining[i];
        if account.key() != share.address {
            return Err(error!(FeeError::ShareholderAccountMismatch)
                .with_values((account.key(), share.address)));
        }
        if account.executable { continue; }
        retained.push(share.clone());
        accounts.push(account.clone());
        if share.share_bps > max_weight {
            max_weight = share.share_bps;
            max_index = retained.len() - 1;
        }
        sum = sum.saturating_add(share.share_bps);
    }
    if retained.len() == shareholders.len() {
        return Ok(PreparedV2Payout {
            shareholders: retained, changed: false, accounts: remaining.to_vec(),
        });
    }
    if retained.is_empty() {
        return Ok(PreparedV2Payout {
            shareholders: Vec::new(), changed: true, accounts: Vec::new(),
        });
    }
    v2_rescale_retained(&mut retained, sum, max_index);
    Ok(PreparedV2Payout { shareholders: retained, changed: true, accounts })
}

// Arithmetic contract shared by the two V2-owned preparation fragments.
// This factoring is a source hypothesis, not a recovered historical function.
fn v2_rescale_retained(retained: &mut [Shareholder], sum: u16, max_index: usize) {
    let mut normalized_sum = 0u16;
    for share in retained.iter_mut() {
        share.share_bps = if sum == 0 { 0 } else {
            ((u64::from(share.share_bps) * 10_000 / u64::from(sum)).min(10_000)) as u16
        };
        normalized_sum = normalized_sum.saturating_add(share.share_bps);
    }
    retained[max_index].share_bps = retained[max_index].share_bps
        .saturating_add(10_000u16.saturating_sub(normalized_sum));
}

// [212344,213328): data_len and infallible borrow, not try_borrow_data.
fn v2_expected_quote(info: &AccountInfo<'_>) -> Pubkey {
    let stored = if info.data_len() < 115 { Pubkey::default() } else {
        let data = info.data.borrow();
        Pubkey::new_from_array(data[83..115].try_into().unwrap())
    };
    if stored == Pubkey::default() {
        anchor_spl::token::spl_token::native_mint::ID
    } else { stored }
}

fn v2_transfer_creator_fees_to_pump<'info>(
    curve: &BondingCurve,
    accounts: crate::pump_amm::cpi::accounts::TransferCreatorFeesToPumpV2<'info>,
) -> Result<()> {
    if curve.real_sol_reserves == 0
        && curve.virtual_sol_reserves == 0
        && curve.real_token_reserves == 0
        && curve.virtual_token_reserves == 0
        && accounts.coin_creator_vault_ata.owner != &System::id()
    {
        let program = accounts.program.clone();
        crate::pump_amm::cpi::transfer_creator_fees_to_pump_v2(
            CpiContext::new(program, accounts),
        )?;
    }
    Ok(())
}

fn v2_distribute_creator_fees<'info>(
    accounts: crate::pump::cpi::accounts::DistributeCreatorFeesV2<'info>,
    remaining: Vec<AccountInfo<'info>>,
) -> Result<()> {
    let program = accounts.program.clone();
    crate::pump::cpi::distribute_creator_fees_v2(
        CpiContext::new(program, accounts).with_remaining_accounts(remaining), true,
    )?;
    Ok(())
}

// [225920,231576): scalar errors, then total, then duplicate count.
fn v2_validate_shares(shares: &[Shareholder]) -> Result<()> {
    require!(!shares.is_empty(), FeeError::NoShareholders);
    require!(shares.len() <= 10, FeeError::TooManyShareholders);
    let mut sum = 0u16;
    let mut keys = Vec::with_capacity(shares.len());
    for share in shares {
        require!(share.share_bps > 0, FeeError::ZeroShareNotAllowed);
        sum = sum.checked_add(share.share_bps).ok_or(FeeError::ShareCalculationOverflow)?;
        keys.push(share.address);
    }
    keys.sort_unstable();
    keys.dedup();
    require!(sum == 10_000, FeeError::InvalidShareTotal);
    require!(keys.len() == shares.len(), FeeError::DuplicateShareholder);
    Ok(())
}
