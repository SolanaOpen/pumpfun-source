//! Partial independently compiled Mayhem implementation: 24 recovered administration, sweep,
//! burn and recovered trade handlers. Unknown fields are preserved as byte arrays, not guessed names.
//! This is not a complete or byte-identical program reconstruction.
use anchor_lang::prelude::*;
mod checked_program;
pub mod state;
pub mod instructions;
pub use state::GlobalParams;
pub use instructions::fee_recipients::*;
pub use instructions::admin::*;
pub use instructions::extend_account::*;
pub use instructions::burn::*;
pub use instructions::sell::*;
pub use instructions::buy::*;
pub use instructions::sell_v2::*;
pub use instructions::buy_v2::*;
pub use instructions::buy_pump_swap::*;
pub use instructions::sell_pump_swap::*;
declare_program!(pump);
declare_program!(pump_amm);

declare_id!("MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e");

#[program]
pub mod mayhem_program {
    use super::*;
    pub fn buy_pump_swap(ctx: Context<BuyPumpSwap>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::buy_pump_swap::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn sell_pump_swap(ctx: Context<SellPumpSwap>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::sell_pump_swap::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn buy_v2(ctx: Context<BuyV2>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::buy_v2::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn sell_v2(ctx: Context<SellV2>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::sell_v2::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn buy(ctx: Context<LegacyBuy>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::buy::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn sell(ctx: Context<LegacySell>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> {
        instructions::sell::handler(ctx, reference, tolerance_bps, retained_reserve_bps)
    }
    pub fn burn_tokens(ctx: Context<BurnTokens>) -> Result<()> {
        instructions::burn::handler(ctx)
    }
    pub fn sweep_fees<'info>(ctx: Context<'info, SweepFees<'info>>) -> Result<()> { instructions::fee_recipients::sweep_fees(ctx) }
    pub fn sweep_fees_admin<'info>(ctx: Context<'info, SweepFeesAdmin<'info>>) -> Result<()> { instructions::fee_recipients::sweep_fees_admin(ctx) }
    pub fn withdraw_quote_admin(ctx: Context<WithdrawQuoteAdmin>, amount: u64) -> Result<()> { instructions::admin::withdraw_quote_admin(ctx, amount) }
    pub fn withdraw_admin(ctx: Context<WithdrawAdmin>, amount: u64) -> Result<()> { instructions::admin::withdraw_admin(ctx, amount) }
    pub fn extend_account(ctx: Context<ExtendAccount>) -> Result<()> { instructions::extend_account::extend_account(ctx) }
    pub fn initialize_fee_recipient(ctx: Context<InitializeFeeRecipient>, index: u8) -> Result<()> { super::initialize_fee_recipient_impl(ctx, index) }
    pub fn initialize_global_params(ctx: Context<InitializeGlobalParams>) -> Result<()> { instructions::admin::initialize_global_params(ctx) }
    pub fn update_global_params(ctx: Context<Admin>, args: UpdateGlobalParamsArgs) -> Result<()> { instructions::admin::update_global_params(ctx, args.field_0, args.field_1, args.field_2, args.field_3, args.field_4, args.field_5, args.field_6, args.field_7, args.field_8, args.field_9, args.field_10, args.field_11, args.field_12, args.field_13, args.field_14, args.field_15) }
    pub fn update_admin(ctx: Context<Admin>, new_admin: Pubkey) -> Result<()> { instructions::admin::update_admin(ctx, new_admin) }
    pub fn update_trader(ctx: Context<Admin>, new_trader: Pubkey) -> Result<()> { instructions::admin::update_trader(ctx, new_trader) }
    pub fn update_max_sol_per_sweep(ctx: Context<Admin>, value: u64) -> Result<()> { instructions::admin::update_max_sol_per_sweep(ctx, value) }
    pub fn update_max_stable_per_sweep(ctx: Context<Admin>, value: u64) -> Result<()> { instructions::admin::update_max_stable_per_sweep(ctx, value) }
    pub fn reset_sol_deployed(ctx: Context<Admin>) -> Result<()> { instructions::admin::reset_sol_deployed(ctx) }
    pub fn reset_stable_deployed(ctx: Context<Admin>) -> Result<()> { instructions::admin::reset_stable_deployed(ctx) }
    pub fn reset_state(ctx: Context<Admin>) -> Result<()> { instructions::admin::reset_state(ctx) }
    pub fn stop_trading(ctx: Context<Admin>) -> Result<()> { instructions::admin::stop_trading(ctx) }
    pub fn resume_trading(ctx: Context<Admin>) -> Result<()> { instructions::admin::resume_trading(ctx) }
}

#[error_code]
pub enum MayhemError {
    #[msg("Trading is paused")]
    TradingPaused,
    #[msg("Trading is active")]
    TradingActive,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Withdraw Rate limit")]
    WithdrawRateLimit = 14,
    #[msg("Account type not supported")]
    AccountTypeNotSupported = 16,
    #[msg("Only 8 fee recipients supported")]
    MaxFeeRecipientsExceeded = 17,
}

#[inline(never)]
fn initialize_fee_recipient_impl(ctx: Context<InitializeFeeRecipient>, index: u8) -> Result<()> {
    if index >= 8 {
        return Err(anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
            error_name: MayhemError::MaxFeeRecipientsExceeded.name(),
            error_code_number: MayhemError::MaxFeeRecipientsExceeded.into(),
            error_msg: MayhemError::MaxFeeRecipientsExceeded.to_string(),
            error_origin: Some(anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                filename: "programs/mayhem-program/src/instructions/fee_recipients.rs",
                line: 186,
            })),
            compared_values: None,
        })
        .with_values((8, index)));
    }
    instructions::fee_recipients::initialize_fee_recipient(core::hint::black_box(ctx), index)
}
