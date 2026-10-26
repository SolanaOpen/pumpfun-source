pub mod create_fee_sharing_config;

use anchor_lang::prelude::*;
use crate::FeeError;

// Reconstructed name; the original routine's historical identifier is unknown.
pub(crate) fn admit_associated_token_account<'info>(
    associated_account: &AccountInfo<'info>,
    wallet: &AccountInfo<'info>,
    mint: &Pubkey,
    token_program: &Pubkey,
    mint_info: &AccountInfo<'info>,
    token_program_info: &AccountInfo<'info>,
    create_if_missing: bool,
    payer: &Signer<'info>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    let expected = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        wallet.key, mint, token_program,
    );
    require_keys_eq!(*associated_account.key, expected, anchor_lang::error::ErrorCode::ConstraintSeeds);
    if associated_account.owner == &anchor_lang::system_program::ID {
        require!(create_if_missing, anchor_lang::error::ErrorCode::AccountNotInitialized);
        let instruction = anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            payer.key, wallet.key, mint, token_program,
        );
        anchor_lang::solana_program::program::invoke(
            &instruction,
            &[
                payer.to_account_info(),
                associated_account.clone(),
                wallet.clone(),
                mint_info.clone(),
                system_program.to_account_info(),
                token_program_info.clone(),
            ],
        )?;
    }
    require_keys_eq!(*associated_account.owner, *token_program, FeeError::AccountTypeNotSupported);
    let data = associated_account.try_borrow_data()?;
    let account = anchor_spl::token_interface::TokenAccount::try_deserialize(&mut &data[..])?;
    require_keys_eq!(account.mint, *mint, FeeError::InvalidMint);
    Ok(())
}
