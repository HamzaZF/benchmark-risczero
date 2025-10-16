#!/usr/bin/env python3
"""
RISC Zero Benchmark Analysis Tool

This script analyzes benchmark results from the RISC Zero auction benchmarks
and generates visualizations and statistical reports.

Usage:
    python3 analyze_benchmarks.py [results_dir]

    results_dir: Path to benchmark results directory (default: benchmark_results/latest)
"""

import json
import sys
import os
from pathlib import Path
from typing import List, Dict, Any
import csv

def load_benchmark_data(results_dir: str) -> List[Dict[str, Any]]:
    """Load all benchmark JSON files from the results directory."""
    summary_file = Path(results_dir) / "benchmark_summary.json"

    if summary_file.exists():
        with open(summary_file, 'r') as f:
            return json.load(f)

    # Fallback: load individual files
    results = []
    for json_file in sorted(Path(results_dir).glob("benchmark_N*.json")):
        with open(json_file, 'r') as f:
            results.append(json.load(f))

    return results

def print_statistics(data: List[Dict[str, Any]]):
    """Print statistical summary of benchmark results."""
    if not data:
        print("No data to analyze")
        return

    print("\n" + "="*80)
    print("RISC Zero Auction Benchmark Analysis")
    print("="*80 + "\n")

    # Overall statistics
    print(f"Total benchmark runs: {len(data)}")
    print(f"Participant range: {min(d['participant_count'] for d in data)} to {max(d['participant_count'] for d in data)}")
    print()

    # Detailed table
    print("Detailed Results:")
    print("-" * 135)
    print(f"{'N':>4} | {'User Cycles':>12} | {'Total Cycles':>13} | {'Segments':>8} | {'Proving (ms)':>12} | {'Total (ms)':>10} | {'Receipt (KB)':>12} | {'Journal (KB)':>12}")
    print("-" * 135)

    for item in sorted(data, key=lambda x: x['participant_count']):
        n = item['participant_count']
        user_cycles = item['user_cycles']
        total_cycles = item['total_cycles']
        segments = item['session_segments']
        proving_ms = item['proving_time_ms']
        total_ms = item['total_time_ms']
        receipt_kb = item['receipt_size_bytes'] / 1024
        journal_kb = item['journal_size_bytes'] / 1024

        print(f"{n:>4} | {user_cycles:>12,} | {total_cycles:>13,} | {segments:>8} | {proving_ms:>12,} | {total_ms:>10,} | {receipt_kb:>12.2f} | {journal_kb:>12.2f}")

    print("-" * 135)
    print()

    # Scaling analysis
    print("Scaling Analysis:")
    print("-" * 80)

    # Compare first and last
    first = min(data, key=lambda x: x['participant_count'])
    last = max(data, key=lambda x: x['participant_count'])

    n_ratio = last['participant_count'] / first['participant_count']
    user_cycle_ratio = last['user_cycles'] / first['user_cycles']
    total_cycle_ratio = last['total_cycles'] / first['total_cycles']
    segment_ratio = last['session_segments'] / first['session_segments']
    time_ratio = last['total_time_ms'] / first['total_time_ms']

    print(f"Participants increased by: {n_ratio:.1f}x ({first['participant_count']} → {last['participant_count']})")
    print(f"User cycles increased by:  {user_cycle_ratio:.2f}x ({first['user_cycles']:,} → {last['user_cycles']:,})")
    print(f"Total cycles increased by: {total_cycle_ratio:.2f}x ({first['total_cycles']:,} → {last['total_cycles']:,})")
    print(f"Segments increased by:     {segment_ratio:.2f}x ({first['session_segments']} → {last['session_segments']})")
    print(f"Time increased by:         {time_ratio:.2f}x ({first['total_time_ms']:,}ms → {last['total_time_ms']:,}ms)")
    print()

    # Estimate complexity
    import math
    if n_ratio > 1:
        complexity_user_cycles = math.log(user_cycle_ratio) / math.log(n_ratio)
        complexity_total_cycles = math.log(total_cycle_ratio) / math.log(n_ratio)
        complexity_time = math.log(time_ratio) / math.log(n_ratio)

        print("Estimated Computational Complexity:")
        print(f"  User Cycles:  O(n^{complexity_user_cycles:.2f})")
        print(f"  Total Cycles: O(n^{complexity_total_cycles:.2f})")
        print(f"  Time:         O(n^{complexity_time:.2f})")
        print()

    # Efficiency metrics
    print("Efficiency Metrics:")
    print("-" * 80)
    avg_user_cycles_per_participant = sum(d['user_cycles'] / d['participant_count'] for d in data) / len(data)
    avg_total_cycles_per_participant = sum(d['total_cycles'] / d['participant_count'] for d in data) / len(data)
    avg_time_per_participant = sum(d['total_time_ms'] / d['participant_count'] for d in data) / len(data)

    # Calculate overhead
    avg_overhead = sum((d['total_cycles'] - d['user_cycles']) / d['user_cycles'] * 100 for d in data) / len(data)

    print(f"Average user cycles per participant:    {avg_user_cycles_per_participant:,.0f}")
    print(f"Average total cycles per participant:   {avg_total_cycles_per_participant:,.0f}")
    print(f"Average cycle overhead (padding):       {avg_overhead:.1f}%")
    print(f"Average time per participant:           {avg_time_per_participant:,.0f} ms")
    print()

