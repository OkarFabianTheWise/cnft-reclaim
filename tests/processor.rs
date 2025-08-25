// integration tests
use std::str::FromStr;
use solana_program_test::*;
use solana_sdk::{
    msg,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    pubkey::Pubkey,
};
use cnft_reclaim::processor::Processor;
use solana_program::instruction::AccountMeta;
use solana_program::instruction;

#[tokio::test(flavor = "multi_thread")]
async fn test_process_begin_reclaim() {
    use std::str::FromStr;
    use borsh::BorshSerialize;
    use cnft_reclaim::instruction::ReclaimInstruction;
    use solana_sdk::{signature::read_keypair_file, signer::Signer, transaction::Transaction, pubkey::Pubkey};

    // Use the deployed program ID
    let program_id = Pubkey::from_str("4LLS9Kqk2AB7fbKD4EXvDBi2JH7nfteiLBoMH8jGovVi").unwrap();
    // Use the real Pyth price feed address
    let pyth_feed_pubkey = Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();
    // system program
    let system_program_id = solana_sdk::system_program::ID;

    // Load payer from Solana CLI config path
    let payer = read_keypair_file(
        shellexpand::tilde("~/.config/solana/id.json").to_string()
    ).expect("Failed to read keypair file");

    // Find the PDA for the TWAP account
    let (twap_key, _bump) = Pubkey::find_program_address(&[b"twap"], &program_id);

    // log accounts
    msg!("TWAP Account: {}", twap_key);
    msg!("Payer Account: {}", payer.pubkey());
    msg!("Pyth Feed Account: {}", pyth_feed_pubkey);
    msg!("System Program Account: {}", system_program_id);

    // Build the instruction for process_begin_reclaim
    let burn_amount = 850_000u64; // 0.85 SOL in 1e6 units
    let ix_data = ReclaimInstruction::BeginReclaim { burn_amount }.try_to_vec().unwrap();
    let instruction = solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(twap_key, false),
            solana_sdk::instruction::AccountMeta::new(payer.pubkey(), true),
            solana_sdk::instruction::AccountMeta::new_readonly(pyth_feed_pubkey, false),
            solana_sdk::instruction::AccountMeta::new_readonly(system_program_id, false),
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

/* Add this debug version to help identify the issue 
#[test]
fn debug_pyth_account() {
    let pyth_feed_pubkey = Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();
    let rpc_url = std::env::var("SOLANA_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let client = solana_client::rpc_client::RpcClient::new(rpc_url);
    
    // Fetch the account data to examine its structure
    if let Ok(account) = client.get_account(&pyth_feed_pubkey) {
        println!("Account owner: {}", account.owner);
        println!("Account data length: {}", account.data.len());
        println!("Account executable: {}", account.executable);
        
        // Print first 100 bytes as hex
        let preview_len = std::cmp::min(100, account.data.len());
        println!("First {} bytes as hex:", preview_len);
        for (i, byte) in account.data[0..preview_len].iter().enumerate() {
            if i % 16 == 0 {
                print!("\n{:04x}: ", i);
            }
            print!("{:02x} ", byte);
        }
        println!("\n");
        
        // Try to determine what kind of Pyth account this is
        if account.data.len() >= 8 {
            let discriminator = u64::from_le_bytes([
                account.data[0], account.data[1], account.data[2], account.data[3],
                account.data[4], account.data[5], account.data[6], account.data[7]
            ]);
            println!("Discriminator: 0x{:x}", discriminator);
        }
    } else {
        println!("Failed to fetch account data");
    }
}*/