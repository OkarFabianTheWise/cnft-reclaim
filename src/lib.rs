pub mod instruction;
mod state;
mod utils;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use instruction::TwapInstruction;
use state::{TwapStorage, PriceObservation};
use utils::token::get_token_account_amount;
use utils::twap::calculate_twap;

// === TWAP Observation Parameters ===
pub const OBSERVATION_COUNT: usize = 3; // Number of price observations to store
pub const OBSERVATION_INTERVAL: i64 = 100; // Minimum seconds between observations (1 minute and 40 seconds)
pub const RAYDIUM_AMM_PROGRAM_ID: Pubkey = solana_program::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TwapInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TwapInstruction::InitializeTwapStorage => {
            process_initialize_twap_storage(program_id, accounts)
        }
        TwapInstruction::UpdatePriceObservation => {
            process_update_price_observation(program_id, accounts)
        }
        TwapInstruction::GetTwapPrice { window_minutes } => {
            process_get_twap_price(program_id, accounts, window_minutes)
        }
        TwapInstruction::Reclaim => {
            process_reclaim(program_id, accounts)
        }
    }
}

// Burns token fragments from the caller and logs info
fn process_reclaim(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    use solana_program::program_pack::Pack;
    let account_info_iter = &mut accounts.iter();
    let user = next_account_info(account_info_iter)?;
    let user_token_x_account = next_account_info(account_info_iter)?;
    let token_x_mint = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    // Check user is signer
    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Get user's full token_x balance
    let user_token_x_amount = get_token_account_amount(user_token_x_account)?;
    if user_token_x_amount == 0 {
        msg!("User does not hold any token_x");
        return Err(ProgramError::InsufficientFunds);
    }

    // Burn full token_x balance from user's account
    let ix = spl_token::instruction::burn(
        token_program.key,
        user_token_x_account.key,
        token_x_mint.key,
        user.key,
        &[],
        user_token_x_amount,
    )?;
    invoke_signed(
        &ix,
        &[user_token_x_account.clone(), token_x_mint.clone(), user.clone(), token_program.clone()],
        &[],
    )?;

    // Log info
    msg!("Reclaim: user {} burnt {} token_x (decimals: 6)", user.key, user_token_x_amount / 1_000_000);
    Ok(())
}

// Initialize the twap_storage pda if it doesn't exist
fn process_initialize_twap_storage(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let twap_storage_account = next_account_info(account_info_iter)?;
    let token_x_mint = next_account_info(account_info_iter)?;
    let usdc_mint = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Verify PDA
    let (expected_pda, bump_seed) = Pubkey::find_program_address(
        &[
            b"twap",
            token_x_mint.key.as_ref(),
            usdc_mint.key.as_ref(),
        ],
        program_id,
    );

    if expected_pda != *twap_storage_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if account is already initialized
    if twap_storage_account.data.borrow().is_empty() {
        // Create account
        let rent = solana_program::sysvar::rent::Rent::get()?;
        let lamports = rent.minimum_balance(TwapStorage::LEN);

        invoke_signed(
            &solana_program::system_instruction::create_account(
                payer.key,
                twap_storage_account.key,
                lamports,
                TwapStorage::LEN as u64,
                program_id,
            ),
            &[payer.clone(), twap_storage_account.clone(), system_program.clone()],
            &[&[
                b"twap",
                token_x_mint.key.as_ref(),
                usdc_mint.key.as_ref(),
                &[bump_seed],
            ]],
        )?;

        // Initialize storage
        let mut twap_storage = TwapStorage {
            is_initialized: true,
            token_x_mint: *token_x_mint.key,
            usdc_mint: *usdc_mint.key,
            observations: [PriceObservation::default(); OBSERVATION_COUNT],
            current_index: 0,
            observation_count: 0,
        };

        twap_storage.serialize(&mut &mut twap_storage_account.data.borrow_mut()[..])?;
        
        msg!("TWAP storage initialized for token pair");
    }

    Ok(())
}

