use std::str::FromStr;
use crate::instruction::ReclaimInstruction;
use crate::state::Vault;
use crate::state::twap::TwapAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use pyth_sdk_solana::state::SolanaPriceAccount;
use solana_program::account_info::AccountInfo;
use solana_program::{
    account_info::{next_account_info},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::Pack,
    sysvar::Sysvar,
};

pub struct Processor;
// Hardcoded Pyth price feed public key
// const PYTH_FEED_PUBKEY_STR: &str = "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ";

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = ReclaimInstruction::unpack(instruction_data)?;
        match instruction {
            ReclaimInstruction::InitializeVault { total_supply } => {
                msg!("Instruction: InitializeVault");
                Self::process_initialize_vault(program_id, accounts, total_supply)
            }
            ReclaimInstruction::BeginReclaim { burn_amount } => {
                msg!("Instruction: BeginReclaim");
                Self::process_begin_reclaim(program_id, accounts, burn_amount)
            }
            ReclaimInstruction::ClaimBuyout {} => {
                msg!("Instruction: ClaimBuyout");
                Self::process_claim_buyout(program_id, accounts)
            }
            ReclaimInstruction::UpdateTwap {} => {
                msg!("Instruction: UpdateTwap");
                // TODO: implement process_update_twap
                Ok(())
            }
            ReclaimInstruction::ReclaimFractions { amount } => {
                msg!("Instruction: ReclaimFractions");
                // TODO: implement process_reclaim_fractions
                Ok(())
            }
        }
    }

    fn process_initialize_vault(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _total_supply: u64,
    ) -> ProgramResult {
        // TODO: store Vault state in PDA
        Ok(())
    }

    pub fn process_begin_reclaim(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let twap_ai = next_account_info(accounts_iter)?;   // writable TwapAccount
    let user_ai = next_account_info(accounts_iter)?;   // signer reclaiming
    let feed_ai = next_account_info(accounts_iter)?;   // Pyth feed (SOL/USD)

    if !user_ai.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Manual approach: Extract data from AccountInfo and parse Pyth price manually
    let feed_data = &feed_ai.data.borrow();
    let feed_key = feed_ai.key;
    let feed_owner = feed_ai.owner;
    let feed_lamports = feed_ai.lamports();
    let feed_executable = feed_ai.executable;

    let now = Clock::get()?.unix_timestamp;

    // Parse Pyth price data manually from the account data
    // Pyth price account structure starts with a discriminator
    if feed_data.len() < 8 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if this is a valid Pyth price account
    // Pyth accounts typically start with specific magic bytes
    let magic = u32::from_le_bytes([feed_data[0], feed_data[1], feed_data[2], feed_data[3]]);
    if magic != 0xa1b2c3d4 { // Pyth magic number (adjust if needed)
        msg!("Warning: Pyth magic number not found, proceeding anyway");
    }

    // Extract price and expo from fixed offsets in Pyth account data
    // These offsets are based on Pyth's account structure
    let price_offset = 208; // Adjust based on actual Pyth structure
    let expo_offset = 216;
    let timestamp_offset = 224;

    if feed_data.len() < timestamp_offset + 8 {
        return Err(ProgramError::InvalidAccountData);
    }

    let price_bytes: [u8; 8] = feed_data[price_offset..price_offset + 8]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let price = i64::from_le_bytes(price_bytes);

    let expo_bytes: [u8; 4] = feed_data[expo_offset..expo_offset + 4]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let expo = i32::from_le_bytes(expo_bytes);

    let timestamp_bytes: [u8; 8] = feed_data[timestamp_offset..timestamp_offset + 8]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let price_timestamp = i64::from_le_bytes(timestamp_bytes);

    // Check if price is not too old (5 minutes = 300 seconds)
    if now - price_timestamp > 300 {
        return Err(ProgramError::Custom(0)); // Custom error for stale price
    }

    let mut twap_data = TwapAccount {
        feed: *feed_ai.key,
        price: price,
        expo: expo,
        last_updated: now,
    };

    // Store new TWAP into account
    twap_data.serialize(&mut &mut twap_ai.data.borrow_mut()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!("TWAP updated: {}e{}", twap_data.price, twap_data.expo);

    // 2. Compute reclaim value
    // total supply assumed = 1 (whole asset = 1 SOL)
    let total_supply: u64 = 1_000_000; // fixed-point (1e6 = 1 SOL)
    let holding = amount as f64 / total_supply as f64; // fraction held

    // convert TWAP to f64
    let twap_price = (twap_data.price as f64) * 10f64.powi(twap_data.expo);
    let reclaim_value = holding * twap_price;

    msg!("User {:?} reclaiming {} fractions (~{} of asset)", 
        user_ai.key, amount, holding);
    msg!("Reclaim fair value = {} USD", reclaim_value);

    // Later: perform SPL transfer of equivalent value, or store in another account
    Ok(())
}

    fn process_claim_buyout(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
    ) -> ProgramResult {
        // TODO:
        // - verify claimant's fraction balance
        // - transfer compensation from vault to claimant
        // - burn claimant's fractions
        Ok(())
    }
}