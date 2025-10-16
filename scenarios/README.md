# Auction Scenarios

This directory contains test scenario files for the double auction mechanism.

## Scenario Format

```json
{
  "scenario_name": "Descriptive name",
  "description": "Detailed description",
  "participants": [
    {
      "id": 0,
      "role": 0,           // 0 = BUY, 1 = SELL
      "price": 100,        // Bid for buyers, ask for sellers
      "quantity": 50,      // Desired trade amount
      "in_coin": 2000,     // Initial coin balance
      "in_energy": 0       // Initial energy balance
    },
    ...
  ]
}
```

## Fields Explanation

**Input fields** (required):
- **id**: Participant identifier (0 to N-1)
- **role**: 0 for buyer, 1 for seller
- **price**: Maximum willing to pay (buyers) or minimum willing to accept (sellers)
- **quantity**: Amount of energy to trade
- **in_coin**: Starting coin balance
- **in_energy**: Starting energy balance

**Output fields** (computed by RISC Zero, NOT in scenario file):
- **out_coin**: Final coin balance (in journal output)
- **out_energy**: Final energy balance (in journal output)

## Test Scenarios

### **auction_N10.json**
- 10 participants (5 buyers, 5 sellers)
- Balanced market
- Expected clearing price: ~77-80
- Expected trade: ~150 units

### Creating Custom Scenarios

1. Copy a template file
2. Adjust number of participants
3. Set realistic price ranges:
   - Buyers: high to low prices
   - Sellers: low to high prices
4. Ensure buyers have sufficient coins
5. Ensure sellers have sufficient energy
6. **Do NOT include** out_coin/out_energy (they're computed by RISC Zero)

## Testing

Test a scenario:
```bash
cd /home/async0b1/protocol_prod
./utils/test_pipeline.sh scenarios/auction_N10.json
```

Expected output:
- risc0/risc0_receipt.json
- risc0/journal.json
- circom/circom_data/proof.json
- circom/circom_data/public_signals.json
