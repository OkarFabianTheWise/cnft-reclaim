use solana_program::pubkey::Pubkey;

// Simplified Raydium pool state structure
#[repr(C, packed)]
#[derive(Debug)]
pub struct RaydiumPoolInfo {
    pub status: u64,
    pub nonce: u64,
    pub max_order: u64,
    pub depth: u64,
    pub base_decimals: u64,
    pub quote_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimals_value: u64,
    // Key addresses
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_need_take_pnl: u64,
    pub quote_need_take_pnl: u64,
    // Add padding/remaining fields as needed based on actual Raydium struct
}

// SPL Token Account structure
#[repr(C)]
#[derive(Debug)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate_option: u32,
    pub delegate: Pubkey,
    pub state: u8,
    pub is_native_option: u32,
    pub is_native: u64,
    pub delegated_amount: u64,
    pub close_authority_option: u32,
    pub close_authority: Pubkey,
}
