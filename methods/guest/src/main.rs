// ═══════════════════════════════════════════════════════════════════════════
// RISC Zero Guest Program - Modular Auction Implementation
// ═══════════════════════════════════════════════════════════════════════════
//
// ARCHITECTURE:
//   This program executes inside the RISC Zero zkVM to compute auction results
//   with cryptographic proof. Developers can implement custom auction algorithms
//   by modifying only the MODULAR ALGORITHM section below.
//
// MODULARITY POINT:
//   Replace run_double_auction() function body (line 107) with your algorithm.
//
// CONSTRAINTS:
//   • Input:  AuctionInput  (participants with bids/balances)
//   • Output: PublicJournal (sorted: buyers DESC, sellers ASC by price)
//   • Law:    Σ in_coin == Σ out_coin, Σ in_energy == Σ out_energy
//   • Must be deterministic (no external I/O, randomness, or time)
//
// EXAMPLES OF ALTERNATIVE ALGORITHMS:
//   • Vickrey auction (second-price sealed bid)
//   • Dutch auction (descending price)
//   • Combinatorial auction (bundle bidding)
//   • Continuous double auction (time-priority matching)
//
// ═══════════════════════════════════════════════════════════════════════════

use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES (DO NOT MODIFY)
// ═══════════════════════════════════════════════════════════════════════════

/// Participant in the auction (received from host)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub id: u32,        // Unique ID (0..N-1)
    pub role: u32,      // 0=BUY, 1=SELL
    pub price: u64,     // Bid (buyers) or Ask (sellers)
    pub quantity: u64,  // Desired trade amount
    pub in_coin: u64,   // Initial coin balance
    pub in_energy: u64, // Initial energy balance
}

/// Input to the auction algorithm
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuctionInput {
    pub participants: Vec<Participant>,
}

/// Output journal committed to zkVM receipt
/// CRITICAL: Arrays must be sorted [buyers DESC by price, sellers ASC by price]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicJournal {
    pub in_coin: Vec<u64>,      // Input balances (protocol order)
    pub in_energy: Vec<u64>,    // Input balances (protocol order)
    pub out_coin: Vec<u64>,     // Output balances (YOUR ALGORITHM)
    pub out_energy: Vec<u64>,   // Output balances (YOUR ALGORITHM)
}

// ═══════════════════════════════════════════════════════════════════════════
// ENTRY POINT (DO NOT MODIFY)
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    let auction_input: AuctionInput = env::read();
    let journal = run_double_auction(&auction_input);
    env::commit(&journal);
}

// ═══════════════════════════════════════════════════════════════════════════
// MODULAR ALGORITHM INTERFACE
// ═══════════════════════════════════════════════════════════════════════════
//
// DEVELOPER CUSTOMIZATION POINT:
//   Replace the function body below with your custom auction algorithm.
//   Keep the function signature unchanged.
//
// INPUT:
//   • participants: Vec<Participant> with id, role, price, quantity, balances
//
// OUTPUT:
//   • PublicJournal with computed allocations
//   • MUST preserve protocol ordering (buyers DESC, sellers ASC by price)
//   • MUST satisfy conservation: Σ in == Σ out (both coin and energy)
//
// CURRENT IMPLEMENTATION:
//   Uniform-price double auction with marginal pricing
//
// ═══════════════════════════════════════════════════════════════════════════

fn run_double_auction(input: &AuctionInput) -> PublicJournal {
    let participants = &input.participants;

    // Separate and sort participants (protocol requirement)
    let mut buyers: Vec<&Participant> = participants
        .iter()
        .filter(|p| p.role == 0)
        .collect();
    let mut sellers: Vec<&Participant> = participants
        .iter()
        .filter(|p| p.role == 1)
        .collect();

    // Sort: buyers DESC by price, sellers ASC by price (ties broken by ID)
    buyers.sort_by(|a, b| match b.price.cmp(&a.price) {
        core::cmp::Ordering::Equal => a.id.cmp(&b.id),
        other => other,
    });
    sellers.sort_by(|a, b| match a.price.cmp(&b.price) {
        core::cmp::Ordering::Equal => a.id.cmp(&b.id),
        other => other,
    });

    // ─────────────────────────────────────────────────────────────────────────
    // AUCTION ALGORITHM (CUSTOMIZE THIS SECTION)
    // ─────────────────────────────────────────────────────────────────────────

    let clearing_result = find_clearing_price(&buyers, &sellers);

    let (clearing_price, allocations) = match clearing_result {
        Some(result) => result,
        None => return build_journal(participants, &buyers, &sellers), // No trade
    };

    let mut outputs = compute_outputs(participants, &allocations, clearing_price);

    // ─────────────────────────────────────────────────────────────────────────
    // END CUSTOMIZABLE SECTION
    // ─────────────────────────────────────────────────────────────────────────

    // Format journal in protocol order (DO NOT MODIFY)
    build_journal_with_outputs(participants, &buyers, &sellers, &mut outputs)
}

