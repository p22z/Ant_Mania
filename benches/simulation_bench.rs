use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize};
use ant_mania::simulation::Simulation;
use ant_mania::{parser, SimulationConfig};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn create_test_map(size: usize) -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("bench_map_{}_pid_{}.txt", size, std::process::id()));
    let mut file = fs::File::create(&file_path).unwrap();
    
    match size {
        3 => {
            writeln!(file, "A north=B south=C").unwrap();
            writeln!(file, "B south=A east=C").unwrap();
            writeln!(file, "C north=A west=B").unwrap();
        }
        10 => {
            for i in 0..10 {
                let name = format!("Colony{}", i);
                let mut line = name.clone();
                if i > 0 {
                    line.push_str(&format!(" west=Colony{}", i - 1));
                }
                if i < 9 {
                    line.push_str(&format!(" east=Colony{}", i + 1));
                }
                writeln!(file, "{}", line).unwrap();
            }
        }
        28 => {
            for i in 0..28 {
                let name = format!("Colony{}", i);
                let mut line = name.clone();
                if i % 7 != 0 {
                    line.push_str(&format!(" west=Colony{}", i - 1));
                }
                if i % 7 != 6 {
                    line.push_str(&format!(" east=Colony{}", i + 1));
                }
                if i >= 7 {
                    line.push_str(&format!(" north=Colony{}", i - 7));
                }
                if i < 21 {
                    line.push_str(&format!(" south=Colony{}", i + 7));
                }
                writeln!(file, "{}", line).unwrap();
            }
        }
        _ => panic!("Unsupported map size"),
    }
    
    file_path
}

// Cleanup function for temp files
fn cleanup_temp_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

fn benchmark_simulation_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_hot_path");
    
    // Create test file once
    let map_file = create_test_map(28);
    let map_path = map_file.to_str().unwrap();
    
    for num_ants in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("28_colonies", num_ants),
            num_ants,
            |b, &num_ants| {
                // Setup simulation ONCE outside the timing loop
                let config = SimulationConfig {
                    num_ants,
                    map_file: map_path.to_string(),
                    max_moves: 10000,
                    seed: Some(42),
                };
                
                b.iter_batched(
                    || {
                        // Setup: parse and initialize (not timed)
                        let mut sim = Simulation::new(config.clone());
                        parser::parse_map_file(&mut sim, map_path).unwrap();
                        sim.initialize_ants(num_ants);
                        sim
                    },
                    |mut sim| {
                        // Only measure the actual simulation
                        black_box(sim.run_simulation());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    
    cleanup_temp_file(&map_file);
    group.finish();
}

fn benchmark_parsing_isolated(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_only");
    
    // Pre-create all test files
    let files: Vec<(usize, PathBuf)> = vec![3, 10, 28]
        .into_iter()
        .map(|size| (size, create_test_map(size)))
        .collect();
    
    for (size, file_path) in &files {
        let path_str = file_path.to_str().unwrap();
        
        // Read file content into memory once
        let file_content = fs::read_to_string(file_path).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("colonies", size),
            size,
            |b, _| {
                b.iter_batched(
                    || {
                        // Create temp file with content
                        let temp_path = std::env::temp_dir().join(format!("parse_bench_{}.txt", std::process::id()));
                        fs::write(&temp_path, &file_content).unwrap();
                        (temp_path, SimulationConfig {
                            num_ants: 10,
                            map_file: path_str.to_string(),
                            max_moves: 10000,
                            seed: Some(42),
                        })
                    },
                    |(temp_path, config)| {
                        let mut sim = Simulation::new(config);
                        black_box(parser::parse_map_file(&mut sim, temp_path.to_str().unwrap()).unwrap());
                        cleanup_temp_file(&temp_path);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    
    // Cleanup
    for (_, path) in files {
        cleanup_temp_file(&path);
    }
    
    group.finish();
}

fn benchmark_initialization_isolated(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization_only");
    
    let map_file = create_test_map(28);
    let map_path = map_file.to_str().unwrap();
    
    // Parse map once
    let config = SimulationConfig {
        num_ants: 10,
        map_file: map_path.to_string(),
        max_moves: 10000,
        seed: Some(42),
    };
    
    for num_ants in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("ants", num_ants),
            num_ants,
            |b, &num_ants| {
                b.iter_batched(
                    || {
                        let mut sim = Simulation::new(config.clone());
                        parser::parse_map_file(&mut sim, map_path).unwrap();
                        sim
                    },
                    |mut sim| {
                        black_box(sim.initialize_ants(num_ants));
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    
    cleanup_temp_file(&map_file);
    group.finish();
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_analysis");
    group.sample_size(50); // Reduced for longer benchmarks
    
    let map_file = create_test_map(28);
    let map_path = map_file.to_str().unwrap();
    
    // Test scaling with different ant counts
    for num_ants in [1, 10, 25, 50, 75, 100, 150, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("ants", num_ants),
            num_ants,
            |b, &num_ants| {
                let config = SimulationConfig {
                    num_ants,
                    map_file: map_path.to_string(),
                    max_moves: 1000, // Reduced for scaling test
                    seed: Some(42),
                };
                
                b.iter_batched(
                    || {
                        let mut sim = Simulation::new(config.clone());
                        parser::parse_map_file(&mut sim, map_path).unwrap();
                        sim.initialize_ants(num_ants);
                        sim
                    },
                    |mut sim| {
                        black_box(sim.run_simulation());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    
    cleanup_temp_file(&map_file);
    group.finish();
}

fn benchmark_random_seed_variance(c: &mut Criterion) {
    let mut group = c.benchmark_group("seed_variance");
    group.sample_size(100); // Need good sample for variance
    
    let map_file = create_test_map(28);
    let map_path = map_file.to_str().unwrap();
    
    group.bench_function("100_ants_random_seeds", |b| {
        let mut seed_counter = 0u64;
        
        b.iter_batched(
            || {
                seed_counter += 1;
                let config = SimulationConfig {
                    num_ants: 100,
                    map_file: map_path.to_string(),
                    max_moves: 1000,
                    seed: Some(seed_counter), // Different seed each time
                };
                let mut sim = Simulation::new(config);
                parser::parse_map_file(&mut sim, map_path).unwrap();
                sim.initialize_ants(100);
                sim
            },
            |mut sim| {
                black_box(sim.run_simulation());
            },
            BatchSize::SmallInput,
        );
    });
    
    cleanup_temp_file(&map_file);
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_simulation_hot_path,
    benchmark_parsing_isolated,
    benchmark_initialization_isolated,
    benchmark_scaling,
    benchmark_random_seed_variance
);
criterion_main!(benches);