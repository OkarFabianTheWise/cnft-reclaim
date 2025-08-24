use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Vault {
    pub nft_mint: Pubkey,
    pub fraction_mint: Pubkey,
    pub total_supply: u64,
    pub reclaimed: bool,
    pub fair_price_per_fraction: u64, // snapshot at reclaim start
    pub bump: u8,
}
pub mod twap;