// ═══════════════════════════════════════════════════════════════════════════
// REFERENCE IMPLEMENTATION: Uniform-Price Double Auction
// ═══════════════════════════════════════════════════════════════════════════
//
// The functions below implement the default auction mechanism.
// You may DELETE or REPLACE these when implementing a custom algorithm.
//
// ALGORITHM:
//   1. Find clearing price p* where supply(p) >= demand(p)
//   2. Set price = (marginal_bid + marginal_ask) / 2
//   3. Allocate based on effective caps (min of quantity, balance constraint)
//   4. Pro-rata allocation on binding side
//
// ═══════════════════════════════════════════════════════════════════════════

/// Find uniform clearing price using supply-demand crossing
///
/// Returns: Option<(clearing_price, Vec<(participant_id, allocation)>)>
fn find_clearing_price(
    buyers: &[&Participant],
    sellers: &[&Participant],
) -> Option<(u64, Vec<(u32, u64)>)> {
    if buyers.is_empty() || sellers.is_empty() {
        return None;
    }

    // Build price grid from all bids and asks
    let mut prices: Vec<u64> = buyers.iter().map(|b| b.price).collect();
    prices.extend(sellers.iter().map(|s| s.price));
    prices.sort();
    prices.dedup();

    // Find p* where supply >= demand
    let mut p_star = None;
    for &p in &prices {
        let (demand, supply) = demand_supply_at(buyers, sellers, p);
        if supply >= demand {
            p_star = Some(p);
            break;
        }
    }
    let p_star = p_star?;

    // Identify qualified participants at p*
    let qualified_buyers: Vec<&Participant> = buyers
        .iter()
        .copied()
        .filter(|b| b.price >= p_star)
        .collect();
    let qualified_sellers: Vec<&Participant> = sellers
        .iter()
        .copied()
        .filter(|s| s.price <= p_star)
        .collect();

    if qualified_buyers.is_empty() || qualified_sellers.is_empty() {
        return None;
    }

    // Marginal pricing: average of lowest buyer and highest seller
    let b_marg = qualified_buyers.last().unwrap().price;
    let a_marg = qualified_sellers.last().unwrap().price;
    let clearing_price = (b_marg + a_marg) / 2;

    if clearing_price == 0 {
        return None;
    }

    // Compute effective caps (budget and inventory constraints)
    let mut buyer_caps: Vec<(u32, u64)> = Vec::new();
    let mut seller_caps: Vec<(u32, u64)> = Vec::new();

    let mut eff_demand = 0u64;
    for buyer in &qualified_buyers {
        let afford = buyer.in_coin / clearing_price;
        let cap = buyer.quantity.min(afford);
        buyer_caps.push((buyer.id, cap));
        eff_demand += cap;
    }

    let mut eff_supply = 0u64;
    for seller in &qualified_sellers {
        let cap = seller.quantity.min(seller.in_energy);
        seller_caps.push((seller.id, cap));
        eff_supply += cap;
    }

    let traded_total = eff_demand.min(eff_supply);
    if traded_total == 0 {
        return Some((clearing_price, Vec::new()));
    }

    // Allocate based on binding constraint
    let mut allocations: Vec<(u32, u64)> = Vec::new();

    if eff_demand >= eff_supply {
        // Supply-constrained: fill all sellers, allocate buyers by priority
        for (id, cap) in seller_caps {
            allocations.push((id, cap));
        }

        let mut remaining = traded_total;
        for buyer in &qualified_buyers {
            if remaining == 0 {
                break;
            }
            let cap = buyer_caps
                .iter()
                .find(|(id, _)| *id == buyer.id)
                .map(|(_, cap)| *cap)
                .unwrap_or(0);
            let take = cap.min(remaining);
            if take > 0 {
                allocations.push((buyer.id, take));
                remaining -= take;
            }
        }
    } else {
        // Demand-constrained: fill all buyers, allocate sellers by priority
        for (id, cap) in buyer_caps {
            allocations.push((id, cap));
        }

        let mut remaining = traded_total;
        for seller in &qualified_sellers {
            if remaining == 0 {
                break;
            }
            let cap = seller_caps
                .iter()
                .find(|(id, _)| *id == seller.id)
                .map(|(_, cap)| *cap)
                .unwrap_or(0);
            let take = cap.min(remaining);
            if take > 0 {
                allocations.push((seller.id, take));
                remaining -= take;
            }
        }
    }

    Some((clearing_price, allocations))
}

