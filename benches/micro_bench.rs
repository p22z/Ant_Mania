use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ant_mania::simulation::Simulation;
use ant_mania::{parser, SimulationConfig};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Micro-benchmarks for specific performance-critical operations

fn create_dense_map(size: usize) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("dense_map_{}_pid_{}.txt", size, std::process::id()));
    let mut file = fs::File::create(&file_path).unwrap();
    
    // Create a dense grid map for maximum collision potential
    let grid_size = (size as f64).sqrt() as usize;
    
    for y in 0..grid_size {
        for x in 0..grid_size {
            let name = format!("C{}_{}", x, y);
            let mut line = name.clone();
            
            // Connect to all 4 neighbors if they exist
            if x > 0 {
                line.push_str(&format!(" west=C{}_{}", x - 1, y));
            }
            if x < grid_size - 1 {
                line.push_str(&format!(" east=C{}_{}", x + 1, y));
            }
            if y > 0 {
                line.push_str(&format!(" north=C{}_{}", x, y - 1));
            }
            if y < grid_size - 1 {
                line.push_str(&format!(" south=C{}_{}", x, y + 1));
            }
            
            writeln!(file, "{}", line).unwrap();
        }
    }
    
    file_path
}

fn benchmark_rng_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng_performance");
    
    group.bench_function("rng_next_u32_1M", |b| {
        b.iter(|| {
            let mut rng = ant_mania::rng::FastRng::new(42);
            let mut sum = 0u64;
            
            // Actually test RNG performance with 1M calls
            for _ in 0..1_000_000 {
                sum = sum.wrapping_add(black_box(rng.next_u32()) as u64);
            }
            black_box(sum); // Prevent optimization
        });
    });
    
    group.finish();
}

fn benchmark_collision_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("collision_hot_path");
    
    let map_file = create_dense_map(100); // 10x10 grid = 100 colonies
    let map_path = map_file.to_str().unwrap();
    
    // High ant density to force many collisions
    group.bench_function("dense_collisions_200_ants", |b| {
        b.iter_batched(
            || {
                let config = SimulationConfig {
                    num_ants: 200,
                    map_file: map_path.to_string(),
                    max_moves: 50, // Short simulation, focus on collision detection
                    seed: Some(42),
                };
                let mut sim = Simulation::new(config);
                parser::parse_map_file(&mut sim, map_path).unwrap();
                sim.initialize_ants(200);
                sim
            },
            |mut sim| {
                black_box(sim.run_simulation());
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    let _ = fs::remove_file(map_file);
    group.finish();
}

fn benchmark_direction_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("direction_selection");
    
    let map_file = create_dense_map(25); // 5x5 grid for quick setup
    let map_path = map_file.to_str().unwrap();
    
    group.bench_function("direction_lookup_heavy", |b| {
        b.iter_batched(
            || {
                let config = SimulationConfig {
                    num_ants: 100,
                    map_file: map_path.to_string(),
                    max_moves: 500, // Many moves to test direction selection
                    seed: Some(42),
                };
                let mut sim = Simulation::new(config);
                parser::parse_map_file(&mut sim, map_path).unwrap();
                sim.initialize_ants(100);
                sim
            },
            |mut sim| {
                black_box(sim.run_simulation());
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    let _ = fs::remove_file(map_file);
    group.finish();
}

fn benchmark_memory_access_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    let map_file = create_dense_map(64); // 8x8 grid
    let map_path = map_file.to_str().unwrap();
    
    // Test different ant counts to see cache effects
    for num_ants in [32, 64, 128, 256].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::new("cache_pressure", num_ants),
            num_ants,
            |b, &num_ants| {
                b.iter_batched(
                    || {
                        let config = SimulationConfig {
                            num_ants,
                            map_file: map_path.to_string(),
                            max_moves: 100,
                            seed: Some(42),
                        };
                        let mut sim = Simulation::new(config);
                        parser::parse_map_file(&mut sim, map_path).unwrap();
                        sim.initialize_ants(num_ants);
                        sim
                    },
                    |mut sim| {
                        black_box(sim.run_simulation());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    let _ = fs::remove_file(map_file);
    group.finish();
}

criterion_group!(
    benches,
    benchmark_rng_performance,
    benchmark_collision_detection,
    benchmark_direction_selection,
    benchmark_memory_access_patterns
);
criterion_main!(benches);