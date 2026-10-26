//! Reads accounts written by an older program version: data shorter than the
//! current layout is zero-padded before it is deserialized. Reconstruction:
//! the original binary carries panic locations in `src/account_wrapper.rs` at
//! 90:26 and 91:15, inside its PumpSwap Pool reader.
use anchor_lang::prelude::*;

/// Deserialize `T` from `data`, zero-padding it to `len` bytes first.
pub fn try_deserialize_padded<T: AccountDeserialize>(data: &[u8], len: usize) -> Result<T> {
    let mut padded = vec![0u8; len.max(data.len())];
    padded[..data.len()].copy_from_slice(data);
    T::try_deserialize(&mut &padded[..])
}
