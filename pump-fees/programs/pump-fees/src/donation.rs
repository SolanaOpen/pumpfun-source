//! Partial donation behavior integrated from the independently validated local draft.
//! Captured relay code executes only as an external dependency in local tests.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_spl::token_interface::Mint;
use crate::{FeeError, ID};
use crate::types::FeeProgramGlobal;
pub const RELAY: Pubkey = anchor_lang::solana_program::pubkey!("RLAYHr9TRFcKB2ubYQhspcnXiaGpaVzNQvHytt47RZu");
pub const PUMP: Pubkey = crate::pump_global::PUMP_ID;
pub const AMM: Pubkey = anchor_lang::solana_program::pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub fn crank(ctx: Context<CrankDonationFeePda>) -> Result<()> {
        require!(ctx.accounts.fee_program_global.disable_flags & 8 == 0, FeeError::FeatureDeactivated);
        let donation=ctx.accounts.donation_fee_pda.to_account_info();
        let excess=donation.lamports().saturating_sub(Rent::get()?.minimum_balance(218));
        let mint=ctx.accounts.quote_mint.key();
        let base=ctx.accounts.donation_fee_pda.base_mint;
        let config=ctx.accounts.donation_fee_pda.config_id;
        let bump=[ctx.accounts.donation_fee_pda.bump];
        let seeds:&[&[u8]]=&[b"donation-fee-pda",base.as_ref(),config.as_ref(),&bump];
        if mint==anchor_spl::token::spl_token::native_mint::ID && excess>0 {
            **donation.try_borrow_mut_lamports()? -= excess;
            **ctx.accounts.donation_fee_pda_ata.to_account_info().try_borrow_mut_lamports()? += excess;
            let ix=anchor_lang::solana_program::instruction::Instruction {program_id:anchor_spl::token::ID,
                accounts:vec![AccountMeta::new(ctx.accounts.donation_fee_pda_ata.key(),false),AccountMeta::new_readonly(ctx.accounts.rent.key(),false),AccountMeta::new_readonly(donation.key(),true)],data:vec![17]};
            anchor_lang::solana_program::program::invoke_signed(&ix,&[ctx.accounts.donation_fee_pda_ata.to_account_info(),ctx.accounts.rent.to_account_info(),donation.clone()],&[seeds])?;
            ctx.accounts.donation_fee_pda_ata.reload()?;
        }
        let amount=ctx.accounts.donation_fee_pda_ata.amount;
        if amount==0{return Ok(());}
        let message=format!("{},{}",base,ctx.accounts.donation_fee_pda.creator);
        let mut payload=vec![120,217,57,241,135,104,139,184];
        payload.extend_from_slice(&amount.to_le_bytes());payload.extend_from_slice(config.as_ref());
        payload.extend_from_slice(&0u16.to_le_bytes());message.serialize(&mut payload)?;payload.extend_from_slice(ID.as_ref());
        let infos=[ctx.accounts.epoch_tracker.to_account_info(),ctx.accounts.debouncer.to_account_info(),ctx.accounts.debouncer_ata.to_account_info(),ctx.accounts.quote_mint.to_account_info(),ctx.accounts.donation_fee_pda_ata.to_account_info(),donation.clone(),ctx.accounts.payer.to_account_info(),ctx.accounts.mint_whitelist.to_account_info(),ctx.accounts.associated_token_program.to_account_info(),ctx.accounts.token_program.to_account_info(),ctx.accounts.system_program.to_account_info(),ctx.accounts.donation_relay_event_authority.to_account_info(),ctx.accounts.donation_relay_program.to_account_info()];
        let metas=infos.iter().enumerate().map(|(i,a)| if i<7 {AccountMeta::new(a.key(),i==5||i==6)} else {AccountMeta::new_readonly(a.key(),false)}).collect();
        let ix=anchor_lang::solana_program::instruction::Instruction {program_id:RELAY,accounts:metas,data:payload};
        anchor_lang::solana_program::program::invoke_signed(&ix,&infos,&[seeds])?;
        let state=&mut ctx.accounts.donation_fee_pda;
        state.total_donated=state.total_donated.checked_add(amount).ok_or(error!(FeeError::ShareCalculationOverflow))?;
        state.last_crank_ts=Clock::get()?.unix_timestamp;
        let event=DonationFeePdaCranked {timestamp:state.last_crank_ts,signer:ctx.accounts.payer.key(),donation_fee_pda:state.key(),config_id:config,base_mint:base,quote_mint:mint,creator:state.creator,amount};
        emit!(event);emit_cpi!(event);Ok(())
    }
