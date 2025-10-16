// RISC Zero Host Program: Auction Proof Generator
//
// This program runs the double auction guest program in the RISC Zero zkVM
// and generates a cryptographic receipt proving correct execution.

use methods::DOUBLE_AUCTION_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt, ProverOpts, recursion::identity_p254};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub id: u32,
    pub role: u32,
    pub price: u64,
    pub quantity: u64,
    pub in_coin: u64,
    pub in_energy: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuctionInput {
    pub participants: Vec<Participant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuctionScenario {
    pub scenario_name: String,
    pub description: String,
    pub participants: Vec<Participant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicJournal {
    pub in_coin: Vec<u64>,
    pub in_energy: Vec<u64>,
    pub out_coin: Vec<u64>,
    pub out_energy: Vec<u64>,
}

fn main() {
    println!("═══════════════════════════════════════════════");
    println!("  RISC Zero Double Auction Proof Generator");
    println!("═══════════════════════════════════════════════\n");

    // Get scenario file from command line
    let args: Vec<String> = env::args().collect();
    let scenario_file = if args.len() > 1 {
        &args[1]
    } else {
        "auction_scenario.json"
    };

    // Load scenario
    let scenario = load_scenario(scenario_file).expect("Failed to load scenario");
    println!("✓ Loaded scenario: {}", scenario.scenario_name);
    println!("  Participants: {}\n", scenario.participants.len());

    // Prepare input for guest (only id, role, price, quantity, in_coin, in_energy)
    let guest_input = AuctionInput {
        participants: scenario.participants.clone(),
    };

    // Build executor environment
    let env = ExecutorEnv::builder()
        .write(&guest_input)
        .unwrap()
        .build()
        .unwrap();

    // Run the prover
    println!("▸ Generating RISC Zero proof...");
    let prover = default_prover();
    let opts = ProverOpts::succinct();

    let prove_info = prover
        .prove_with_opts(env, DOUBLE_AUCTION_GUEST_ELF, &opts)
        .expect("Failed to generate proof");

    let receipt = prove_info.receipt;
    println!("✓ Proof generated\n");

    // Decode journal
    let journal: PublicJournal = receipt.journal.decode().expect("Failed to decode journal");

    // Display results
    println!("▸ Auction Results:");
    println!("  Participants: {}", journal.in_coin.len());
    println!();

    // Verify receipt (optional but recommended)
    // receipt.verify(DOUBLE_AUCTION_ID).expect("Receipt verification failed");

    // Convert to identity_p254 for Groth16
    println!("▸ Converting to Groth16 format...");
    let identity_receipt = match &receipt.inner {
        InnerReceipt::Succinct(succinct) => {
            identity_p254(succinct).expect("Failed to convert to identity_p254")
        }
        _ => panic!("Expected succinct receipt"),
    };

    // Extract seal bytes for Circom
    let seal_bytes: Vec<u8> = identity_receipt
        .seal
        .iter()
        .flat_map(|&x| x.to_le_bytes())
        .collect();

    // Save input.json for Circom
    let output_file = fs::File::create("input.json").expect("Failed to create input.json");
    let cursor = std::io::Cursor::new(&seal_bytes);
    risc0_zkvm::seal_to_json(cursor, output_file).expect("Failed to write input.json");
    println!("✓ Generated input.json ({} bytes)\n", seal_bytes.len());

    // Save receipt for Go integration
    let receipt_json = serde_json::to_string_pretty(&receipt).expect("Failed to serialize receipt");
    fs::write("risc0_receipt.json", receipt_json).expect("Failed to write receipt");
    println!("✓ Saved risc0_receipt.json");

    // Save journal for verification
    let journal_json = serde_json::to_string_pretty(&journal).expect("Failed to serialize journal");
    fs::write("journal.json", journal_json).expect("Failed to write journal");
    println!("✓ Saved journal.json");

    println!("\n✓ RISC Zero proof generation complete");
}

fn load_scenario(filename: &str) -> Result<AuctionScenario, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(filename)?;
    let scenario: AuctionScenario = serde_json::from_str(&content)?;
    Ok(scenario)
}
