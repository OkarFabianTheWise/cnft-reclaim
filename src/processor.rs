use std::str::FromStr;
use crate::instruction::ReclaimInstruction;
use crate::state::Vault;
use crate::state::twap::TwapAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;
use solana_program::clock::Clock;
use pyth_sdk_solana::state::SolanaPriceAccount;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_pack::Pack,
    sysvar::Sysvar,
};

pub struct Processor;
// Hardcoded Pyth price feed public key
const PYTH_FEED_PUBKEY_STR: &str = "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ";

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

        // Check that the provided feed_ai matches the hardcoded pubkey
        let expected_feed_pubkey = Pubkey::from_str(PYTH_FEED_PUBKEY_STR)
            .map_err(|_| ProgramError::InvalidArgument)?;
        if feed_ai.key != &expected_feed_pubkey {
            msg!("Error: Provided feed account does not match expected Pyth feed pubkey");
            return Err(ProgramError::InvalidArgument);
        }

        if !user_ai.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // 1. Load and update TWAP from Pyth
        let feed = pyth_sdk_solana::load_price_feed_from_account_info(feed_ai)
            .map_err(|_| ProgramError::InvalidArgument)?;

        let now = Clock::get()?.unix_timestamp;

        let price_data = feed
            .get_ema_price_no_older_than(now, 300)
            .ok_or(ProgramError::InvalidArgument)?;

        let mut twap_data = TwapAccount {
            feed: *feed_ai.key,
            price: price_data.price,
            expo: price_data.expo,
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