def generate_csv_report(data: List[Dict[str, Any]], output_file: str):
    """Generate a detailed CSV report."""
    with open(output_file, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow([
            'N', 'User Cycles', 'Total Cycles', 'Segments', 'Proving Time (ms)', 'Total Time (ms)',
            'Receipt Size (bytes)', 'Journal Size (bytes)', 'User Cycles/Participant',
            'Total Cycles/Participant', 'Overhead %', 'Time/Participant (ms)', 'Timestamp'
        ])

        for item in sorted(data, key=lambda x: x['participant_count']):
            overhead = (item['total_cycles'] - item['user_cycles']) / item['user_cycles'] * 100 if item['user_cycles'] > 0 else 0
            writer.writerow([
                item['participant_count'],
                item['user_cycles'],
                item['total_cycles'],
                item['session_segments'],
                item['proving_time_ms'],
                item['total_time_ms'],
                item['receipt_size_bytes'],
                item['journal_size_bytes'],
                item['user_cycles'] / item['participant_count'],
                item['total_cycles'] / item['participant_count'],
                overhead,
                item['total_time_ms'] / item['participant_count'],
                item['timestamp']
            ])

    print(f"✓ Detailed CSV report saved to: {output_file}")

def generate_plot(data: List[Dict[str, Any]], output_dir: str):
    """Generate plots if matplotlib is available."""
    try:
        import matplotlib.pyplot as plt
        import matplotlib
        matplotlib.use('Agg')  # Non-interactive backend
    except ImportError:
        print("Note: matplotlib not available, skipping plot generation")
        print("      Install with: pip install matplotlib")
        return

    # Sort data by participant count
    sorted_data = sorted(data, key=lambda x: x['participant_count'])

    participants = [d['participant_count'] for d in sorted_data]
    user_cycles = [d['user_cycles'] for d in sorted_data]
    total_cycles = [d['total_cycles'] for d in sorted_data]
    segments = [d['session_segments'] for d in sorted_data]
    times = [d['total_time_ms'] / 1000 for d in sorted_data]  # Convert to seconds

    # Create subplots
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(14, 10))
    fig.suptitle('RISC Zero Auction Benchmark Results', fontsize=16, fontweight='bold')

    # Plot 1: Cycles vs Participants (both user and total)
    ax1.plot(participants, user_cycles, 'o-', linewidth=2, markersize=6, color='#2563eb', label='User Cycles')
    ax1.plot(participants, total_cycles, 's--', linewidth=2, markersize=5, color='#dc2626', alpha=0.6, label='Total Cycles (padded)')
    ax1.set_xlabel('Number of Participants', fontsize=11)
    ax1.set_ylabel('Cycles', fontsize=11)
    ax1.set_title('Computational Cost (User vs Total Cycles)', fontsize=12, fontweight='bold')
    ax1.grid(True, alpha=0.3)
    ax1.legend()
    ax1.ticklabel_format(style='plain', axis='y')

    # Plot 2: Segments vs Participants
    ax2.plot(participants, segments, 'o-', linewidth=2, markersize=6, color='#dc2626')
    ax2.set_xlabel('Number of Participants', fontsize=11)
    ax2.set_ylabel('Number of Segments', fontsize=11)
    ax2.set_title('Proof Segments', fontsize=12, fontweight='bold')
    ax2.grid(True, alpha=0.3)

    # Plot 3: Time vs Participants
    ax3.plot(participants, times, 'o-', linewidth=2, markersize=6, color='#16a34a')
    ax3.set_xlabel('Number of Participants', fontsize=11)
    ax3.set_ylabel('Total Time (seconds)', fontsize=11)
    ax3.set_title('Execution Time', fontsize=12, fontweight='bold')
    ax3.grid(True, alpha=0.3)

    # Plot 4: Overhead analysis
    overhead = [(t - u) / u * 100 if u > 0 else 0 for u, t in zip(user_cycles, total_cycles)]
    ax4.plot(participants, overhead, 'o-', linewidth=2, markersize=6, color='#9333ea')
    ax4.set_xlabel('Number of Participants', fontsize=11)
    ax4.set_ylabel('Overhead (%)', fontsize=11)
    ax4.set_title('Cycle Overhead (Power-of-2 Padding)', fontsize=12, fontweight='bold')
    ax4.grid(True, alpha=0.3)
    ax4.axhline(y=0, color='gray', linestyle='--', alpha=0.5)

    plt.tight_layout()

    output_file = os.path.join(output_dir, 'benchmark_plots.png')
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"✓ Plots saved to: {output_file}")

def main():
    # Determine results directory
    if len(sys.argv) > 1:
        results_dir = sys.argv[1]
    else:
        results_dir = "benchmark_results/latest"

    if not os.path.exists(results_dir):
        print(f"Error: Results directory not found: {results_dir}")
        print(f"\nUsage: {sys.argv[0]} [results_dir]")
        sys.exit(1)

    # Load data
    print(f"Loading benchmark data from: {results_dir}")
    data = load_benchmark_data(results_dir)

    if not data:
        print("Error: No benchmark data found")
        sys.exit(1)

    print(f"Loaded {len(data)} benchmark results")

    # Generate statistics
    print_statistics(data)

    # Generate CSV report
    csv_output = os.path.join(results_dir, "detailed_analysis.csv")
    generate_csv_report(data, csv_output)

    # Generate plots
    generate_plot(data, results_dir)

    print("\n" + "="*80)
    print("Analysis complete!")
    print("="*80 + "\n")

if __name__ == "__main__":
    main()
