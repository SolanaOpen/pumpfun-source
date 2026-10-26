use anchor_lang::prelude::*;
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_quote_reserves: u64,
    pub real_token_reserves: u64,
    pub real_quote_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
    pub quote_mint: Pubkey,
    pub creator_fee_bps: u64,
    pub can_edit_creator_fee: bool,
    pub is_holder_reward: bool,
    pub creator_fee: u64,
    pub protocol_fees: u64,
    pub depth: u8,
    pub initial_virtual_quote_reserves: u64,
    pub post_complete_base_out: u64,
    pub post_complete_quote_in: u64,
}
impl anchor_lang::Discriminator for BondingCurve {
    const DISCRIMINATOR: &'static [u8] = &[23, 183, 248, 55, 96, 216, 172, 96];
}
impl anchor_lang::Owner for BondingCurve {
    fn owner() -> Pubkey { crate::ID }
}
impl anchor_lang::AccountSerialize for BondingCurve {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(Self::DISCRIMINATOR)?;
        AnchorSerialize::serialize(self, writer)?;
        Ok(())
    }
}
// Source proposal only: original checked 0x9b148 and unchecked 0x9b430.
// Account owner/lamports/borrow checks and field-name errors belong to Anchor.
impl anchor_lang::AccountDeserialize for BondingCurve {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < Self::DISCRIMINATOR.len()
            || &buf[..Self::DISCRIMINATOR.len()] != Self::DISCRIMINATOR
        {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        Self::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < Self::DISCRIMINATOR.len() {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        *buf = &buf[Self::DISCRIMINATOR.len()..];
        decode_original_curve_payload(buf)
    }
}

fn historical_field<T: AnchorDeserialize>(buf: &mut &[u8]) -> Result<T> {
    // Original maps Borsh I/O errors through Anchor's From<io::Error> path,
    // not AccountDidNotDeserialize / a custom numeric error.
    T::deserialize(buf).map_err(anchor_lang::error::Error::from)
}

fn decode_original_curve_payload(buf: &mut &[u8]) -> Result<BondingCurve> {
    let historical_len = buf.len();
    if historical_len < 41 {
        return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
    }
    let mut curve = BondingCurve {
        virtual_token_reserves: historical_field(buf)?,
        virtual_quote_reserves: historical_field(buf)?,
        real_token_reserves: historical_field(buf)?,
        real_quote_reserves: historical_field(buf)?,
        token_total_supply: historical_field(buf)?,
        complete: historical_field(buf)?,
        creator: Pubkey::default(),
        is_mayhem_mode: false,
        is_cashback_coin: false,
        quote_mint: Pubkey::default(),
        creator_fee_bps: 0,
        can_edit_creator_fee: false,
        is_holder_reward: false,
        creator_fee: 0,
        protocol_fees: 0,
        depth: 0,
        initial_virtual_quote_reserves: 0,
        post_complete_base_out: 0,
        post_complete_quote_in: 0,
    };
    // These gates are layout-version boundaries, not per-field best efforts.
    // Bytes short of the next complete version remain unconsumed.
    if historical_len >= 73 { curve.creator = historical_field(buf)?; }
    if historical_len >= 74 { curve.is_mayhem_mode = historical_field(buf)?; }
    if historical_len >= 75 { curve.is_cashback_coin = historical_field(buf)?; }
    if historical_len >= 107 { curve.quote_mint = historical_field(buf)?; }
    if historical_len >= 116 {
        curve.creator_fee_bps = historical_field(buf)?;
        curve.can_edit_creator_fee = historical_field(buf)?;
    }
    if historical_len >= 117 { curve.is_holder_reward = historical_field(buf)?; }
    if historical_len >= 133 {
        curve.creator_fee = historical_field(buf)?;
        curve.protocol_fees = historical_field(buf)?;
    }
    if historical_len >= 158 {
        curve.depth = historical_field(buf)?;
        curve.initial_virtual_quote_reserves = historical_field(buf)?;
        curve.post_complete_base_out = historical_field(buf)?;
        curve.post_complete_quote_in = historical_field(buf)?;
    }
    Ok(curve)
}
// Original shared preparation 0x3cd50 and rent funding 0x3d250.
// The Account is already decoded; temporary AccountInfo clones do not keep a
// data borrow alive across the System CPI or realloc. No state reload follows.
pub fn prepare_bonding_curve<'info>(
    curve: &Account<'info, BondingCurve>,
    system_program: &Program<'info, System>,
    payer: &Signer<'info>,
) -> Result<()> {
    let current_size = curve.to_account_info().data_len();
    fund_bonding_curve_rent(curve, system_program, payer, current_size.max(166))?;
    if current_size < 166 {
        curve.to_account_info().realloc(166, false)?;
    }
    Ok(())
}

fn reserved_bonding_curve_fees(curve: &BondingCurve) -> Result<u64> {
    curve.creator_fee.checked_add(curve.protocol_fees)
        .ok_or_else(|| error!(crate::PumpError::Overflow))
}

fn fund_bonding_curve_rent<'info>(
    curve: &Account<'info, BondingCurve>,
    system_program: &Program<'info, System>,
    payer: &Signer<'info>,
    size: usize,
) -> Result<()> {
    let reserved = if curve.quote_mint == Pubkey::default() {
        let fees = reserved_bonding_curve_fees(curve)?;
        curve.real_quote_reserves.checked_add(fees)
            .ok_or_else(|| error!(crate::PumpError::Overflow))?
    } else {
        0
    };
    let minimum = Rent::get()?.minimum_balance(size);
    let available = curve.to_account_info().lamports().saturating_sub(reserved);
    if minimum > available {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(),
                    to: curve.to_account_info(),
                },
            ),
            minimum.saturating_sub(available),
        )?;
    }
    Ok(())
}

#[event]
#[derive(Clone)]
pub struct TradeEvent {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u64,
    pub fee: u64,
    pub creator: Pubkey,
    pub creator_fee_basis_points: u64,
    pub creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    pub ix_name: String,
    pub mayhem_mode: bool,
    pub cashback_fee_basis_points: u64,
    pub cashback: u64,
    pub buyback_fee_basis_points: u64,
    pub buyback_fee: u64,
    pub shareholders: Vec<Shareholder>,
    pub quote_mint: Pubkey,
    pub quote_amount: u64,
    pub virtual_quote_reserves: u64,
    pub real_quote_reserves: u64,
    pub holder_rewards_bps: u64,
    pub holder_rewards: u64,
    pub creator_fee_unclaimed: u64,
    pub unparsed_trailing_field: u64,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Shareholder {
    pub address: Pubkey,
    pub share_bps: u16,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeeConfig {
    pub bump: u8,
    pub admin: Pubkey,
    pub flat_fees: Fees,
    pub fee_tiers: Vec<FeeTier>,
    pub stable_fee_tiers: Vec<FeeTier>,
    pub exotic_flat_fees: Fees,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeeTier {
    pub market_cap_lamports_threshold: u128,
    pub fees: Fees,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Fees {
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub creator_fee_bps: u64,
}

#[event]
#[derive(Clone)]
pub struct CompleteEvent {pub user:Pubkey,pub mint:Pubkey,pub bonding_curve:Pubkey,pub timestamp:i64,pub quote_mint:Pubkey}