pub fn create(ctx: Context<CreateDonationFeePda>) -> Result<()> {
        if ctx.accounts.fee_program_global.disable_flags & 4 != 0 {
            msg!("CreateDonationFeePda is currently disabled");
            return Err(FeeError::FeatureDeactivated.into());
        }
        if ctx.accounts.donation_fee_pda.version != 0 {
            msg!("donation_fee_pda already exists");
            return Ok(());
        }
        let curve = &ctx.accounts.bonding_curve;
        let (creator, quote_mint) = if curve.real_sol_reserves != 0
            || curve.virtual_sol_reserves != 0
            || curve.real_token_reserves != 0
            || curve.virtual_token_reserves != 0
        {
            let quote = donation_raw_curve_quote(curve.to_account_info());
            let quote = if quote == Pubkey::default() {
                anchor_spl::token::spl_token::native_mint::ID
            } else {
                quote
            };
            (curve.creator, quote)
        } else {
            let info = &ctx.accounts.pool;
            require_keys_eq!(*info.owner, AMM, FeeError::InvalidPool);
            let data=info.try_borrow_data()?;
            let pool=Pool::try_deserialize(&mut &data[..])?;
            require_keys_eq!(pool.base_mint, ctx.accounts.base_mint.key(), FeeError::InvalidPool);
            let authority=Pubkey::find_program_address(&[b"pool-authority",ctx.accounts.base_mint.key().as_ref()],&PUMP).0;
            require_keys_eq!(pool.creator,authority,FeeError::InvalidPool);
            let expected=Pubkey::create_program_address(&[b"pool",&pool.index.to_le_bytes(),pool.creator.as_ref(),pool.base_mint.as_ref(),pool.quote_mint.as_ref(),&[pool.pool_bump]],&AMM).map_err(|_| FeeError::InvalidPool)?;
            require_keys_eq!(info.key(),expected,FeeError::InvalidPool);
            (pool.coin_creator, pool.quote_mint)
        };
        let creator = if creator == ctx.accounts.sharing_config.key() {
            let sharing = &ctx.accounts.sharing_config;
            require_keys_eq!(*sharing.owner,ID,FeeError::InvalidSharingConfig);
            require!(!sharing.data_is_empty(),FeeError::InvalidSharingConfig);
            let bytes=sharing.try_borrow_data()?;
            let config=SharingConfig::try_deserialize(&mut &bytes[..])?;
            require_keys_eq!(config.mint,ctx.accounts.base_mint.key(),FeeError::InvalidSharingConfig);
            require!(config.admin == ctx.accounts.payer.key(), FeeError::NotAuthorized);
            if config.admin_revoked {
                return Err(FeeError::SharingConfigAdminRevoked.into());
            }
            require!(config.status==ConfigStatus::Active,FeeError::SharingConfigNotActive);
            config.admin
        } else {
            require_keys_eq!(ctx.accounts.payer.key(),creator,FeeError::NotAuthorized);
            creator
        };
        let pda=&mut ctx.accounts.donation_fee_pda;
        pda.bump=ctx.bumps.donation_fee_pda;
        pda.version=1;
        pda.config_id=ctx.accounts.config_id.key();
        pda.base_mint=ctx.accounts.base_mint.key();
        pda.quote_mint=quote_mint;
        pda.creator=creator;
        pda.total_donated=0;pda.last_crank_ts=0;pda.reserved=[0;64];
        let event=DonationFeePdaCreated { timestamp:Clock::get()?.unix_timestamp, created_by:ctx.accounts.payer.key(),donation_fee_pda:pda.key(),config_id:pda.config_id,base_mint:pda.base_mint,quote_mint:pda.quote_mint,creator:pda.creator };
        emit!(event);emit_cpi!(event);Ok(())
    }
fn donation_raw_curve_quote(info: AccountInfo<'_>) -> Pubkey {
    let mut quote_bytes = [0u8; 32];
    if info.data_len() >= 115 {
        let data = info.data.borrow();
        quote_bytes.copy_from_slice(&data[83..115]);
    }
    Pubkey::new_from_array(quote_bytes)
}