/// Compute aggregate demand and supply at given price
fn demand_supply_at(buyers: &[&Participant], sellers: &[&Participant], price: u64) -> (u64, u64) {
    let demand: u64 = buyers
        .iter()
        .filter(|b| b.price >= price)
        .map(|b| b.quantity)
        .sum();

    let supply: u64 = sellers
        .iter()
        .filter(|s| s.price <= price)
        .map(|s| s.quantity)
        .sum();

    (demand, supply)
}

/// Apply allocations to compute final balances
///
/// Returns: Vec<(participant_id, out_coin, out_energy)>
fn compute_outputs(
    participants: &[Participant],
    allocations: &[(u32, u64)],
    clearing_price: u64,
) -> Vec<(u32, u64, u64)> {
    let mut outputs = Vec::new();

    for p in participants {
        let allocated = allocations
            .iter()
            .find(|(id, _)| *id == p.id)
            .map(|(_, amount)| *amount)
            .unwrap_or(0);

        let (out_coin, out_energy) = if p.role == 0 {
            // BUY: spend coins, receive energy
            if allocated > 0 {
                (
                    p.in_coin - (clearing_price * allocated),
                    p.in_energy + allocated,
                )
            } else {
                (p.in_coin, p.in_energy)
            }
        } else {
            // SELL: receive coins, spend energy
            if allocated > 0 {
                (
                    p.in_coin + (clearing_price * allocated),
                    p.in_energy - allocated,
                )
            } else {
                (p.in_coin, p.in_energy)
            }
        };

        outputs.push((p.id, out_coin, out_energy));
    }

    outputs
}

// ═══════════════════════════════════════════════════════════════════════════
// PROTOCOL INFRASTRUCTURE (DO NOT MODIFY)
// ═══════════════════════════════════════════════════════════════════════════

/// Build journal with no trades (fallback for no market clearing)
fn build_journal(
    participants: &[Participant],
    buyers: &[&Participant],
    sellers: &[&Participant],
) -> PublicJournal {
    let mut outputs_vec: Vec<(u32, u64, u64)> = participants
        .iter()
        .map(|p| (p.id, p.in_coin, p.in_energy))
        .collect();

    build_journal_with_outputs(participants, buyers, sellers, &mut outputs_vec)
}

/// Build journal in protocol order: buyers (DESC price) then sellers (ASC price)
///
/// CRITICAL: This ordering is required for circuit verification. Do not modify.
fn build_journal_with_outputs(
    _participants: &[Participant],
    buyers_sorted: &[&Participant],
    sellers_sorted: &[&Participant],
    outputs: &mut [(u32, u64, u64)],
) -> PublicJournal {
    use std::collections::BTreeMap;

    // Index outputs by participant ID
    let mut output_map: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
    for (id, coin, energy) in outputs {
        output_map.insert(*id, (*coin, *energy));
    }

    let mut in_coin = Vec::new();
    let mut in_energy = Vec::new();
    let mut out_coin = Vec::new();
    let mut out_energy = Vec::new();

    // Buyers first (descending by price)
    for buyer in buyers_sorted {
        in_coin.push(buyer.in_coin);
        in_energy.push(buyer.in_energy);

        let default = (buyer.in_coin, buyer.in_energy);
        let (oc, oe) = output_map.get(&buyer.id).unwrap_or(&default);
        out_coin.push(*oc);
        out_energy.push(*oe);
    }

    // Sellers second (ascending by price)
    for seller in sellers_sorted {
        in_coin.push(seller.in_coin);
        in_energy.push(seller.in_energy);

        let default = (seller.in_coin, seller.in_energy);
        let (oc, oe) = output_map.get(&seller.id).unwrap_or(&default);
        out_coin.push(*oc);
        out_energy.push(*oe);
    }

    PublicJournal {
        in_coin,
        in_energy,
        out_coin,
        out_energy,
    }
}