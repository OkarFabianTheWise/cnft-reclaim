use reclaim_amm::find_raydium_pool_address;
// Integration tests for reclaim-amm program
use std::str::FromStr;
use solana_program_test::*;
use solana_sdk::{
    instruction::{
        Instruction,
        AccountMeta
    },
    signature::{Keypair, read_keypair_file},
    signer::Signer,
    transaction::Transaction,
    pubkey::Pubkey,
    system_program,
};
use reclaim_amm::{find_twap_storage_address};
use reclaim_amm::instruction::TwapInstruction;
use borsh::BorshSerialize;
use solana_client::rpc_client::RpcClient;
mod raydium;

// Helper to get payer keypair from CLI config
fn get_payer() -> Keypair {
    read_keypair_file(shellexpand::tilde("~/.config/solana/id.json").to_string())
        .expect("Failed to read keypair file")
}

// Helper to get payer keypair from CLI config
#[tokio::test(flavor = "multi_thread")]
async fn test_reclaim() {
    let payer = get_payer();
    let user = &payer;
    let token_x_mint = Pubkey::from_str("HyjDHQrqA7YogQGAEcJA6zJaeTRVJ5qWSJABBEF8cNGf").unwrap();
    let token_program = spl_token::id();

    // User is payer
    let user_token_x_account = spl_associated_token_account::get_associated_token_address(&user.pubkey(), &token_x_mint);

    let rpc_url = std::env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = RpcClient::new(rpc_url);
    let recent_blockhash = client.get_latest_blockhash().expect("blockhash");

    // Mint 2_000_000 token_x (decimals: 6) to user
    // (You may need to airdrop SOL to payer for fees)
    // For brevity, skip minting code here; assume user_token_x_account has 2_000_000

    // Build reclaim instruction
    let program_id = Pubkey::from_str("EAUWzk6LNrRPCYeSfxkbp659MUqTsDrQbJcLUKrLzAZc").unwrap();
    let reclaim_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(user_token_x_account, false),
            AccountMeta::new(token_x_mint, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: TwapInstruction::Reclaim.try_to_vec().unwrap(),
    };

    // Send transaction
    let mut tx = Transaction::new_with_payer(&[reclaim_ix], Some(&user.pubkey()));
    tx.sign(&[user], recent_blockhash);
    let sig = client.send_and_confirm_transaction(&tx).expect("send tx");
    println!("Reclaim tx: {}", sig);

    // Try reclaiming again (should error, balance is zero)
    let over_reclaim_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(user_token_x_account, false),
            AccountMeta::new(token_x_mint, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: TwapInstruction::Reclaim.try_to_vec().unwrap(),
    };
    let mut tx2 = Transaction::new_with_payer(&[over_reclaim_ix], Some(&user.pubkey()));
    tx2.sign(&[user], recent_blockhash);
    let result = client.send_and_confirm_transaction(&tx2);
    assert!(result.is_err(), "Should error if user does not hold any token_x");
}


#[tokio::test(flavor = "multi_thread")]
async fn test_initialize_twap_storage() {
    let program_id = Pubkey::from_str("EAUWzk6LNrRPCYeSfxkbp659MUqTsDrQbJcLUKrLzAZc").unwrap();
    let token_x_mint = Pubkey::from_str("HyjDHQrqA7YogQGAEcJA6zJaeTRVJ5qWSJABBEF8cNGf").unwrap(); // Example: SOL
    let usdc_mint = Pubkey::from_str("9KRFTov9dsj5r4jTLc5h9VszjXoPKVKUkzEf8WxBox2k").unwrap(); // Example: USDC
    let payer = get_payer();
    let (twap_storage, _bump) = find_twap_storage_address(&token_x_mint, &usdc_mint, &program_id);
    // Fetch pool info from API
    let ix = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(payer.pubkey(), true),
            solana_sdk::instruction::AccountMeta::new(twap_storage, false),
            solana_sdk::instruction::AccountMeta::new_readonly(token_x_mint, false),
            solana_sdk::instruction::AccountMeta::new_readonly(usdc_mint, false),
            solana_sdk::instruction::AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: TwapInstruction::InitializeTwapStorage.try_to_vec().unwrap(),
    };

    let rpc_url = std::env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = RpcClient::new(rpc_url);
    let recent_blockhash = client.get_latest_blockhash().expect("blockhash");
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    let sig = client.send_and_confirm_transaction(&tx).expect("send tx");
    println!("InitializeTwapStorage tx: {}", sig);
}