fn process_update_price_observation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let twap_storage_account = next_account_info(account_info_iter)?;
    let pool_account = next_account_info(account_info_iter)?;
    let base_vault_account = next_account_info(account_info_iter)?;
    let quote_vault_account = next_account_info(account_info_iter)?;
    let clock_account = next_account_info(account_info_iter)?;

    // Get current time
    let clock = Clock::from_account_info(clock_account)?;
    let current_timestamp = clock.unix_timestamp;

    // Load TWAP storage
    let mut twap_storage = TwapStorage::try_from_slice(&twap_storage_account.data.borrow())?;
    
    if !twap_storage.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // Check if enough time has passed
    if twap_storage.observation_count > 0 {
        let last_index = if twap_storage.current_index == 0 {
            OBSERVATION_COUNT - 1
        } else {
            (twap_storage.current_index - 1).into()
        };
        let last_observation = twap_storage.observations[last_index];
        if current_timestamp - last_observation.timestamp < OBSERVATION_INTERVAL {
            msg!("Not enough time passed since last observation");
            return Ok(()); // Skip update
        }
    }

    // Read pool state and calculate current price
    let current_price = get_current_pool_price(
        pool_account,
        base_vault_account,
        quote_vault_account,
    )?;

    // Add new observation (circular buffer)
    twap_storage.observations[twap_storage.current_index as usize] = PriceObservation {
        timestamp: current_timestamp,
        price: current_price,
    };

    // Update indices
    twap_storage.current_index = (twap_storage.current_index + 1) % (OBSERVATION_COUNT as u8);
    if usize::from(twap_storage.observation_count) < OBSERVATION_COUNT {
            twap_storage.observation_count += 1;
    }

    // Save updated storage
    twap_storage.serialize(&mut &mut twap_storage_account.data.borrow_mut()[..])?;

    // log price in usdc(decimals: 6)
    msg!("Price observation updated: {} at timestamp {}", current_price / 1_000_000, current_timestamp);
    Ok(())
}

fn process_get_twap_price(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    window_minutes: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let twap_storage_account = next_account_info(account_info_iter)?;
    let clock_account = next_account_info(account_info_iter)?;

    let clock = Clock::from_account_info(clock_account)?;
    let current_timestamp = clock.unix_timestamp;
    let window_seconds = window_minutes as i64 * 60;

    // Load TWAP storage
    let twap_storage = TwapStorage::try_from_slice(&twap_storage_account.data.borrow())?;
    
    if !twap_storage.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if twap_storage.observation_count == 0 {
        msg!("No observations available");
        return Err(ProgramError::InvalidAccountData);
    }

    // Calculate TWAP
    let twap_price = calculate_twap(&twap_storage, current_timestamp, window_seconds)?;
    
    msg!("TWAP price calculated: {}", twap_price / 1_000_000); // log price in usdc (decimals: 6)
    Ok(())
}

fn get_current_pool_price(
    pool_account: &AccountInfo,
    base_vault_account: &AccountInfo,
    quote_vault_account: &AccountInfo,
) -> Result<u64, ProgramError> {
    // Read token amounts from vault accounts
    let base_amount = get_token_account_amount(base_vault_account)?;
    let quote_amount = get_token_account_amount(quote_vault_account)?;

    if base_amount == 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Calculate price: quote_amount / base_amount, scaled by 10^9
    let price = (quote_amount as u128)
        .checked_mul(1_000_000_000_u128)
        .and_then(|x| x.checked_div(base_amount as u128))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(price)
}

// Helper function to find the TWAP storage PDA
pub fn find_twap_storage_address(
    token_x_mint: &Pubkey,
    usdc_mint: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"twap",
            token_x_mint.as_ref(),
            usdc_mint.as_ref(),
        ],
        program_id,
    )
}

// Helper function to find Raydium pool address
pub fn find_raydium_pool_address(
    token_a_mint: &Pubkey,
    token_b_mint: &Pubkey,
) -> (Pubkey, u8) {
    // Raydium uses a specific seed pattern for pool PDAs
    // This might need adjustment based on Raydium's actual implementation
    let (mut base, mut quote) = (token_a_mint, token_b_mint);
    if base > quote {
        std::mem::swap(&mut base, &mut quote);
    }
    
    Pubkey::find_program_address(
        &[
            base.as_ref(),
            quote.as_ref(),
        ],
        &RAYDIUM_AMM_PROGRAM_ID,
    )
}