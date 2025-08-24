/// Fast XorShift RNG for performance-critical simulation
/// Much faster than standard cryptographic RNG
pub struct FastRng {
    pub state: u64,
}

impl FastRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }
    
    /// Generate next random u32 using optimized XorShift algorithm
    #[inline(always)]
    pub fn next_u32(&mut self) -> u32 {
        // Slightly faster XorShift variant
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x as u32
    }
    
    /// Generate random number in range [0, max) without modulo bias
    #[inline(always)]
    pub fn next_range(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        
        // Use rejection sampling to eliminate modulo bias
        // Calculate the largest multiple of max that fits in u32
        let threshold = (u32::MAX / max) * max;
        
        loop {
            let value = self.next_u32();
            if value < threshold {
                return value % max;
            }
            // Reject and try again - this happens rarely
        }
    }
}