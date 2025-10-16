#!/bin/bash

# ═══════════════════════════════════════════════════════════════════════════
# RISC Zero Auction Benchmark Runner
# ═══════════════════════════════════════════════════════════════════════════
#
# This script runs benchmarks for auction scenarios from 10 to 300 participants
# in steps of 10, collecting performance metrics for each run.
#
# Usage:
#   ./run_benchmarks.sh [options]
#
# Options:
#   --start N      Start from N participants (default: 10)
#   --end N        End at N participants (default: 300)
#   --step N       Step size (default: 10)
#   --output DIR   Output directory for results (default: ./benchmark_results)
#   --build        Build the project before running (default: skip)
#
# Output:
#   - Individual JSON files for each participant count
#   - Combined results in benchmark_summary.json
#   - CSV format for easy analysis
#
# ═══════════════════════════════════════════════════════════════════════════

set -e  # Exit on error

# Default parameters
START_N=10
END_N=300
STEP_N=10
OUTPUT_DIR="benchmark_results"
BUILD=false
SCENARIOS_DIR="scenarios"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --start)
            START_N="$2"
            shift 2
            ;;
        --end)
            END_N="$2"
            shift 2
            ;;
        --step)
            STEP_N="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --build)
            BUILD=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--start N] [--end N] [--step N] [--output DIR] [--build]"
            exit 1
            ;;
    esac
done

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "═══════════════════════════════════════════════════════════════"
echo "  RISC Zero Auction Benchmark Suite"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Configuration:"
echo "  Participant range: ${START_N} to ${END_N} (step: ${STEP_N})"
echo "  Output directory: ${OUTPUT_DIR}"
echo "  Scenarios directory: ${SCENARIOS_DIR}"
echo ""

# Build the project if requested
if [ "$BUILD" = true ]; then
    echo -e "${BLUE}▸ Building project...${NC}"
    cargo build --release
    echo -e "${GREEN}✓ Build complete${NC}"
    echo ""
fi

# Create output directory with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RUN_DIR="${OUTPUT_DIR}/${TIMESTAMP}"
mkdir -p "$RUN_DIR"

echo -e "${BLUE}▸ Results will be saved to: ${RUN_DIR}${NC}"
echo ""

# Initialize summary arrays
declare -a ALL_RESULTS=()

# Run benchmarks
TOTAL_RUNS=$(( (END_N - START_N) / STEP_N + 1 ))
CURRENT_RUN=0

for N in $(seq $START_N $STEP_N $END_N); do
    CURRENT_RUN=$((CURRENT_RUN + 1))
    SCENARIO_FILE="${SCENARIOS_DIR}/benchmark_N${N}.json"
    OUTPUT_FILE="${RUN_DIR}/benchmark_N${N}.json"

    echo -e "${YELLOW}▸ [$CURRENT_RUN/$TOTAL_RUNS] Running benchmark for N=${N}...${NC}"

    # Check if scenario file exists
    if [ ! -f "$SCENARIO_FILE" ]; then
        echo -e "${RED}✗ Scenario file not found: ${SCENARIO_FILE}${NC}"
        echo -e "${RED}  Skipping N=${N}${NC}"
        echo ""
        continue
    fi

    # Run the benchmark
    if cargo run --release --bin host -- "$SCENARIO_FILE" --benchmark "$OUTPUT_FILE" > "${RUN_DIR}/log_N${N}.txt" 2>&1; then
        echo -e "${GREEN}✓ Completed N=${N}${NC}"

        # Extract key metrics for display
        if [ -f "$OUTPUT_FILE" ]; then
            USER_CYCLES=$(jq -r '.user_cycles' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            TOTAL_CYCLES=$(jq -r '.total_cycles' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            SEGMENTS=$(jq -r '.session_segments' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            TIME_MS=$(jq -r '.total_time_ms' "$OUTPUT_FILE" 2>/dev/null || echo "N/A")
            echo -e "  User Cycles: ${USER_CYCLES}, Total Cycles: ${TOTAL_CYCLES}, Segments: ${SEGMENTS}, Time: ${TIME_MS}ms"
        fi
    else
        echo -e "${RED}✗ Failed N=${N}${NC}"
        echo -e "${RED}  Check log: ${RUN_DIR}/log_N${N}.txt${NC}"
    fi

    echo ""
done

echo -e "${BLUE}▸ Generating summary reports...${NC}"

# Create combined JSON summary
jq -s '.' ${RUN_DIR}/benchmark_N*.json > "${RUN_DIR}/benchmark_summary.json" 2>/dev/null || true

# Create CSV summary
CSV_FILE="${RUN_DIR}/benchmark_summary.csv"
echo "participant_count,scenario_name,user_cycles,total_cycles,session_segments,proving_time_ms,total_time_ms,receipt_size_bytes,journal_size_bytes,timestamp" > "$CSV_FILE"

for N in $(seq $START_N $STEP_N $END_N); do
    JSON_FILE="${RUN_DIR}/benchmark_N${N}.json"
    if [ -f "$JSON_FILE" ]; then
        jq -r '[.participant_count, .scenario_name, .user_cycles, .total_cycles, .session_segments, .proving_time_ms, .total_time_ms, .receipt_size_bytes, .journal_size_bytes, .timestamp] | @csv' "$JSON_FILE" >> "$CSV_FILE" 2>/dev/null || true
    fi
done

echo -e "${GREEN}✓ Summary saved to:${NC}"
echo -e "  ${RUN_DIR}/benchmark_summary.json"
echo -e "  ${RUN_DIR}/benchmark_summary.csv"
echo ""

# Create symlink to latest results
ln -sfn "${TIMESTAMP}" "${OUTPUT_DIR}/latest"
echo -e "${GREEN}✓ Latest results symlink: ${OUTPUT_DIR}/latest${NC}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo -e "${GREEN}  Benchmark suite complete!${NC}"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Quick analysis:"
if [ -f "${RUN_DIR}/benchmark_summary.csv" ]; then
    echo ""
    echo "Participant Count | User Cycles | Total Cycles | Segments | Time (ms)"
    echo "------------------|-------------|--------------|----------|----------"
    tail -n +2 "${RUN_DIR}/benchmark_summary.csv" | while IFS=, read -r count name user_cycles total_cycles segments prov_time total_time receipt journal ts; do
        # Remove quotes from CSV fields
        count=$(echo $count | tr -d '"')
        user_cycles=$(echo $user_cycles | tr -d '"')
        total_cycles=$(echo $total_cycles | tr -d '"')
        segments=$(echo $segments | tr -d '"')
        total_time=$(echo $total_time | tr -d '"')
        printf "%-17s | %-11s | %-12s | %-8s | %-10s\n" "$count" "$user_cycles" "$total_cycles" "$segments" "$total_time"
    done
fi
