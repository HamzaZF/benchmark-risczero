# RISC Zero Auction Benchmark Guide

This guide explains how to run performance benchmarks for the RISC Zero auction proof system with varying numbers of participants (10-300).

## Overview

The benchmark suite measures key performance metrics as the number of auction participants scales:

- **Total Cycles**: Computational cost in zkVM cycles
- **Segments**: Number of proof segments generated
- **Execution Time**: Total time to generate the proof
- **Proof Size**: Size of the generated receipt and journal

## Quick Start

### 1. Build the Project

```bash
cargo build --release
```

### 2. Run Benchmarks

Run the full benchmark suite (N=10 to N=300, step 10):

```bash
./run_benchmarks.sh --build
```

### 3. Analyze Results

```bash
python3 analyze_benchmarks.py benchmark_results/latest
```

## Benchmark Runner Script

### Usage

```bash
./run_benchmarks.sh [options]
```

### Options

- `--start N` - Start from N participants (default: 10)
- `--end N` - End at N participants (default: 300)
- `--step N` - Step size (default: 10)
- `--output DIR` - Output directory for results (default: ./benchmark_results)
- `--build` - Build the project before running (default: skip)

### Examples

**Run benchmarks for N=10 to N=100:**

```bash
./run_benchmarks.sh --start 10 --end 100 --step 10
```

**Run with custom output directory:**

```bash
./run_benchmarks.sh --output my_results --build
```

**Test a specific range:**

```bash
./run_benchmarks.sh --start 50 --end 150 --step 25
```

## Output Structure

Results are saved in timestamped directories:

```
benchmark_results/
├── 20251016_143022/
│   ├── benchmark_N10.json
│   ├── benchmark_N20.json
│   ├── ...
│   ├── benchmark_N300.json
│   ├── benchmark_summary.json
│   ├── benchmark_summary.csv
│   ├── log_N10.txt
│   ├── log_N20.txt
│   └── ...
└── latest -> 20251016_143022/
```

### File Descriptions

- `benchmark_N{X}.json` - Individual benchmark result for X participants
- `benchmark_summary.json` - Combined results in JSON array format
- `benchmark_summary.csv` - Combined results in CSV format
- `log_N{X}.txt` - Console output from each benchmark run
- `latest/` - Symlink to most recent results

## Benchmark Metrics

Each benchmark run captures the following metrics:

```json
{
  "participant_count": 10,
  "scenario_name": "10 Participant Auction",
  "total_cycles": 524288,
  "session_segments": 8,
  "executor_time_ms": 0,
  "proving_time_ms": 12500,
  "total_time_ms": 12850,
  "receipt_size_bytes": 245760,
  "journal_size_bytes": 462,
  "timestamp": "2025-10-16T14:30:22Z"
}
```

### Metric Definitions

- **participant_count**: Number of participants in the auction
- **scenario_name**: Descriptive name of the scenario
- **total_cycles**: Total zkVM execution cycles (rounded to next power of 2)
- **session_segments**: Number of proof segments generated
- **proving_time_ms**: Time spent generating the proof
- **total_time_ms**: Total execution time including setup
- **receipt_size_bytes**: Size of the RISC Zero receipt
- **journal_size_bytes**: Size of the public journal output
- **timestamp**: ISO 8601 timestamp of the benchmark run

## Analysis Tool

The Python analysis script provides:

1. **Statistical Summary**: Min, max, averages across all runs
2. **Scaling Analysis**: How metrics scale with participant count
3. **Complexity Estimation**: Computational complexity (O(n), O(n²), etc.)
4. **Efficiency Metrics**: Cycles and time per participant
5. **Visualizations**: Plots of key metrics (requires matplotlib)

### Usage

```bash
python3 analyze_benchmarks.py [results_dir]
```

### Example Output

