use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ant_mania::simulation::Simulation;
use ant_mania::{parser, SimulationConfig};
use std::path::Path;

fn benchmark_real_maps(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_maps");
    
    // Test with small map if it exists
    if Path::new("maps/hiveum_map_small.txt").exists() {
        for num_ants in [100, 500, 1000].iter() {
            group.bench_with_input(
                BenchmarkId::new("small_map", num_ants),
                num_ants,
                |b, &num_ants| {
                    b.iter(|| {
                        let config = SimulationConfig {
                            num_ants,
                            map_file: "maps/hiveum_map_small.txt".to_string(),
                            max_moves: 10000,
                            seed: Some(42),
                        };
                        let mut sim = Simulation::new(config);
                        parser::parse_map_file(&mut sim, "maps/hiveum_map_small.txt").unwrap();
                        sim.initialize_ants(num_ants);
                        black_box(sim.run_simulation());
                    });
                },
            );
        }
    }
    
    // Test with medium map if it exists
    if Path::new("maps/hiveum_map_medium.txt").exists() {
        for num_ants in [100, 500, 1000].iter() {
            group.bench_with_input(
                BenchmarkId::new("medium_map", num_ants),
                num_ants,
                |b, &num_ants| {
                    b.iter(|| {
                        let config = SimulationConfig {
                            num_ants,
                            map_file: "maps/hiveum_map_medium.txt".to_string(),
                            max_moves: 10000,
                            seed: Some(42),
                        };
                        let mut sim = Simulation::new(config);
                        parser::parse_map_file(&mut sim, "maps/hiveum_map_medium.txt").unwrap();
                        sim.initialize_ants(num_ants);
                        black_box(sim.run_simulation());
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn benchmark_single_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration_latency");
    group.sample_size(10); // Reduce sample size for long-running benchmarks
    
    if Path::new("maps/hiveum_map_medium.txt").exists() {
        group.bench_function("medium_map_1000_ants_single_iteration", |b| {
            // Setup once outside the timing loop
            let config = SimulationConfig {
                num_ants: 1000,
                map_file: "maps/hiveum_map_medium.txt".to_string(),
                max_moves: 1, // Only one move to measure single iteration
                seed: Some(42),
            };
            
            b.iter_batched(
                || {
                    let mut sim = Simulation::new(config.clone());
                    parser::parse_map_file(&mut sim, "maps/hiveum_map_medium.txt").unwrap();
                    sim.initialize_ants(1000);
                    sim
                },
                |mut sim| {
                    black_box(sim.run_simulation());
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_real_maps, benchmark_single_iteration);
criterion_main!(benches);