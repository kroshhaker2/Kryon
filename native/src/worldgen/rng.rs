const GOLDEN_RATIO_64: u64 = 0x9E3779B97F4A7C15;
const SILVER_RATIO_64: u64 = 0x6A09E667F3BCC909;

pub struct Xoroshiro {
    seed_lo: u64,
    seed_hi: u64,
}

impl Xoroshiro {
    pub fn from_seed(&self, seed: i64) -> Self {
        let seed = seed as u64;

        Self {
            seed_lo: seed ^ self.seed_lo,
            seed_hi: seed ^ self.seed_hi,
        }
    }

    pub fn next_long(&mut self) -> i64 {
        todo!()
    }
}

#[test]
fn same_seed_is_deterministic() {
    let mut first = Xoroshiro::from_seed(12345);
    let mut second = Xoroshiro::from_seed(12345);

    assert_eq!(first.next_long(), second.next_long());
}
