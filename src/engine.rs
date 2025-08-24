use crate::types::{ColonyId, AntId};
use crate::simulation::Simulation;

const MAX_ITERATIONS: u32 = 1_000_000;

/// Main simulation engine with optimized hot path
impl Simulation {
    /// Run the complete simulation until termination condition
    pub fn run_simulation(&mut self) -> SimulationResult {
        let mut iteration = 0;
        let mut total_moves = 0;
        let mut destructions = Vec::new();
        
        loop {
            let moves_this_iteration = self.step_simulation(&mut destructions);
            total_moves += moves_this_iteration;
            iteration += 1;
            
            // Check termination conditions
            if self.all_ants_dead() || self.all_ants_reached_max_moves() {
                break;
            }
            
            // Safety check to prevent infinite loops
            if iteration > MAX_ITERATIONS {
                eprintln!("Warning: Simulation exceeded {} iterations, terminating", MAX_ITERATIONS);
                break;
            }
        }
        
        SimulationResult {
            iterations: iteration,
            total_moves,
            destructions,
            surviving_colonies: self.get_surviving_colonies(),
        }
    }
    
    /// Execute one step of the simulation (process all living ants once)
    /// Uses two-phase approach: calculate moves, then apply them with collision detection
    #[inline(always)]
    fn step_simulation(&mut self, destructions: &mut Vec<String>) -> u32 {
        let mut moves_count = 0;
        let mut pending_moves = Vec::with_capacity(self.ant_colonies.len() / 2); // Pre-allocate capacity
        
        // Phase 1: Calculate moves for all living ants (based on current state)
        for ant_id in 0..self.ant_colonies.len() {
            let ant_id = ant_id as AntId;
            
            // Skip dead ants efficiently (branch prediction optimization)
            if !self.ant_alive[ant_id as usize] {
                continue;
            }
            
            // Check if ant has reached move limit
            if self.ant_moves[ant_id as usize] >= self.config.max_moves {
                continue;
            }
            
            // Calculate where this ant wants to move
            if let Some(target_colony) = self.calculate_ant_move(ant_id) {
                pending_moves.push((ant_id, target_colony));
            } else {
                // Ant is trapped, just increment move counter
                self.ant_moves[ant_id as usize] += 1;
                moves_count += 1;
            }
        }
        
        // Phase 2: Apply moves sequentially with collision detection
        for (ant_id, target_colony) in pending_moves {
            // Check if ant is still alive (might have died in earlier collision)
            if self.ant_alive[ant_id as usize] {
                self.move_ant_to_colony(ant_id, target_colony, destructions);
                moves_count += 1;
            }
        }
        
        moves_count
    }
    
    /// Calculate where an ant wants to move (Phase 1 - no state changes)
    #[inline(always)]
    fn calculate_ant_move(&mut self, ant_id: AntId) -> Option<ColonyId> {
        let ant_idx = ant_id as usize;
        let current_colony = self.ant_colonies[ant_idx];
        let current_colony_idx = current_colony as usize;
        
        // Check if current colony is destroyed
        if !self.colony_valid[current_colony_idx] {
            // Ant will die, no move
            return None;
        }
        
        // Get valid directions for current colony
        let valid_dirs = self.colony_valid_dirs[current_colony_idx];
        
        // Check if ant is trapped (no valid moves)
        if valid_dirs == 0 {
            // Ant stays in place (no move)
            return None;
        }
        
        // Select random direction
        if let Some(direction) = self.select_random_direction(valid_dirs) {
            if let Some(target_colony) = self.get_neighbor(current_colony, direction) {
                // Check if target colony is valid
                if self.colony_valid[target_colony as usize] {
                    return Some(target_colony);
                }
            }
        }
        
        // No valid move found
        None
    }
    
    /// Move ant to target colony and handle collision detection
    #[inline(always)]
    fn move_ant_to_colony(&mut self, ant_id: AntId, target_colony: ColonyId, destructions: &mut Vec<String>) {
        let ant_idx = ant_id as usize;
        let current_colony = self.ant_colonies[ant_idx];
        let target_idx = target_colony as usize;
        
        // Check if target colony has been destroyed since move calculation
        if !self.colony_valid[target_idx] {
            // Target colony destroyed, ant dies
            self.remove_ant_from_colony(ant_id, current_colony);
            self.kill_ant(ant_id);
            return;
        }
        
        // Remove ant from current colony first
        self.remove_ant_from_colony(ant_id, current_colony);
        
        // Check for collision AFTER removing from current but BEFORE adding to target
        if self.colony_ant_count[target_idx] > 0 {
            // Collision detected! Use O(1) tracking to find the other ant
            let other_ant = self.colony_first_ant[target_idx];
            
            // Record destruction message
            let colony_name = &self.colony_names[target_idx];
            let destruction_msg = format!("{} has been destroyed by ant {} and ant {}!", 
                                         colony_name, 
                                         ant_id, 
                                         other_ant.unwrap_or(0));
            destructions.push(destruction_msg);
            
            // Kill both ants
            self.kill_ant(ant_id);
            if let Some(other_ant) = other_ant {
                self.kill_ant(other_ant);
            }
            
            // Destroy colony
            self.destroy_colony(target_colony);
        } else {
            // No collision, move ant safely
            // Place ant in target colony
            self.ant_colonies[ant_idx] = target_colony;
            self.ant_moves[ant_idx] += 1;
            
            // Update target colony occupancy
            self.colony_ant_count[target_idx] += 1;  // INCREMENT, don't set to 1!
            self.colony_first_ant[target_idx] = Some(ant_id);
        }
    }
    
