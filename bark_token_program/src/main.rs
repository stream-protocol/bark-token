// main.rs

use anchor_lang::prelude::*;
use env_logger;

mod transferhook;
use transferhook::transferhook::initialize;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Start the Anchor runtime
    anchor_lang::env::start(env::args().collect());

    // Define Solana program entry
    let program = anchor_lang::program!();

    // Configure the fee rate and fee wallet
    let fee_rate: u8 = 2;  // Adjust the fee rate as needed
    let fee_wallet = Pubkey::new_from_array([0; 32]);  // Replace with the actual fee wallet address

    // Run the initialization logic
    program
        .initialize(
            anchor_lang::client::Client::default(),
            false,
            fee_rate,
            fee_wallet,
        )
        .await
        .unwrap_or_else(|err| {
            eprintln!("Error initializing Bark program: {}", err);
            std::process::exit(1);
        });

    // Stop the Anchor runtime
    anchor_lang::env::stop();
}
