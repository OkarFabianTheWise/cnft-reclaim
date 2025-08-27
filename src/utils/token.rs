use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
};

pub fn get_token_account_amount(token_account: &AccountInfo) -> Result<u64, ProgramError> {
    // SPL Token account data is 165 bytes, amount is at offset 64
    if token_account.data.borrow().len() < 72 {
        return Err(ProgramError::InvalidAccountData);
    }

    let data = token_account.data.borrow();
    let amount_bytes: [u8; 8] = data[64..72]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    Ok(u64::from_le_bytes(amount_bytes))
}
