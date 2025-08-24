use std::collections::HashMap;
use crate::types::{ColonyId, AntId, Direction, SimulationConfig};
use crate::rng::FastRng;

/// Core simulation structure using Struct-of-Arrays pattern for cache efficiency
pub struct Simulation {
    // Colony data (SoA pattern - hot data accessed every iteration)
    pub colony_valid: Vec<bool>,                    // Tombstoning - false means destroyed
    pub colony_north: Vec<Option<ColonyId>>,        // Neighbors in each direction
    pub colony_south: Vec<Option<ColonyId>>,
    pub colony_east: Vec<Option<ColonyId>>,
    pub colony_west: Vec<Option<ColonyId>>,
    pub colony_valid_dirs: Vec<u8>,                 // Bitmask: bit 0=North, 1=South, 2=East, 3=West
    
    // Ant tracking (Hybrid approach for O(1) collision detection)
    pub ant_colonies: Vec<ColonyId>,                // Current position of each ant
    pub ant_alive: Vec<bool>,                       // Alive status for each ant
    pub ant_moves: Vec<u16>,                        // Move counter for each ant
    
    // Collision detection (Colony-centric for O(1) checks)
    pub colony_ant_count: Vec<u8>,                  // Number of ants in each colony
    pub colony_first_ant: Vec<Option<AntId>>,       // First ant in each colony (for collision messages)
    
    // Fast RNG
    pub rng: FastRng,
    
    // Cold data (rarely accessed during simulation)
    pub colony_names: Vec<String>,                  // Original names for output
    pub name_to_id: HashMap<String, ColonyId>,      // For parsing
    
