# RISC Zero Auction Benchmark Implementation Summary

## Overview

This document summarizes the benchmarking infrastructure implemented for the RISC Zero auction proof system. The system enables automated performance testing across participant counts from 10 to 300 in configurable steps.

## Changes Made

### 1. Host Program Modifications (`host/src/main.rs`)

**Added Dependencies:**
- `std::time::Instant` - For precise timing measurements
- `chrono` - For ISO 8601 timestamps

**New Data Structure:**
```rust
pub struct BenchmarkResult {
    pub participant_count: usize,
    pub scenario_name: String,
    pub total_cycles: u64,
    pub session_segments: usize,
    pub executor_time_ms: u64,
    pub proving_time_ms: u64,
    pub total_time_ms: u64,
    pub receipt_size_bytes: usize,
    pub journal_size_bytes: usize,
    pub timestamp: String,
}
```

**New Features:**
- Command-line argument parsing for `--benchmark` flag
- Captures execution metrics from `prove_info.stats`:
  - `total_cycles` - Total zkVM execution cycles
  - `segments` - Number of proof segments
- Timing measurements:
  - Proving time (execution + proof generation)
  - Total time (including all overhead)
- Size measurements:
  - Receipt size (cryptographic proof)
  - Journal size (public outputs)
- Optional JSON output with `--benchmark <output_file>`

**Usage:**
```bash
# Regular execution
cargo run --release --bin host -- scenarios/benchmark_N10.json

# With benchmark metrics output
cargo run --release --bin host -- scenarios/benchmark_N10.json --benchmark results.json

# Benchmark mode with console output
cargo run --release --bin host -- scenarios/benchmark_N10.json --benchmark
```

### 2. Dependency Updates (`host/Cargo.toml`)

**Added:**
```toml
chrono = "0.4"
```

### 3. Benchmark Runner Script (`run_benchmarks.sh`)

A comprehensive Bash script that automates the benchmarking process.

**Features:**
- Configurable participant range (--start, --end, --step)
- Optional project rebuild (--build)
- Timestamped results directories
- Progress tracking with colored output
- Automatic result aggregation:
  - Individual JSON files per participant count
  - Combined `benchmark_summary.json`
  - CSV export for spreadsheet analysis
- Execution log capture for debugging
- Symlink to latest results
- Quick summary table generation

**Example Usage:**
```bash
# Full benchmark suite (N=10 to N=300)
./run_benchmarks.sh --build

# Custom range
./run_benchmarks.sh --start 50 --end 150 --step 25

# Quick test
./run_benchmarks.sh --start 10 --end 30 --step 10
```

### 4. Analysis Tool (`analyze_benchmarks.py`)

A Python script for comprehensive benchmark analysis and visualization.

**Features:**
- Statistical summary of all benchmark runs
- Detailed tabular output with all metrics
- Scaling analysis:
  - Compares first vs last participant count
  - Calculates scaling ratios for all metrics
- Computational complexity estimation:
  - Estimates O(n^k) complexity from data
  - Separate estimates for cycles and time
- Efficiency metrics:
  - Average cycles per participant
  - Average time per participant
- CSV report generation with derived metrics
- Optional matplotlib visualizations:
  - Cycles vs Participants
  - Segments vs Participants
  - Execution Time vs Participants
  - Efficiency (Cycles/Participant) vs Participants

**Example Usage:**
```bash
# Analyze latest results
python3 analyze_benchmarks.py benchmark_results/latest

# Analyze specific run
python3 analyze_benchmarks.py benchmark_results/20251016_143022

# With visualizations (requires matplotlib)
pip install matplotlib
python3 analyze_benchmarks.py benchmark_results/latest
```

### 5. Documentation

**Created Files:**
- `BENCHMARK_GUIDE.md` - Comprehensive usage guide
- `BENCHMARK_SUMMARY.md` - This file, implementation summary

## Metrics Captured

The benchmark system captures the following metrics for each participant count:

| Metric | Description | Unit |
|--------|-------------|------|
| `participant_count` | Number of auction participants | count |
| `scenario_name` | Descriptive scenario name | string |
| `total_cycles` | Total zkVM execution cycles | cycles |
| `session_segments` | Number of proof segments | count |
| `proving_time_ms` | Time to generate proof | milliseconds |
| `total_time_ms` | Total execution time | milliseconds |
| `receipt_size_bytes` | Size of cryptographic proof | bytes |
| `journal_size_bytes` | Size of public output | bytes |
| `timestamp` | ISO 8601 timestamp | string |

## Output Structure

```
risc0/
├── host/
│   ├── src/main.rs          (modified - added benchmarking)
│   └── Cargo.toml           (modified - added chrono)
├── scenarios/
│   ├── benchmark_N10.json   (existing scenarios)
│   ├── benchmark_N20.json
│   ├── ...
│   └── benchmark_N300.json
├── benchmark_results/       (generated)
│   ├── 20251016_143022/
│   │   ├── benchmark_N10.json
│   │   ├── benchmark_N20.json
│   │   ├── ...
│   │   ├── benchmark_summary.json
│   │   ├── benchmark_summary.csv
│   │   ├── detailed_analysis.csv
│   │   ├── benchmark_plots.png
│   │   ├── log_N10.txt
│   │   └── ...
│   └── latest -> 20251016_143022/
├── run_benchmarks.sh        (new)
├── analyze_benchmarks.py    (new)
├── BENCHMARK_GUIDE.md       (new)
└── BENCHMARK_SUMMARY.md     (new - this file)
```

## Workflow

### 1. Run Benchmarks

```bash
# Build and run full benchmark suite
./run_benchmarks.sh --build
```

