use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum ReclaimInstruction {
    /// Initialize a vault holding the locked cNFT
    /// Accounts: [writable vault PDA, nft account, authority]
    InitializeVault { total_supply: u64 },

    /// Start reclaim, burn fractions, deposit compensation
    /// Accounts: [vault PDA, fraction mint, reclaimer token account, token program]
    BeginReclaim { burn_amount: u64 },

    /// Minority holder claims buyout
    /// Accounts: [vault PDA, claimant token account, claimant wallet]
    ClaimBuyout {},

    /// Update TWAP from a Pyth price feed
    /// Accounts:
    /// 0. [writable] TwapAccount
    /// 1. [signer] Payer/authority (optional, just funds rent)
    /// 2. [] Pyth Price Feed Account
    UpdateTwap {},

    /// Reclaim fractions (will read TWAP stored in TwapAccount)
    ReclaimFractions {
        amount: u64,
    },
}

impl ReclaimInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        ReclaimInstruction::try_from_slice(input).map_err(|_| ProgramError::InvalidInstructionData)
    }
}