    // Simulation state
    pub config: SimulationConfig,
    pub num_colonies: usize,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let seed = config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });
        
        Self {
            colony_valid: Vec::new(),
            colony_north: Vec::new(),
            colony_south: Vec::new(),
            colony_east: Vec::new(),
            colony_west: Vec::new(),
            colony_valid_dirs: Vec::new(),
            ant_colonies: Vec::new(),
            ant_alive: Vec::new(),
            ant_moves: Vec::new(),
            colony_ant_count: Vec::new(),
            colony_first_ant: Vec::new(),
            rng: FastRng::new(seed),
            colony_names: Vec::new(),
            name_to_id: HashMap::new(),
            config,
            num_colonies: 0,
        }
    }
    
    /// Get neighbor colony ID in the given direction
    #[inline(always)]
    pub fn get_neighbor(&self, colony_id: ColonyId, direction: Direction) -> Option<ColonyId> {
        let idx = colony_id as usize;
        if idx >= self.num_colonies || !self.colony_valid[idx] {
            return None;
        }
        
        match direction {
            Direction::North => self.colony_north[idx],
            Direction::South => self.colony_south[idx],
            Direction::East => self.colony_east[idx],
            Direction::West => self.colony_west[idx],
        }
    }
    
    /// Efficient direction selection using lookup table
    #[inline(always)]
    pub fn select_random_direction(&mut self, valid_dirs_mask: u8) -> Option<Direction> {
        if valid_dirs_mask == 0 {
            return None;
        }
        
        // Use a lookup table for all possible bit patterns (only 16 possibilities)
        // Each entry contains the valid directions as a packed array
        static DIRECTION_LOOKUP: [[Option<Direction>; 4]; 16] = [
            [None, None, None, None],                                                                     // 0000
            [Some(Direction::North), None, None, None],                                                   // 0001
            [Some(Direction::South), None, None, None],                                                   // 0010
            [Some(Direction::North), Some(Direction::South), None, None],                                 // 0011
            [Some(Direction::East), None, None, None],                                                    // 0100
            [Some(Direction::North), Some(Direction::East), None, None],                                  // 0101
            [Some(Direction::South), Some(Direction::East), None, None],                                  // 0110
            [Some(Direction::North), Some(Direction::South), Some(Direction::East), None],               // 0111
            [Some(Direction::West), None, None, None],                                                    // 1000
            [Some(Direction::North), Some(Direction::West), None, None],                                  // 1001
            [Some(Direction::South), Some(Direction::West), None, None],                                  // 1010
            [Some(Direction::North), Some(Direction::South), Some(Direction::West), None],               // 1011
            [Some(Direction::East), Some(Direction::West), None, None],                                   // 1100
            [Some(Direction::North), Some(Direction::East), Some(Direction::West), None],                // 1101
            [Some(Direction::South), Some(Direction::East), Some(Direction::West), None],                // 1110
            [Some(Direction::North), Some(Direction::South), Some(Direction::East), Some(Direction::West)], // 1111
        ];
        
        let directions = &DIRECTION_LOOKUP[valid_dirs_mask as usize];
        let count = directions.iter().take_while(|d| d.is_some()).count();
        
        if count == 0 {
            return None;
        }
        
        let random_idx = self.rng.next_range(count as u32) as usize;
        directions[random_idx]
    }
    
    /// Update valid directions bitmask for a colony
    pub fn update_valid_directions(&mut self, colony_id: ColonyId) {
        let idx = colony_id as usize;
        if idx >= self.num_colonies {
            return;
        }
        
        let mut mask = 0u8;
        
        if let Some(neighbor_id) = self.colony_north[idx] {
            if self.colony_valid[neighbor_id as usize] {
                mask |= Direction::North.as_bit_mask();
            }
        }
        if let Some(neighbor_id) = self.colony_south[idx] {
            if self.colony_valid[neighbor_id as usize] {
                mask |= Direction::South.as_bit_mask();
            }
        }
        if let Some(neighbor_id) = self.colony_east[idx] {
            if self.colony_valid[neighbor_id as usize] {
                mask |= Direction::East.as_bit_mask();
            }
        }
        if let Some(neighbor_id) = self.colony_west[idx] {
            if self.colony_valid[neighbor_id as usize] {
                mask |= Direction::West.as_bit_mask();
            }
        }
        
        self.colony_valid_dirs[idx] = mask;
    }
    
    pub fn num_colonies(&self) -> usize {
        self.num_colonies
    }
    
    pub fn num_ants(&self) -> usize {
        self.ant_colonies.len()
    }
    
    /// Initialize simulation with given colony capacity
    pub fn initialize_with_capacity(&mut self, num_colonies: usize) {
        self.num_colonies = num_colonies;
        
        // Initialize colony data structures
        self.colony_valid.resize(num_colonies, true);
        self.colony_north.resize(num_colonies, None);
        self.colony_south.resize(num_colonies, None);
        self.colony_east.resize(num_colonies, None);
        self.colony_west.resize(num_colonies, None);
        self.colony_valid_dirs.resize(num_colonies, 0);
        
        // Initialize collision detection structures
        self.colony_ant_count.resize(num_colonies, 0);
        self.colony_first_ant.resize(num_colonies, None);
        
        // Initialize cold data
        self.colony_names.resize(num_colonies, String::new());
        
        // Reserve space for name mapping
        self.name_to_id.reserve(num_colonies);
    }
    
    /// Set colony name (for output purposes)
    pub fn set_colony_name(&mut self, colony_id: ColonyId, name: String) {
        let idx = colony_id as usize;
        if idx < self.num_colonies {
            self.name_to_id.insert(name.clone(), colony_id);
            self.colony_names[idx] = name;
        }
    }
    
    /// Set neighbor colony in the given direction
    pub fn set_neighbor(&mut self, colony_id: ColonyId, direction: Direction, neighbor_id: ColonyId) {
        let idx = colony_id as usize;
        if idx >= self.num_colonies || neighbor_id as usize >= self.num_colonies {
            return;
        }
        
        match direction {
            Direction::North => self.colony_north[idx] = Some(neighbor_id),
            Direction::South => self.colony_south[idx] = Some(neighbor_id),
            Direction::East => self.colony_east[idx] = Some(neighbor_id),
            Direction::West => self.colony_west[idx] = Some(neighbor_id),
        }
    }
    
    /// Update valid direction bitmasks for all colonies
    pub fn update_all_valid_directions(&mut self) {
        for colony_id in 0..self.num_colonies {
            self.update_valid_directions(colony_id as ColonyId);
        }
    }
    
    /// Initialize ants at random positions
    pub fn initialize_ants(&mut self, num_ants: u16) {
        if self.num_colonies == 0 {
            return;
        }
        
        let num_ants = num_ants as usize;
        self.ant_colonies.resize(num_ants, 0);
        self.ant_alive.resize(num_ants, true);
        self.ant_moves.resize(num_ants, 0);
        
        // Place ants randomly in valid colonies
        for ant_id in 0..num_ants {
            loop {
                let colony_id = self.rng.next_range(self.num_colonies as u32) as ColonyId;
                if self.colony_valid[colony_id as usize] {
                    self.ant_colonies[ant_id] = colony_id;
                    
                    // Update colony occupancy
                    let colony_idx = colony_id as usize;
                    self.colony_ant_count[colony_idx] += 1;
                    
                    // Always update first_ant - overwrite with latest ant ID
                    // This ensures we can always find at least one ant in the colony
                    self.colony_first_ant[colony_idx] = Some(ant_id as AntId);
                    
                    break;
                }
            }
        }
    }
}