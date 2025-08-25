// integration tests

use solana_program_test::*;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    pubkey::Pubkey,
};
use cnft_reclaim::processor::Processor;


#[tokio::test]
async fn test_process_begin_reclaim_real() {
    use std::str::FromStr;
    use borsh::BorshSerialize;
    use cnft_reclaim::instruction::ReclaimInstruction;
    use solana_sdk::{signature::read_keypair_file, signer::Signer, transaction::Transaction, pubkey::Pubkey};

    // Use the deployed program ID
    let program_id = Pubkey::from_str("BuKUMTi3AVa7FTbLLyrzcmYS3ctsVzV8JbtUNmTVpnjG").unwrap();
    // Use the real Pyth price feed address
    let pyth_feed_pubkey = Pubkey::from_str("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ").unwrap();

    // Load payer from Solana CLI config path
    let payer = read_keypair_file(
        shellexpand::tilde("~/.config/solana/id.json").to_string()
    ).expect("Failed to read keypair file");

    // TODO: Set the correct TwapAccount PDA for your program and feed
    let twap_key = Pubkey::new_unique(); // Replace with PDA if needed

    // Build the instruction for process_begin_reclaim
    let burn_amount = 850_000u64; // 0.85 SOL in 1e6 units
    let ix_data = ReclaimInstruction::BeginReclaim { burn_amount }.try_to_vec().unwrap();
    let instruction = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(twap_key, false),
            solana_sdk::instruction::AccountMeta::new(payer.pubkey(), true),
            solana_sdk::instruction::AccountMeta::new_readonly(pyth_feed_pubkey, false),
        ],
        data: ix_data,
    };

    // You must use a real RPC client for devnet/mainnet, not program_test
    let rpc_url = std::env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = solana_client::rpc_client::RpcClient::new(rpc_url);

    // Get a recent blockhash
    let recent_blockhash = client.get_latest_blockhash().expect("blockhash");

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    let signature = client.send_and_confirm_transaction(&transaction).expect("send tx");
    println!("Transaction signature: {}", signature);
}
