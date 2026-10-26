//! IDL argument types needed for instruction deserialization.
use anchor_lang::prelude::*;

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

#[derive(AnchorSerialize, Clone)]
pub struct OptionBool(pub bool);

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Shareholder {
    pub address: Pubkey,
    pub share_bps: u16,
}

impl AnchorDeserialize for OptionBool {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut flag = [0u8];
        match reader.read(&mut flag)? {
            0 => Ok(Self(false)),
            _ if flag[0] <= 1 => Ok(Self(flag[0] == 1)),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid bool")),
        }
    }
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
impl Owner for FeeConfig { fn owner() -> Pubkey { crate::ID } }
impl anchor_lang::Discriminator for FeeConfig {
    const DISCRIMINATOR: &'static [u8] = &[143, 52, 146, 187, 219, 123, 76, 155];
}
impl AccountSerialize for FeeConfig {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        use anchor_lang::Discriminator;
        writer.write_all(&Self::DISCRIMINATOR)?;
        self.serialize(writer)?;
        Ok(())
    }
}
impl AccountDeserialize for FeeConfig {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        use anchor_lang::Discriminator;
        if buf.len() < 8 || &buf[..8] != Self::DISCRIMINATOR { return Err(ProgramError::InvalidAccountData.into()); }
        Self::try_deserialize_unchecked(buf)
    }
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 2512 { return Err(ProgramError::InvalidAccountData.into()); }
        let mut data = &buf[8..];
        let decode = (|| -> std::io::Result<Self> {
            let bump = u8::deserialize(&mut data)?;
            let admin = Pubkey::deserialize(&mut data)?;
            let flat_fees = Fees::deserialize(&mut data)?;
            let fee_tiers = Vec::<FeeTier>::deserialize(&mut data)?;
            let stable_fee_tiers = if buf.len() >= 4073 { Vec::<FeeTier>::deserialize(&mut data)? } else { vec![] };
            let exotic_flat_fees = if buf.len() >= 4097 { Fees::deserialize(&mut data)? } else { Fees { lp_fee_bps: 0, protocol_fee_bps: 0, creator_fee_bps: 0 } };
            Ok(Self { bump, admin, flat_fees, fee_tiers, stable_fee_tiers, exotic_flat_fees })
        })();
        decode.map_err(|e| ProgramError::BorshIoError(e.to_string()).into())
    }
}

pub use crate::state::FeeProgramGlobal;
#[event]
pub struct SetAuthorityEvent { pub timestamp: i64, pub old_authority: Pubkey, pub new_authority: Pubkey }
#[event]
pub struct SetSocialClaimAuthorityEvent { pub timestamp: i64, pub social_claim_authority: Pubkey }
#[event]
pub struct SetDisableFlagsEvent { pub timestamp: i64, pub disable_flags: u8 }
#[event]
pub struct SetClaimRateLimitEvent { pub timestamp: i64, pub claim_rate_limit: u64 }

#[event]
pub struct InitializeFeeProgramGlobalEvent {
    pub timestamp: i64, pub authority: Pubkey, pub social_claim_authority: Pubkey,
    pub disable_flags: u8, pub claim_rate_limit: u64,
}

#[event]
#[derive(Clone)]
pub struct SetExoticFlatFeesEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey, pub exotic_flat_fees: Fees,
}
#[event]
#[derive(Clone)]
pub struct UpdateFeeConfigEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub fee_tiers: Vec<FeeTier>, pub flat_fees: Fees,
}
#[event]
#[derive(Clone)]
pub struct UpdateStableFeeConfigEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub stable_fee_tiers: Vec<FeeTier>, pub flat_fees: Fees,
}

#[event]
pub struct ExtendFeeConfigEvent {
    pub fee_config: Pubkey, pub user: Pubkey, pub current_size: u64,
    pub new_size: u64, pub timestamp: i64,
}

#[event]
#[derive(Clone)]
pub struct UpsertFeeTiersEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub fee_tiers: Vec<FeeTier>, pub offset: u8,
}
#[event]
#[derive(Clone)]
pub struct UpsertStableFeeTiersEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub stable_fee_tiers: Vec<FeeTier>, pub offset: u8,
}

#[event]
pub struct UpdateAdminEvent { pub timestamp: i64, pub old_admin: Pubkey, pub new_admin: Pubkey }

#[event]
pub struct InitializeFeeConfigEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub fee_config: Pubkey,
}
