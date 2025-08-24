# Ant Mania Simulation

A high-performance simulation of giant space ants invading planet Hiveum, built in Rust with extensive optimizations for speed and efficiency.

## Overview

This project simulates ants moving randomly through a network of colonies connected by tunnels. When two ants meet in the same colony, they fight and destroy both themselves and the colony. The simulation continues until either all ants are destroyed or each ant has moved 10,000 times.

## Features

- **High Performance**: Optimized for sub-millisecond latency using advanced techniques
- **Memory Efficient**: Struct-of-Arrays pattern for cache-friendly data access
- **Comprehensive Benchmarks**: Detailed performance analysis and optimization reports
- **Deterministic**: Reproducible results with optional seed control

## Usage

### Building the Project

```bash
# Build in release mode for optimal performance
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Running the Simulation

```bash
# Basic usage
cargo run --release -- <num_ants> <map_file>

# Example with 100 ants on the small map
cargo run --release -- 100 maps/hiveum_map_small.txt

# Example with 1000 ants on the medium map
cargo run --release -- 1000 maps/hiveum_map_medium.txt

# With optional seed for reproducibility
cargo run --release -- 100 maps/hiveum_map_small.txt --seed 12345
```

## Performance Results

### Benchmark Summary

| Map Size | Ants | Runtime | Per-Move Latency |
|----------|------|---------|------------------|
| Small (28 colonies) | 100 | 0.98ms | 24.3ns |
| Small (28 colonies) | 500 | 31μs* | 66.1ns* |
| Small (28 colonies) | 1000 | 35μs* | 36.2ns* |
| Medium (6,763 colonies) | 100 | 7.4ms | 170.5ns |
| Medium (6,763 colonies) | 500 | 11.8ms | 183.8ns |
| Medium (6,763 colonies) | 1000 | 13.3ms | 207.7ns |

*Early termination due to all ants being destroyed

### Key Optimizations

1. **Struct-of-Arrays (SoA)**: Cache-friendly memory layout
2. **Fast RNG**: Custom XorShift generator (10x faster than standard)
3. **Tombstoning**: O(1) colony destruction instead of removal
4. **Direction Lookup**: Pre-computed tables for movement selection
5. **Collision Detection**: O(1) occupancy tracking per colony

## Documentation

- [`description.txt`](description.txt) - Original problem specification
- [`OPTIMIZATION_REPORT.md`](OPTIMIZATION_REPORT.md) - Detailed performance analysis
- [`ant_mania_solution_analysis.md`](ant_mania_solution_analysis.md) - Comprehensive solution design
- [`TODO.md`](TODO.md) - Complete task breakdown and implementation plan

## Project Structure

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library root
├── cli.rs           # Command-line argument parsing
├── parser.rs        # Map file parsing
├── types.rs         # Core data types and structures
├── simulation.rs    # Main simulation logic
├── engine.rs        # Optimized simulation engine
└── rng.rs           # Fast random number generation

maps/
├── hiveum_map_small.txt  # Small test map (28 colonies)
├── hiveum_map_medium.txt # Medium test map (6,763 colonies)
├── test_map.txt         # Simple test map
└── description.txt      # Original problem specification

benches/
├── micro_bench.rs   # Micro-benchmarks for individual components
├── real_map_bench.rs # Real map performance testing
└── simulation_bench.rs # Full simulation benchmarks
```

## Requirements

- Rust 1.70+ (2021 edition)
- No external dependencies for core functionality
- Optional: Criterion for benchmarking

## License

This project is provided as-is for evaluation purposes.

## Development

This codebase was developed with the assistance of [Claude Code](https://claude.ai/code) and [Cursor](https://cursor.sh/).# Hiveum-Ants
