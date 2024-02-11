// Import necessary Solana Program modules and dependencies
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

// Custom error enum for BARK program
#[derive(Debug, Clone, Copy)]
pub enum CustomError {
    InvalidInstructionData,
    OtherProgramError(u32),
}

// Utility function to create a new account securely
pub fn create_new_account<'a>(
    payer: &AccountInfo<'a>,
    space: u64,
    lamports: u64,
    program_id: &Pubkey,
) -> Result<AccountInfo<'a>, ProgramError> {
    let account = AccountInfo::new(next_account_info(payer)?, false, true, program_id);
    let create_account_ix = system_instruction::create_account(
        payer.key,
        account.key,
        lamports,
        space,
        program_id,
    );
    solana_program::program::invoke(
        &create_account_ix,
        &[payer.clone(), account.clone()],
        &[],
    )?;
    Ok(account)
}

// Utility function to create an associated BARK token account securely
pub fn create_associated_token_account<'a>(
    payer: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> Result<AccountInfo<'a>, ProgramError> {
    // Placeholder logic for creating associated BARK token accounts securely
    // Replace the logic below with BARK implementation
    let associated_token_account =
        AccountInfo::new(next_account_info(payer)?, false, true, program_id);
    // ... (your associated BARK token account creation logic securely)
    Ok(associated_token_account)
}

// Utility function to burn BARK tokens based on a percentage
pub fn burn_tokens(
    mint_account: &AccountInfo,
    burn_amount: u64,
) -> Result<(), ProgramError> {
    // Placeholder logic for burning BARK tokens securely
    // Replace the logic below with BARK actual implementation
    let burn_address = Pubkey::new_unique();
    let burn_ix = system_instruction::transfer(mint_account.key, &burn_address, burn_amount);
    solana_program::program::invoke(&burn_ix, &[mint_account.clone()], &[])?;
    Ok(())
}

// Utility function to calculate transaction fee securely
pub fn calculate_transaction_fee(amount: u64) -> Result<u64, CustomError> {
    let fee = ((amount as u128 * BURN_PERCENTAGE as u128) / 100) as u64;
    if fee > MAX_FEE_BASIS_POINTS as u64 {
        Err(CustomError::InvalidInstructionData)
    } else {
        Ok(fee)
    }
}

// ... (other utility functions)
