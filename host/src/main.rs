// RISC Zero Host Program: Auction Proof Generator
//
// This program runs the double auction guest program in the RISC Zero zkVM
// and generates a cryptographic receipt proving correct execution.

use methods::DOUBLE_AUCTION_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, InnerReceipt, ProverOpts, recursion::identity_p254};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::time::Instant;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub participant_count: usize,
    pub scenario_name: String,
    pub user_cycles: u64,
    pub total_cycles: u64,
    pub session_segments: usize,
    pub executor_time_ms: u64,
    pub proving_time_ms: u64,
    pub total_time_ms: u64,
    pub receipt_size_bytes: usize,
    pub journal_size_bytes: usize,
    pub timestamp: String,
}

fn main() {
    let start_time = Instant::now();

    println!("═══════════════════════════════════════════════");
    println!("  RISC Zero Double Auction Proof Generator");
    println!("═══════════════════════════════════════════════\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut scenario_file = "auction_scenario.json";
    let mut benchmark_mode = false;
    let mut benchmark_output = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--benchmark" => {
                benchmark_mode = true;
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    benchmark_output = args[i + 1].clone();
                    i += 1;
                }
            }
            arg if !arg.starts_with("--") => {
                scenario_file = arg;
            }
            _ => {}
        }
        i += 1;
    }

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

    // Run the prover with timing
    println!("▸ Generating RISC Zero proof...");
    let exec_start = Instant::now();
    let prover = default_prover();
    let opts = ProverOpts::succinct();

    let prove_info = prover
        .prove_with_opts(env, DOUBLE_AUCTION_GUEST_ELF, &opts)
        .expect("Failed to generate proof");

    let proving_time = exec_start.elapsed();
    let receipt = prove_info.receipt;
    println!("✓ Proof generated\n");

    // Extract execution statistics
    let session_info = &prove_info.stats;
    let user_cycles = session_info.user_cycles;
    let total_cycles = session_info.total_cycles;
    let segments = session_info.segments.len();

    if benchmark_mode {
        println!("▸ Benchmark Metrics:");
        println!("  User Cycles: {}", user_cycles);
        println!("  Total Cycles: {} (padded to power of 2)", total_cycles);
        println!("  Segments: {}", segments);
        println!("  Proving Time: {:?}\n", proving_time);
    }

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
    let journal_size = journal_json.len();
    fs::write("journal.json", &journal_json).expect("Failed to write journal");
    println!("✓ Saved journal.json");

    // Save benchmark results if in benchmark mode
    if benchmark_mode {
        let total_time = start_time.elapsed();
        let receipt_json = serde_json::to_string(&receipt).expect("Failed to serialize receipt");

        let benchmark_result = BenchmarkResult {
            participant_count: scenario.participants.len(),
            scenario_name: scenario.scenario_name.clone(),
            user_cycles,
            total_cycles,
            session_segments: segments,
            executor_time_ms: 0, // Not separately tracked in this implementation
            proving_time_ms: proving_time.as_millis() as u64,
            total_time_ms: total_time.as_millis() as u64,
            receipt_size_bytes: receipt_json.len(),
            journal_size_bytes: journal_size,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let benchmark_json = serde_json::to_string_pretty(&benchmark_result)
            .expect("Failed to serialize benchmark result");

        if !benchmark_output.is_empty() {
            fs::write(&benchmark_output, &benchmark_json)
                .expect("Failed to write benchmark results");
            println!("✓ Saved benchmark results to {}", benchmark_output);
        } else {
            println!("\n▸ Benchmark Results (JSON):");
            println!("{}", benchmark_json);
        }
    }

    println!("\n✓ RISC Zero proof generation complete");
}

fn load_scenario(filename: &str) -> Result<AuctionScenario, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(filename)?;
    let scenario: AuctionScenario = serde_json::from_str(&content)?;
    Ok(scenario)
}
