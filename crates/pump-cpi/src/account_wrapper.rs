//! Reads accounts that another program may still hold in an older layout.
//!
//! Pump and PumpSwap only ever append fields to their accounts (`BondingCurve`,
//! `Pool`). An account created before an upgrade keeps its old, shorter size
//! until the owning program extends it, so deserializing it with the types
//! `declare_program!` generates from the current IDL runs out of bytes before
//! the last fields and fails with `AccountDidNotDeserialize`.
//!
//! [`AccountWrapper`] zero-pads such data to the size of the current layout
//! before deserializing it. A field the account does not have yet reads as its
//! zero value (`0`, `false`, `Pubkey::default()`), which is also what the owning
//! program assumes when it extends the account. Data that is already at least
//! as long as the current layout is deserialized as it is.
//!
//! ```ignore
//! use pump_cpi::account_wrapper::AccountWrapper;
//! use pump_cpi::pump_v2::pump::accounts::BondingCurve;
//!
//! #[account(mut)]
//! pub bonding_curve: Box<Account<'info, AccountWrapper<BondingCurve>>>,
//! ```
//!
//! The discriminator is still checked by the wrapped type, and the owner is the
//! wrapped type's program. Anchor writes an `Account` back on exit only when the
//! executing program owns it, so a wrapped account is never written.
use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::*;

/// An account of another program, deserialized with zero padding when its data
/// is shorter than the current layout of `T`.
#[derive(Clone, Debug)]
pub struct AccountWrapper<T>(T);

impl<T> AccountWrapper<T> {
    /// Wraps an account that is already deserialized.
    pub fn new(account: T) -> Self {
        Self(account)
    }

    /// Returns the wrapped account.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for AccountWrapper<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for AccountWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Owner> Owner for AccountWrapper<T> {
    fn owner() -> Pubkey {
        T::owner()
    }
}

impl<T: Discriminator> Discriminator for AccountWrapper<T> {
    const DISCRIMINATOR: &'static [u8] = T::DISCRIMINATOR;
}

impl<T: AccountSerialize> AccountSerialize for AccountWrapper<T> {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        self.0.try_serialize(writer)
    }
}

impl<T> AccountDeserialize for AccountWrapper<T>
where
    T: AccountSerialize + AccountDeserialize + Default,
{
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        // Size of the current layout: the discriminator and every field.
        let mut current = Vec::new();
        T::default().try_serialize(&mut current)?;
        let len = current.len();
        if buf.len() >= len {
            return T::try_deserialize(buf).map(Self);
        }

        let mut padded = vec![0u8; len];
        padded[..buf.len()].copy_from_slice(buf);
        let mut data: &[u8] = &padded;
        let account = T::try_deserialize(&mut data)?;
        *buf = &buf[buf.len()..];
        Ok(Self(account))
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        T::try_deserialize_unchecked(buf).map(Self)
    }
}
