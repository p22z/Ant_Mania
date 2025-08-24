use std::env;
use crate::types::SimulationConfig;

const MAX_MOVES: u16 = 10_000;

#[derive(Debug)]
pub enum ParseError {
    InvalidUsage(String),
    InvalidAntCount(String),
    FileNotFound(String),
    InvalidSeed(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidUsage(msg) => write!(f, "Usage error: {}", msg),
            ParseError::InvalidAntCount(val) => write!(f, "Invalid number of ants: {}", val),
            ParseError::FileNotFound(path) => write!(f, "Map file does not exist: {}", path),
            ParseError::InvalidSeed(val) => write!(f, "Invalid seed: {}", val),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_args() -> Result<SimulationConfig, ParseError> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        return Err(ParseError::InvalidUsage(
            format!("Usage: {} <num_ants> <map_file> [--seed N]", args[0])
        ));
    }
    
    let num_ants = args[1].parse().map_err(|_| {
        ParseError::InvalidAntCount(args[1].clone())
    })?;
    
    let map_file = args[2].clone();
    
    if !std::path::Path::new(&map_file).exists() {
        return Err(ParseError::FileNotFound(map_file));
    }
    
    let seed = if args.len() > 3 && args[3] == "--seed" && args.len() > 4 {
        Some(args[4].parse().map_err(|_| {
            ParseError::InvalidSeed(args[4].clone())
        })?)
    } else {
        None
    };
    
    Ok(SimulationConfig {
        num_ants,
        map_file,
        max_moves: MAX_MOVES,
        seed,
    })
}