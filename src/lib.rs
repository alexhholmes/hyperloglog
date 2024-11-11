use std::hash::{DefaultHasher, Hash, Hasher};
use std::cmp::max;

#[derive(Copy, Clone, Debug)]
enum Cardinality {
    Evaluated(f64),
    Expired,
}

#[derive(Clone, Debug)]
pub struct HyperLogLog<H: Hasher> {
    hasher: H,
    registers: Vec<u8>,
    cardinality: Cardinality,
}

impl<H: Hasher> HyperLogLog<H> {
    pub fn new_with_hasher(hasher: H) -> Self {
        Self {
            hasher,
            registers: vec![0; 1 << 8],
            cardinality: Cardinality::Expired,
        }
    }

    pub fn insert<V: Hash>(&mut self, elem: V) {
        elem.hash(&mut self.hasher);
        let hash = self.hasher.finish();
        let register = ((0xFF & hash) >> 56) as usize;
        self.registers[register] = max(self.registers[register], hash.leading_zeros() as u8);
        self.cardinality = Cardinality::Expired;
    }

    pub fn merge(&mut self, other: &Self) {
        for (idx, reg) in other.registers.iter().enumerate() {
            self.registers[idx] = max(self.registers[idx], *reg)
        }
    }

    pub fn evaluate(&mut self) -> f64 {
        match self.cardinality {
            Cardinality::Evaluated(eval) => eval,
            Cardinality::Expired => {
                let sum: f64 = self.registers
                    .iter()
                    .map(|&x| x as f64)
                    .sum();
                let avg = sum / (1 << 8) as f64;
                self.cardinality = Cardinality::Evaluated(avg);
                avg
            }
        }
    }
}

impl HyperLogLog<DefaultHasher> {
    pub fn new() -> Self {
        Self::new_with_hasher(DefaultHasher::new())
    }
}

impl Default for HyperLogLog<DefaultHasher> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut hll = HyperLogLog::new();
        hll.insert("test1".as_bytes());
        println!("{}", hll.evaluate())
    }
}