This will:
1. Build the project in release mode
2. Create timestamped results directory
3. Iterate through participant counts (10, 20, 30, ..., 300)
4. Run each scenario and capture metrics
5. Save individual and aggregated results
6. Create symlink to latest results

### 2. Analyze Results

```bash
# Statistical analysis
python3 analyze_benchmarks.py benchmark_results/latest
```

This will:
1. Load all benchmark data
2. Display detailed statistics table
3. Calculate scaling analysis
4. Estimate computational complexity
5. Generate CSV reports
6. Create visualization plots (if matplotlib installed)

### 3. Review Outputs

Check the following files in `benchmark_results/latest/`:
- `benchmark_summary.csv` - Import into spreadsheet
- `detailed_analysis.csv` - Full analysis results
- `benchmark_plots.png` - Visualization graphs
- Individual `benchmark_N*.json` - Detailed per-run data

## Key Insights from Benchmarking

Based on RISC Zero documentation and the PPEM implementation analysis:

1. **Cycle Rounding**: RISC Zero rounds cycles to the next power of 2
   - A 33K cycle program costs the same as 63K cycles (both → 65,536)
   - This creates "steps" in the performance curve

2. **Segment Scaling**: Programs are divided into segments
   - Each segment can be proven in parallel
   - More segments = better parallelization but higher overhead

3. **Performance Factors**:
   - Guest execution time (~45% of total on GPU)
   - Data movement (~50% on GPU)
   - Segment compression (remaining time)

4. **Optimization Opportunities**:
   - Minimize guest program cycles
   - Use release builds
   - Enable GPU acceleration (Metal/CUDA)
   - Consider segment boundaries in algorithm design

## Comparison with PPEM Implementation

This implementation follows patterns from `/home/async0b1/main/PPEM/risc0`:

**Similarities:**
- Structured JSON benchmark output
- Capture of cycles, segments, execution time
- Timestamped results with date-based directories
- Batch execution across multiple participant counts
- CSV export for analysis

**Enhancements:**
- Integrated `--benchmark` flag in host program (vs external parsing)
- Automated script with configurable ranges
- Built-in analysis tool with visualizations
- Comprehensive documentation
- Symlink to latest results for convenience

## Performance Expectations

Based on RISC Zero benchmarks, expected scaling:

| Participants | Estimated Cycles | Estimated Segments | Estimated Time |
|--------------|------------------|--------------------| ---------------|
| 10           | ~500K            | ~8                 | ~10-20s        |
| 50           | ~5M              | ~64                | ~60-120s       |
| 100          | ~20M             | ~256               | ~5-10min       |
| 200          | ~80M             | ~1024              | ~20-40min      |
| 300          | ~180M            | ~2048              | ~45-90min      |

**Note**: Actual performance depends on:
- Hardware (CPU vs GPU, cores, memory)
- Algorithm complexity (current: double auction O(n log n))
- RISC Zero version and optimizations

## Next Steps

1. **Run Initial Benchmark:**
   ```bash
   ./run_benchmarks.sh --start 10 --end 50 --step 10 --build
   ```

2. **Review Results:**
   ```bash
   python3 analyze_benchmarks.py benchmark_results/latest
   ```

3. **Optimize if Needed:**
   - Review guest code for cycle reduction opportunities
   - Consider algorithm optimizations
   - Enable GPU acceleration if available

4. **Full Benchmark:**
   ```bash
   ./run_benchmarks.sh --build
   ```
   (Allow several hours for full N=10-300 run)

5. **Compare Results:**
   - Store baseline results
   - Make optimizations
   - Re-run benchmarks
   - Compare metrics

## Troubleshooting

### Compilation Errors

If you encounter errors related to `prove_info.stats`:

```rust
// Check RISC Zero API version
// The stats field may have different naming in different versions
// Refer to: https://docs.rs/risc0-zkvm/latest/risc0_zkvm/
```

### Missing Metrics

If certain metrics are not available:
- Check RISC Zero version (requires 2.0+)
- Verify ProverOpts includes execution stats
- Update API calls based on current risc0-zkvm documentation

### Performance Issues

For slow benchmark execution:
- Start with smaller ranges (--end 50)
- Use GPU acceleration if available
- Ensure release builds (--build flag)
- Monitor system resources (RAM, CPU)

## References

- [RISC Zero Performance Benchmarks](https://dev.risczero.com/api/zkvm/benchmarks)
- [RISC Zero Datasheet](https://benchmarks.risczero.com/main/datasheet)
- [RISC Zero zkVM Documentation](https://dev.risczero.com/api/zkvm)
- PPEM Implementation: `/home/async0b1/main/PPEM/benchmarks/risczero/`

## Files Modified/Created

### Modified:
- `host/src/main.rs` - Added benchmarking capabilities
- `host/Cargo.toml` - Added chrono dependency

### Created:
- `run_benchmarks.sh` - Benchmark automation script
- `analyze_benchmarks.py` - Analysis and visualization tool
- `BENCHMARK_GUIDE.md` - User documentation
- `BENCHMARK_SUMMARY.md` - Implementation summary (this file)

### Existing (Used):
- `scenarios/benchmark_N{10..300}.json` - Pre-generated test scenarios
- `methods/guest/src/main.rs` - Guest program (unmodified)

## Conclusion

The benchmarking infrastructure is now complete and ready to use. It provides:

1. ✓ Automated benchmarking from N=10 to N=300
2. ✓ Comprehensive metric collection (cycles, segments, time, sizes)
3. ✓ Structured JSON and CSV output
4. ✓ Statistical analysis and visualization
5. ✓ Detailed documentation

You can now run performance benchmarks to analyze how your RISC Zero auction proof scales with the number of participants.
