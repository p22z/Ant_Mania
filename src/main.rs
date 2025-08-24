use std::time::Duration;
use ant_mania::{simulation::Simulation, engine::SimulationResult, cli};

fn main() {
    println!("Ant Mania Simulation");
    
    // Parse command line arguments
    let config = match cli::parse_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };
    
    println!("Configuration:");
    println!("  Ants: {}", config.num_ants);
    println!("  Map: {}", config.map_file);
    println!("  Max moves: {}", config.max_moves);
    if let Some(seed) = config.seed {
        println!("  Seed: {seed}");
    }
    println!();
    
    let mut sim = Simulation::new(config.clone());
    
    // Parse map file
    match ant_mania::parser::parse_map_file(&mut sim, &config.map_file) {
        Ok(()) => {
            println!("Successfully parsed map with {} colonies", sim.num_colonies());
            
            // Initialize ants
            sim.initialize_ants(config.num_ants);
            println!("Initialized {} ants", sim.num_ants());
            
            // Run simulation
            println!("Starting simulation...");
            let start_time = std::time::Instant::now();
            
            let result = sim.run_simulation();
            
            let elapsed = start_time.elapsed();
            println!("Simulation completed in {elapsed:?}");
            
            // Output results
            print_results(&result);
            
            // Performance summary
            print_performance_summary(&result, elapsed, sim.num_colonies(), config.num_ants);
        }
        Err(e) => {
            eprintln!("Error parsing map: {e:?}");
            std::process::exit(1);
        }
    }
}

fn print_results(result: &SimulationResult) {
    println!("\n=== Simulation Results ===");
    println!("Iterations: {}", result.iterations);
    println!("Total ant moves: {}", result.total_moves);
    println!("Colonies destroyed: {}", result.destructions.len());
    println!("Colonies surviving: {}", result.surviving_colonies.len());
    
    if !result.destructions.is_empty() {
        println!("\nDestruction events:");
        for destruction in &result.destructions {
            println!("{destruction}");
        }
    }
    
    if !result.surviving_colonies.is_empty() {
        println!("\nFinal map state:");
        for colony in &result.surviving_colonies {
            println!("{colony}");
        }
    }
}

fn print_performance_summary(result: &SimulationResult, elapsed: Duration, num_colonies: usize, num_ants: u16) {
    println!("\n=== Performance Summary ===");
    println!("Total runtime: {elapsed:?}");
    println!("Colonies processed: {num_colonies}");
    println!("Ants simulated: {num_ants}");
    
    if result.iterations > 0 {
        let avg_iteration_time = elapsed.as_nanos() as f64 / result.iterations as f64;
        println!("Average time per iteration: {:.2}μs", avg_iteration_time / 1000.0);
    }
    
    if result.total_moves > 0 {
        let avg_move_time = elapsed.as_nanos() as f64 / result.total_moves as f64;
        println!("Average time per ant move: {avg_move_time:.2}ns");
    }
    
    // Performance classification
    let runtime_ms = elapsed.as_micros() as f64 / 1000.0;
    if num_colonies <= 50 {
        // Small map
        if runtime_ms < 100.0 {
            println!("✅ Excellent performance (target: <100ms)");
        } else {
            println!("⚠️  Performance below target (<100ms for small maps)");
        }
    } else {
        // Medium/large map
        if runtime_ms < 1000.0 {
            println!("✅ Excellent performance (target: <1s for large maps)");
        } else {
            println!("⚠️  Performance below target (<1s for large maps)");
        }
    }
}
