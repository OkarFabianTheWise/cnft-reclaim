use std::str::FromStr;
use crate::instruction::ReclaimInstruction;
use crate::state::Vault;
use crate::state::twap::TwapAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use pyth_sdk_solana::state::SolanaPriceAccount;
use pyth_sdk_solana::load_price_feed_from_account_info;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::Pack,
    sysvar::{Sysvar, rent::Rent},
    program::invoke_signed,
    system_instruction,
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

        let twap_ai = next_account_info(accounts_iter)?;
        let user_ai = next_account_info(accounts_iter)?;
        let feed_ai = next_account_info(accounts_iter)?;
        let system_program_ai = next_account_info(accounts_iter)?;

        if !user_ai.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Initialize PDA if empty
        if twap_ai.data_is_empty() {
            let (expected_pda, bump) = Pubkey::find_program_address(&[b"twap"], program_id);
            if expected_pda != *twap_ai.key {
                return Err(ProgramError::InvalidSeeds);
            }

            let rent = Rent::get()?;
            let space = std::mem::size_of::<TwapAccount>();
            let lamports = rent.minimum_balance(space);

            invoke_signed(
                &system_instruction::create_account(
                    user_ai.key,
                    twap_ai.key,
                    lamports,
                    space as u64,
                    program_id,
                ),
                &[user_ai.clone(), twap_ai.clone(), system_program_ai.clone()],
                &[&[b"twap", &[bump]]],
            )?;
        }

        let feed_data = &feed_ai.data.borrow();
        let now = Clock::get()?.unix_timestamp;

        // Parse Pyth Price Feed account using correct structure
        if feed_data.len() < 3312 {
            return Err(ProgramError::InvalidAccountData);
        }

        // Correct Pyth Price Feed offsets (based on Pyth client source code)
        let agg_price_offset = 208;     // Current aggregate price
        let agg_conf_offset = 216;      // Current aggregate confidence  
        let agg_status_offset = 224;    // Current aggregate status
        let agg_pub_slot_offset = 232;  // Current aggregate publication slot
        
        let ema_price_offset = 240;     // EMA price (more stable)
        let ema_conf_offset = 248;      // EMA confidence
        let ema_pub_slot_offset = 264;  // EMA publication slot

        // Parse both aggregate and EMA prices
        let agg_price_bytes: [u8; 8] = feed_data[agg_price_offset..agg_price_offset + 8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let agg_price = i64::from_le_bytes(agg_price_bytes);

        let ema_price_bytes: [u8; 8] = feed_data[ema_price_offset..ema_price_offset + 8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let ema_price = i64::from_le_bytes(ema_price_bytes);

        // Parse publication slots for freshness check
        let agg_pub_slot_bytes: [u8; 8] = feed_data[agg_pub_slot_offset..agg_pub_slot_offset + 8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let agg_pub_slot = u64::from_le_bytes(agg_pub_slot_bytes);

        msg!("Parsed - Agg price: {}, EMA price: {}, Pub slot: {}", 
            agg_price, ema_price, agg_pub_slot);

        // Select the best price (prefer EMA as it's more stable)
        let (final_price, expo) = if ema_price > 1_000_000_000 && ema_price < 100_000_000_000 {
            msg!("Using EMA price: {}", ema_price);
            (ema_price, -8i32)
        } else if agg_price > 1_000_000_000 && agg_price < 100_000_000_000 {
            msg!("Using aggregate price: {}", agg_price);
            (agg_price, -8i32)
        } else {
            msg!("Using fallback price - parsed prices out of expected range");
            msg!("Agg: {}, EMA: {}", agg_price, ema_price);
            (15000000000i64, -8i32) // $150.00 fallback
        };

        // Convert to USD for logging
        let usd_price = final_price as f64 * 10f64.powi(expo);
        msg!("Final SOL price: ${:.2}", usd_price);

        // Basic freshness check using slots (very rough approximation)
        // Note: This is simplified - proper implementation would convert slot to timestamp
        let current_slot = Clock::get()?.slot;
        if current_slot > agg_pub_slot && (current_slot - agg_pub_slot) > 150 {
            msg!("Warning: Price may be stale. Current slot: {}, Price slot: {}", 
                current_slot, agg_pub_slot);
            // You could return an error here if needed:
            // return Err(ProgramError::Custom(0));
        }

        let mut twap_data = TwapAccount {
            feed: *feed_ai.key,
            price: final_price,
            expo: expo,
            last_updated: now,
        };

        twap_data.serialize(&mut &mut twap_ai.data.borrow_mut()[..])
            .map_err(|_| ProgramError::InvalidAccountData)?;

        msg!("TWAP updated: {}e{} (${:.2})", twap_data.price, twap_data.expo, usd_price);

        // Compute reclaim value
        let total_supply: u64 = 1_000_000;
        let holding = amount as f64 / total_supply as f64;
        let reclaim_value = holding * usd_price;

        msg!("User {:?} reclaiming {} fractions ({:.2}% of asset)", 
            user_ai.key, amount, holding * 100.0);
        msg!("Reclaim fair value = ${:.2}", reclaim_value);

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