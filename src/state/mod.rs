use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TwapStorage {
    pub is_initialized: bool,
    pub token_x_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub observations: [PriceObservation; 3], // Fixed size array for 3 observations
    pub current_index: u8, // Circular buffer index
    pub observation_count: u8, // Number of valid observations (0-3)
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct PriceObservation {
    pub timestamp: i64,
    pub price: u64, // Price scaled by 10^9 (tokenX per USDC)
}

impl Default for PriceObservation {
    fn default() -> Self {
        Self {
            timestamp: 0,
            price: 0,
        }
    }
}

impl TwapStorage {
    pub const LEN: usize = 1 + 32 + 32 + (3 * 16) + 1 + 1; // 115 bytes
}
