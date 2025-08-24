/// Core types used throughout the simulation
pub type ColonyId = u16;
pub type AntId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

impl Direction {
    pub fn as_bit_mask(self) -> u8 {
        1 << (self as u8)
    }
}

impl std::str::FromStr for Direction {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "north" => Ok(Direction::North),
            "south" => Ok(Direction::South),
            "east" => Ok(Direction::East),
            "west" => Ok(Direction::West),
            _ => Err(()),
        }
    }
}

/// Simulation parameters
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub max_moves: u16,
    pub num_ants: u16,
    pub map_file: String,
    pub seed: Option<u64>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_moves: 10000,
            num_ants: 0,
            map_file: String::new(),
            seed: None,
        }
    }
}