```
================================================================================
RISC Zero Auction Benchmark Analysis
================================================================================

Total benchmark runs: 30
Participant range: 10 to 300

Detailed Results:
------------------------------------------------------------------------------
   N |       Cycles | Segments | Proving (ms) | Total (ms) | Receipt (KB)
------------------------------------------------------------------------------
  10 |      524,288 |        8 |       12,500 |     12,850 |       240.00
  20 |    1,048,576 |       16 |       24,800 |     25,200 |       480.00
  30 |    2,097,152 |       32 |       51,200 |     51,650 |       960.00
 ...
------------------------------------------------------------------------------

Scaling Analysis:
--------------------------------------------------------------------------------
Participants increased by: 30.0x (10 → 300)
Cycles increased by:       128.0x (524,288 → 67,108,864)
Segments increased by:     64.0x (8 → 512)
Time increased by:         98.5x (12,850ms → 1,266,000ms)

Estimated Computational Complexity:
  Cycles: O(n^2.13)
  Time:   O(n^2.05)
```

### Visualization

If matplotlib is installed, the analysis tool generates plots:

```bash
pip install matplotlib
python3 analyze_benchmarks.py benchmark_results/latest
```

This creates `benchmark_plots.png` with:
- Cycles vs Participants
- Segments vs Participants
- Execution Time vs Participants
- Efficiency (Cycles/Participant) vs Participants

## Running Individual Tests

You can also run individual benchmarks manually:

```bash
# Build the project
cargo build --release

# Run a specific scenario
cargo run --release --bin host -- scenarios/benchmark_N100.json --benchmark results_N100.json

# View the results
cat results_N100.json | jq
```

## Interpreting Results

### Cycles

- RISC Zero rounds cycles to the next power of 2
- A program with 33,000 cycles uses the same proving time as 63,000 cycles (both round to 65,536)
- Higher cycle count = more computation in the zkVM

### Segments

- Larger programs are divided into segments for parallel proving
- More segments = more parallelization opportunity but higher overhead
- Each segment has a maximum cycle count

### Proving Time

- Includes both segment generation and compression
- Scales with cycle count and segment count
- GPU acceleration can significantly reduce proving time

### Receipt Size

- The cryptographic proof that can be verified on-chain
- Grows with the number of segments
- Compression reduces size but increases proving time

## Performance Optimization Tips

1. **Profile your guest code** to minimize cycles
2. **Use release builds** (`cargo build --release`)
3. **Enable GPU acceleration** if available (Metal/CUDA)
4. **Batch operations** to amortize proof generation overhead
5. **Consider segment boundaries** when designing algorithms

## Troubleshooting

### Benchmark fails with "scenario file not found"

Ensure scenario files exist in the `scenarios/` directory:

```bash
ls scenarios/benchmark_N*.json
```

### Out of memory errors

Large participant counts (N>200) may require significant RAM. Try:
- Running smaller ranges
- Increasing system swap space
- Using a machine with more RAM

### Slow execution

First runs include compilation overhead. Subsequent runs are faster. For accurate benchmarks:
- Use `--build` before the benchmark suite
- Run each test multiple times and average
- Use release builds

## Advanced Usage

### Custom Scenarios

Create your own scenario files:

```bash
cp scenarios/benchmark_N10.json scenarios/my_scenario.json
# Edit my_scenario.json
cargo run --release --bin host -- scenarios/my_scenario.json --benchmark my_results.json
```

### Automated Testing

Integrate benchmarks into CI/CD:

```bash
#!/bin/bash
./run_benchmarks.sh --start 10 --end 100 --step 10 --output ci_results
python3 analyze_benchmarks.py ci_results/latest > benchmark_report.txt
```

### Comparing Results

```bash
# Run baseline
./run_benchmarks.sh --output baseline

# Make changes to code

# Run comparison
./run_benchmarks.sh --output optimized

# Compare
python3 analyze_benchmarks.py baseline/latest > baseline_report.txt
python3 analyze_benchmarks.py optimized/latest > optimized_report.txt
diff baseline_report.txt optimized_report.txt
```

## References

- [RISC Zero Performance Benchmarks](https://dev.risczero.com/api/zkvm/benchmarks)
- [RISC Zero Datasheet](https://benchmarks.risczero.com/main/datasheet)
- [zkVM Documentation](https://dev.risczero.com/api/zkvm)

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review RISC Zero documentation
3. Check scenario files are properly formatted
4. Verify dependencies are installed
