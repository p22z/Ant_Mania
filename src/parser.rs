use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use crate::types::{ColonyId, Direction};
use crate::simulation::Simulation;

pub fn parse_map_file(simulation: &mut Simulation, file_path: &str) -> Result<(), String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;
    let reader = BufReader::new(file);
    
    // First pass: collect all colony names to assign IDs
    let mut temp_colonies: Vec<(String, Vec<(Direction, String)>)> = Vec::new();
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {e}"))?;
        let line = line.trim();
        
        // Skip empty lines
        if line.is_empty() {
            continue;
        }
        
        // Parse line format: "ColonyName direction=Neighbor direction=Neighbor..."
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        let colony_name = parts[0].to_string();
        let mut connections = Vec::new();
        
        // Parse direction=neighbor pairs
        for connection_str in &parts[1..] {
            if let Some(eq_pos) = connection_str.find('=') {
                let dir_str = &connection_str[..eq_pos];
                let neighbor_name = &connection_str[eq_pos + 1..];
                
                let direction = dir_str.parse::<Direction>()
                    .map_err(|_| format!("Invalid direction: {dir_str}"))?;
                
                connections.push((direction, neighbor_name.to_string()));
            } else {
                return Err(format!("Invalid connection format: {connection_str}"));
            }
        }
        
        temp_colonies.push((colony_name, connections));
    }
    
    // Initialize simulation data structures
    let num_colonies = temp_colonies.len();
    simulation.initialize_with_capacity(num_colonies);
    
    // Create name-to-ID mapping with pre-allocated capacity
    let mut name_to_id = HashMap::with_capacity(num_colonies);
    for (i, (name, _)) in temp_colonies.iter().enumerate() {
        if name_to_id.insert(name.clone(), i as ColonyId).is_some() {
            return Err(format!("Duplicate colony: {name}"));
        }
    }
    
    // Second pass: build the graph structure
    for (colony_id, (colony_name, connections)) in temp_colonies.iter().enumerate() {
        simulation.set_colony_name(colony_id as ColonyId, colony_name.clone());
        
        for (direction, neighbor_name) in connections {
            let neighbor_id = *name_to_id.get(neighbor_name)
                .ok_or_else(|| format!("Unknown neighbor colony: {neighbor_name}"))?;
            
            simulation.set_neighbor(colony_id as ColonyId, *direction, neighbor_id);
        }
    }
    
    // Update valid direction bitmasks for all colonies
    simulation.update_all_valid_directions();
    
    Ok(())
}