#[tokio::test(flavor = "multi_thread")]
async fn test_update_price_observation() -> Result<(), Box<dyn std::error::Error>> {
    use raydium::{get_pool_accounts_for_instruction, Network, WSOL};
    let payer = get_payer();
    // Use env var to select network
    let network = match std::env::var("RAYDIUM_NETWORK").unwrap_or_else(|_| "devnet".to_string()).to_lowercase().as_str() {
        "mainnet" => Network::Mainnet,
        _ => Network::Devnet,
    };
    // Hardcoded pool and vault addresses for deterministic integration test
    let pool_account = Pubkey::from_str("DtcPMALiFGdTba3hB1sM2hawYqHG3pP7178wgteDh8zD").unwrap(); // ammId
    let base_vault = Pubkey::from_str("3fGADpNgURuM3JuhuHAKdHWeABvps37np12NW3Pq9n3n").unwrap(); // coinVault
    let quote_vault = Pubkey::from_str("Bpux66KMgrbZKgiYG2o4UqfxriVBVbcGVRA2hUCj9DcH").unwrap(); // pcVault

    // Log vault details
    println!("Pool Account: {}", pool_account);
    println!("Base Vault: {}", base_vault);
    println!("Quote Vault: {}", quote_vault);

    let program_id = Pubkey::from_str("EAUWzk6LNrRPCYeSfxkbp659MUqTsDrQbJcLUKrLzAZc").unwrap();
    // Use the correct program_id and mints for the network
    let raydium_amm = network.raydium_program_id();
    let token_x_mint = Pubkey::from_str(WSOL).unwrap();
    let usdc_mint = network.usdc_mint();
    let (twap_storage, _bump) = find_twap_storage_address(&token_x_mint, &usdc_mint, &program_id);

    let clock_sysvar = solana_program::sysvar::clock::ID;

    let ix = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(twap_storage, false),
            solana_sdk::instruction::AccountMeta::new_readonly(pool_account, false),
            solana_sdk::instruction::AccountMeta::new_readonly(base_vault, false),
            solana_sdk::instruction::AccountMeta::new_readonly(quote_vault, false),
            solana_sdk::instruction::AccountMeta::new_readonly(clock_sysvar, false),
        ],
        data: TwapInstruction::UpdatePriceObservation.try_to_vec().unwrap(),
    };

    let rpc_url = std::env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = RpcClient::new(rpc_url);
    let recent_blockhash = client.get_latest_blockhash().expect("blockhash");
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    let sig = client.send_and_confirm_transaction(&tx).expect("send tx");
    println!("UpdatePriceObservation tx: {}", sig);
    Ok(())
}


#[tokio::test(flavor = "multi_thread")]
async fn test_get_twap_price() {
    let program_id = Pubkey::from_str("EAUWzk6LNrRPCYeSfxkbp659MUqTsDrQbJcLUKrLzAZc").unwrap();
    let token_x_mint = Pubkey::from_str("HyjDHQrqA7YogQGAEcJA6zJaeTRVJ5qWSJABBEF8cNGf").unwrap();
    let usdc_mint = Pubkey::from_str("9KRFTov9dsj5r4jTLc5h9VszjXoPKVKUkzEf8WxBox2k").unwrap();
    let payer = get_payer();
    let (twap_storage, _bump) = find_twap_storage_address(&token_x_mint, &usdc_mint, &program_id);
    let clock_sysvar = solana_program::sysvar::clock::ID;

    let ix = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new_readonly(twap_storage, false),
            solana_sdk::instruction::AccountMeta::new_readonly(clock_sysvar, false),
        ],
        data: TwapInstruction::GetTwapPrice { window_minutes: 5 }.try_to_vec().unwrap(),
    };

    let rpc_url = std::env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = RpcClient::new(rpc_url);
    let recent_blockhash = client.get_latest_blockhash().expect("blockhash");
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    let sig = client.send_and_confirm_transaction(&tx).expect("send tx");
    println!("GetTwapPrice tx: {}", sig);
}


#[test]
fn test_find_twap_storage_address() {
    let token_x_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let program_id: Pubkey = Pubkey::from_str("EAUWzk6LNrRPCYeSfxkbp659MUqTsDrQbJcLUKrLzAZc").unwrap();
    let (pda, _bump) = find_twap_storage_address(&token_x_mint, &usdc_mint, &program_id);
    println!("TWAP storage PDA: {}", pda);
}

#[test]
fn test_find_raydium_pool_address() {
    let token_a = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let token_b = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let (pda, _bump) = find_raydium_pool_address(&token_a, &token_b);
    println!("Raydium pool PDA: {}", pda);
}