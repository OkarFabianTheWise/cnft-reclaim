use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TwapInstruction {
    /// Initialize TWAP storage for a token pair
    /// Accounts:
    /// 0. [writable, signer] Payer
    /// 1. [writable] TWAP storage PDA
    /// 2. [] Token X mint
    /// 3. [] USDC mint  
    /// 4. [] System program
    InitializeTwapStorage,
    
    /// Update price observation by reading pool state
    /// Accounts:
    /// 0. [writable] TWAP storage PDA
    /// 1. [] Raydium pool account
    /// 2. [] Pool's base token vault
    /// 3. [] Pool's quote token vault
    /// 4. [] Clock sysvar
    UpdatePriceObservation,
    
    /// Calculate and return TWAP price
    /// Accounts:
    /// 0. [] TWAP storage PDA
    /// 1. [] Clock sysvar
    GetTwapPrice { window_minutes: u8 },
    /// Burn all token_x from caller's account
    /// Accounts:
    /// 0. [signer] User
    /// 1. [writable] User's token_x account
    /// 2. [writable] token_x mint
    /// 3. [] SPL Token program
    Reclaim,
}
