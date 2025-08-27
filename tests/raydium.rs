use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use borsh::{BorshDeserialize, BorshSerialize};

pub const RAYDIUM_AMM_PROGRAM_MAINNET: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const RAYDIUM_AMM_PROGRAM_DEVNET: &str = "DRaya7Kj3aMWQSy19kSjvmuwq9docCHofyP9kanQGaav";

pub const USDC_MAINNET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
// pub const USDC_DEVNET: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
pub const USDC_DEVNET: &str = "9KRFTov9dsj5r4jTLc5h9VszjXoPKVKUkzEf8WxBox2k";
pub const WSOL: &str = "HyjDHQrqA7YogQGAEcJA6zJaeTRVJ5qWSJABBEF8cNGf";

#[derive(Debug, Clone)]
pub struct RaydiumPoolInfo {
    pub pool_address: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub open_orders: Pubkey,
    pub market_id: Pubkey,
}

#[derive(Copy, Clone, Debug)]
pub enum Network {
    Mainnet,
    Devnet,
}

impl Network {
    pub fn raydium_program_id(&self) -> Pubkey {
        match self {
            Network::Mainnet => Pubkey::from_str(RAYDIUM_AMM_PROGRAM_MAINNET).unwrap(),
            Network::Devnet => Pubkey::from_str(RAYDIUM_AMM_PROGRAM_DEVNET).unwrap(),
        }
    }
    pub fn usdc_mint(&self) -> Pubkey {
        match self {
            Network::Mainnet => Pubkey::from_str(USDC_MAINNET).unwrap(),
            Network::Devnet => Pubkey::from_str(USDC_DEVNET).unwrap(),
        }
    }
    pub fn rpc_url(&self) -> &str {
        match self {
            Network::Mainnet => "https://api.mainnet-beta.solana.com",
            Network::Devnet => "https://api.devnet.solana.com",
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RaydiumAmmInfo {
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
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_need_take_pnl: u64,
    pub quote_need_take_pnl: u64,
    pub quote_total_pnl: u64,
    pub base_total_pnl: u64,
    pub pool_open_time: u64,
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,
    pub orderbook_to_init_time: u64,
    pub swap_base_in_amount: u128,
    pub swap_quote_out_amount: u128,
    pub swap_base2_quote_fee: u64,
    pub swap_quote_in_amount: u128,
    pub swap_base_out_amount: u128,
    pub swap_quote2_base_fee: u64,
    pub lp_mint: Pubkey,
    pub open_orders: Pubkey,
    pub market_id: Pubkey,
    pub market_program_id: Pubkey,
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub lp_vault: Pubkey,
    pub owner: Pubkey,
}

pub struct RaydiumPoolFetcher {
    client: RpcClient,
    network: Network,
}

impl RaydiumPoolFetcher {
    pub fn new(network: Network) -> Self {
        let client = RpcClient::new_with_commitment(
            network.rpc_url().to_string(),
            CommitmentConfig::confirmed(),
        );
        Self { client, network }
    }
    pub async fn find_pool_by_mints(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
    ) -> Result<Option<RaydiumPoolInfo>, Box<dyn std::error::Error>> {
        let program_id = self.network.raydium_program_id();
        let accounts = self.client.get_program_accounts(&program_id)?;
        for (pubkey, account) in accounts {
            if account.data.len() < std::mem::size_of::<RaydiumAmmInfo>() {
                continue;
            }
            if let Ok(pool_info) = self.parse_pool_account(&account.data) {
                if (pool_info.base_mint == *token_a && pool_info.quote_mint == *token_b) ||
                   (pool_info.base_mint == *token_b && pool_info.quote_mint == *token_a) {
                    return Ok(Some(RaydiumPoolInfo {
                        pool_address: pubkey,
                        base_mint: pool_info.base_mint,
                        quote_mint: pool_info.quote_mint,
                        base_vault: pool_info.base_vault,
                        quote_vault: pool_info.quote_vault,
                        lp_mint: pool_info.lp_mint,
                        open_orders: pool_info.open_orders,
                        market_id: pool_info.market_id,
                    }));
                }
            }
        }
        Ok(None)
    }
    pub fn parse_pool_account(&self, data: &[u8]) -> Result<RaydiumAmmInfo, Box<dyn std::error::Error>> {
        if data.len() < std::mem::size_of::<RaydiumAmmInfo>() {
            return Err("Account data too small".into());
        }
        unsafe {
            let pool_info = std::ptr::read_unaligned(data.as_ptr() as *const RaydiumAmmInfo);
            Ok(pool_info)
        }
    }
}

pub async fn get_pool_accounts_for_instruction(
    network: &Network,
    token_x_mint: &str,
    usdc_mint_override: Option<&str>,
) -> Result<(Pubkey, Pubkey, Pubkey), Box<dyn std::error::Error>> {
    let fetcher = RaydiumPoolFetcher::new(*network);
    let token_x = Pubkey::from_str(token_x_mint)?;
    let usdc = if let Some(override_mint) = usdc_mint_override {
        Pubkey::from_str(override_mint)?
    } else {
    fetcher.network.usdc_mint()
    };
    match fetcher.find_pool_by_mints(&token_x, &usdc).await? {
        Some(pool) => Ok((pool.pool_address, pool.base_vault, pool.quote_vault)),
        None => Err(format!("No pool found for {} / USDC pair", token_x_mint).into()),
    }
}
