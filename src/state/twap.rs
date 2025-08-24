use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TwapAccount {
    pub feed: Pubkey,       // Pyth feed this TWAP belongs to
    pub price: i64,         // Mantissa (e.g. 24600000000 for $246)
    pub expo: i32,          // Exponent (e.g. -8)
    pub last_updated: i64,  // Unix timestamp of last update
}
