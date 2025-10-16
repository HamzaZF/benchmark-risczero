# Custom Auction Algorithm - Developer Guide

## Quick Start

1. Open `src/main_dev.rs` (or `src/main.rs` if using production)
2. Locate `run_double_auction()` function (line 107)
3. Replace function body with your algorithm
4. Test: `cd risc0 && cargo run --release -- auction_scenario.json`

## Input/Output Contracts

### Input: `AuctionInput`

```rust
pub struct AuctionInput {
    pub participants: Vec<Participant>,
}

pub struct Participant {
    pub id: u32,        // 0 to N-1
    pub role: u32,      // 0=BUY, 1=SELL
    pub price: u64,     // Bid or ask
    pub quantity: u64,  // Desired amount
    pub in_coin: u64,   // Initial balance
    pub in_energy: u64, // Initial balance
}
```

### Output: `PublicJournal`

```rust
pub struct PublicJournal {
    pub in_coin: Vec<u64>,      // Input balances
    pub in_energy: Vec<u64>,    // Input balances
    pub out_coin: Vec<u64>,     // YOUR OUTPUTS
    pub out_energy: Vec<u64>,   // YOUR OUTPUTS
}
```

## Critical Requirements

### 1. Ordering
Arrays **must** be sorted as:
- Buyers first (descending by price, ties by ID)
- Sellers second (ascending by price, ties by ID)

### 2. Conservation Law
```
Σ in_coin[i]   == Σ out_coin[i]
Σ in_energy[i] == Σ out_energy[i]
```

### 3. Determinism
Same input **must** produce same output (no randomness, time, or I/O).

### 4. No External Dependencies
Cannot use: file I/O, network, random numbers, system time.

## Example: Custom Algorithm Template

```rust
fn run_double_auction(input: &AuctionInput) -> PublicJournal {
    let participants = &input.participants;

    // 1. Separate and sort (REQUIRED for protocol)
    let (buyers, sellers) = separate_and_sort(participants);

    // 2. YOUR ALGORITHM HERE
    let results = your_custom_auction_logic(&buyers, &sellers);

    // 3. Compute final balances
    let outputs = apply_results_to_balances(participants, results);

    // 4. Format journal (REQUIRED for protocol)
    build_journal_with_outputs(participants, &buyers, &sellers, &outputs)
}
```

## Alternative Auction Mechanisms

### Vickrey Auction (Second-Price)
```rust
// Winner pays second-highest bid
let winning_bid = buyers[0].price;
let clearing_price = buyers[1].price; // Second price
```

### Dutch Auction
```rust
// Start high, descend until match
let start_price = buyers.iter().map(|b| b.price).max()?;
for price in (0..=start_price).rev() {
    if has_match_at(price) {
        return allocate_at(price);
    }
}
```

### Posted Price
```rust
// Fixed price, FCFS allocation
const POSTED_PRICE: u64 = 100;
let allocations = allocate_fcfs(buyers, sellers, POSTED_PRICE);
```

## Testing Your Algorithm

### 1. Generate Test Scenario
```bash
python3 generate_auction_scenario.py 10 -o test_scenario.json
```

### 2. Run RISC Zero
```bash
cd risc0
cargo run --release -- ../scenarios/test_scenario.json
```

### 3. Verify Output
```bash
cat journal.json
```

Check:
- Conservation: `Σ in == Σ out`
- Ordering: buyers first, sellers second
- Logic: allocations match your algorithm

## Common Pitfalls

| Issue | Solution |
|-------|----------|
| Wrong ordering | Use provided sort functions |
| Value creation | Double-check conservation law |
| Non-determinism | Remove random/time dependencies |
| Overflow | Use `.checked_add()`, `.checked_mul()` |

## Helper Functions

The reference implementation provides:

- `find_clearing_price()` - Supply-demand equilibrium
- `demand_supply_at()` - Aggregate at price
- `compute_outputs()` - Apply allocations
- `build_journal_with_outputs()` - Format for protocol

You can **delete** these when implementing custom logic.

## Support

For questions:
1. Review main.rs comments (modular sections marked with ⭐)
2. Check RISC0_INTEGRATION_GUIDE.md in project root
3. Test with small N first (N=2 or N=10)