    /// Remove ant from colony (update occupancy tracking)
    #[inline(always)]
    fn remove_ant_from_colony(&mut self, ant_id: AntId, colony_id: ColonyId) {
        let colony_idx = colony_id as usize;
        
        // Decrease occupancy count
        if self.colony_ant_count[colony_idx] > 0 {
            self.colony_ant_count[colony_idx] -= 1;
        }
        
        // Update first ant tracking
        if self.colony_first_ant[colony_idx] == Some(ant_id) {
            self.colony_first_ant[colony_idx] = None;
        }
    }
    
    /// Kill an ant (mark as dead)
    #[inline(always)]
    fn kill_ant(&mut self, ant_id: AntId) {
        let ant_idx = ant_id as usize;
        if ant_idx < self.ant_alive.len() {
            self.ant_alive[ant_idx] = false;
            
            // Remove from current colony
            let current_colony = self.ant_colonies[ant_idx];
            self.remove_ant_from_colony(ant_id, current_colony);
        }
    }
    
    /// Destroy a colony using tombstoning (O(1) operation)
    fn destroy_colony(&mut self, colony_id: ColonyId) {
        let colony_idx = colony_id as usize;
        if colony_idx >= self.num_colonies {
            return;
        }
        
        // Mark colony as invalid (tombstoning)
        self.colony_valid[colony_idx] = false;
        
        // Clear occupancy
        self.colony_ant_count[colony_idx] = 0;
        self.colony_first_ant[colony_idx] = None;
        
        // Update valid direction bitmasks for all neighbors
        self.update_neighbors_after_destruction(colony_id);
    }
    
    /// Update neighbor colonies' valid directions after colony destruction
    fn update_neighbors_after_destruction(&mut self, destroyed_colony: ColonyId) {
        let destroyed_idx = destroyed_colony as usize;
        
        // Check all four directions for neighbors
        let neighbors = [
            self.colony_north[destroyed_idx],
            self.colony_south[destroyed_idx],
            self.colony_east[destroyed_idx],
            self.colony_west[destroyed_idx],
        ];
        
        for neighbor_id in neighbors.iter().flatten() {
            self.update_valid_directions(*neighbor_id);
        }
    }
    
    /// Check if all ants are dead (early termination optimization)
    #[inline]
    fn all_ants_dead(&self) -> bool {
        // Use iterator any for early exit
        !self.ant_alive.iter().any(|&alive| alive)
    }
    
    /// Check if all living ants have reached max moves (early termination optimization)
    #[inline]
    fn all_ants_reached_max_moves(&self) -> bool {
        // Use iterator with zip for better performance
        self.ant_alive
            .iter()
            .zip(self.ant_moves.iter())
            .all(|(&alive, &moves)| !alive || moves >= self.config.max_moves)
    }
    
    
    /// Get list of surviving colonies in the same format as input
    fn get_surviving_colonies(&self) -> Vec<String> {
        let mut survivors = Vec::new();
        
        for (i, &valid) in self.colony_valid.iter().enumerate() {
            if valid {
                let colony_name = &self.colony_names[i];

                let mut connection_count = 0;
                // Pre-calculate required capacity to avoid reallocations
                let mut estimated_length = colony_name.len();
                
                // Count valid connections and estimate length
                if let Some(north_id) = self.colony_north[i] {
                    if self.colony_valid[north_id as usize] {
                        estimated_length += 7 + self.colony_names[north_id as usize].len(); // " north="
                        connection_count += 1;
                    }
                }
                if let Some(south_id) = self.colony_south[i] {
                    if self.colony_valid[south_id as usize] {
                        estimated_length += 7 + self.colony_names[south_id as usize].len(); // " south="
                        connection_count += 1;
                    }
                }
                if let Some(east_id) = self.colony_east[i] {
                    if self.colony_valid[east_id as usize] {
                        estimated_length += 6 + self.colony_names[east_id as usize].len(); // " east="
                        connection_count += 1;
                    }
                }
                if let Some(west_id) = self.colony_west[i] {
                    if self.colony_valid[west_id as usize] {
                        estimated_length += 6 + self.colony_names[west_id as usize].len(); // " west="
                        connection_count += 1;
                    }
                }
                
                // Build output string with pre-allocated capacity
                let mut output_line = String::with_capacity(estimated_length);
                output_line.push_str(colony_name);
                
                // Add connections directly without intermediate Vec
                if let Some(north_id) = self.colony_north[i] {
                    if self.colony_valid[north_id as usize] {
                        output_line.push_str(" north=");
                        output_line.push_str(&self.colony_names[north_id as usize]);
                    }
                }
                if let Some(south_id) = self.colony_south[i] {
                    if self.colony_valid[south_id as usize] {
                        output_line.push_str(" south=");
                        output_line.push_str(&self.colony_names[south_id as usize]);
                    }
                }
                if let Some(east_id) = self.colony_east[i] {
                    if self.colony_valid[east_id as usize] {
                        output_line.push_str(" east=");
                        output_line.push_str(&self.colony_names[east_id as usize]);
                    }
                }
                if let Some(west_id) = self.colony_west[i] {
                    if self.colony_valid[west_id as usize] {
                        output_line.push_str(" west=");
                        output_line.push_str(&self.colony_names[west_id as usize]);
                    }
                }
                
                survivors.push(output_line);
            }
        }
        
        // Sort for consistent output
        survivors.sort();
        survivors
    }
}

/// Result of a complete simulation run
#[derive(Debug)]
pub struct SimulationResult {
    pub iterations: u32,
    pub total_moves: u32,
    pub destructions: Vec<String>,
    pub surviving_colonies: Vec<String>,
}