#[derive(Accounts)]
pub struct CreateDonationFeePda<'info> {
    /// CHECK: event PDA validated.
    #[account(seeds=[b"__event_authority"],bump)] pub event_authority: AccountInfo<'info>,
    pub program: Program<'info, crate::program::PumpFeesCandidate>,
    #[account(mut)] pub payer: Signer<'info>,
    pub system_program: Program<'info,System>,
    #[account(seeds=[b"fee-program-global"],bump=fee_program_global.bump)] pub fee_program_global: Account<'info,FeeProgramGlobal>,
    #[account(init_if_needed,payer=payer,space=218,seeds=[b"donation-fee-pda",base_mint.key().as_ref(),config_id.key().as_ref()],bump)] pub donation_fee_pda: Account<'info,DonationFeePda>,
    /// CHECK: identifier only.
    pub config_id: UncheckedAccount<'info>,
    pub base_mint: InterfaceAccount<'info,Mint>,
    #[account(seeds=[b"bonding-curve",base_mint.key().as_ref()],bump,seeds::program=PUMP)] pub bonding_curve: Account<'info,BondingCurve>,
    /// CHECK: validated in handler.
    pub pool: UncheckedAccount<'info>,
    /// CHECK: derived identifier; state rules under recovery.
    #[account(seeds=[b"sharing-config",base_mint.key().as_ref()],bump)] pub sharing_config: UncheckedAccount<'info>,
}
pub use crate::state::DonationFeePda;
pub use crate::pump::accounts::BondingCurve;
pub use crate::pump_amm::accounts::Pool;
#[event]
pub struct DonationFeePdaCreated {pub timestamp:i64,pub created_by:Pubkey,pub donation_fee_pda:Pubkey,pub config_id:Pubkey,pub base_mint:Pubkey,pub quote_mint:Pubkey,pub creator:Pubkey}

#[derive(AnchorSerialize,AnchorDeserialize,Clone,PartialEq)]
pub enum ConfigStatus {Paused,Active}
pub use crate::instructions::fee_sharing::create_fee_sharing_config::SharingConfig;

#[derive(Accounts)]
pub struct CrankDonationFeePda<'info> {
    /// CHECK: event PDA.
    #[account(seeds=[b"__event_authority"],bump)] pub event_authority:UncheckedAccount<'info>,
    pub program:Program<'info,crate::program::PumpFeesCandidate>,
    #[account(mut)]pub payer:Signer<'info>,
    pub system_program:Program<'info,System>,
    pub token_program:Program<'info,anchor_spl::token::Token>,
    pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
    /// CHECK: rent address.
    #[account(address=anchor_lang::solana_program::sysvar::rent::ID)]pub rent:UncheckedAccount<'info>,
    #[account(seeds=[b"fee-program-global"],bump=fee_program_global.bump)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
    #[account(mut,seeds=[b"donation-fee-pda",donation_fee_pda.base_mint.as_ref(),donation_fee_pda.config_id.as_ref()],bump=donation_fee_pda.bump)]pub donation_fee_pda:Account<'info,DonationFeePda>,
    #[account(mut,address=donation_fee_pda.quote_mint @ anchor_lang::error::ErrorCode::ConstraintMintTokenProgram)]pub quote_mint:Account<'info,anchor_spl::token::Mint>,
    #[account(init_if_needed,payer=payer,associated_token::mint=quote_mint,associated_token::authority=donation_fee_pda)]pub donation_fee_pda_ata:Account<'info,anchor_spl::token::TokenAccount>,
    /// CHECK: fixed executable dependency.
    #[account(executable,address=RELAY)]pub donation_relay_program:UncheckedAccount<'info>,
    /// CHECK: dependency enforces its constraints.
    pub donation_relay_event_authority:UncheckedAccount<'info>,
    /// CHECK: dependency enforces its constraints.
    pub mint_whitelist:UncheckedAccount<'info>,
    /// CHECK: dependency enforces its constraints.
    #[account(mut)]pub epoch_tracker:UncheckedAccount<'info>,
    /// CHECK: dependency enforces its constraints.
    #[account(mut)]pub debouncer:UncheckedAccount<'info>,
    /// CHECK: dependency enforces its constraints.
    #[account(mut)]pub debouncer_ata:UncheckedAccount<'info>,
}
#[event]
pub struct DonationFeePdaCranked {pub timestamp:i64,pub signer:Pubkey,pub donation_fee_pda:Pubkey,pub config_id:Pubkey,pub base_mint:Pubkey,pub quote_mint:Pubkey,pub creator:Pubkey,pub amount:u64}
