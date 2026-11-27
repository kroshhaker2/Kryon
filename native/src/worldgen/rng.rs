const GOLDEN_RATIO_64: u64 = 0x9E3779B97F4A7C15;
const SILVER_RATIO_64: u64 = 0x6A09E667F3BCC909;

pub enum RandomSource {
    Xoroshiro(Xoroshiro),
}

impl RandomSource {
    pub fn new(kind: u8, seed: i64) -> Self {
        match kind {
            0 => Self::Xoroshiro(Xoroshiro::new(seed)),
            _ => panic!("Unknown rng"),
        }
    }

    pub fn from_seed(&self, seed: i64) -> Self {
        match self {
            Self::Xoroshiro(rng) => Self::Xoroshiro(rng.from_seed(seed)),
        }
    }

    pub fn next_int(&mut self) -> i32 {
        match self {
            Self::Xoroshiro(rng) => rng.next_int(),
        }
    }

    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        match self {
            Self::Xoroshiro(rng) => rng.next_int_bounded(bound),
        }
    }

    pub fn next_long(&mut self) -> i64 {
        match self {
            Self::Xoroshiro(rng) => rng.next_long(),
        }
    }

    pub fn next_boolean(&mut self) -> bool {
        match self {
            Self::Xoroshiro(rng) => rng.next_boolean(),
        }
    }

    pub fn next_float(&mut self) -> f32 {
        match self {
            Self::Xoroshiro(rng) => rng.next_float(),
        }
    }

    pub fn next_double(&mut self) -> f64 {
        match self {
            Self::Xoroshiro(rng) => rng.next_double(),
        }
    }

    pub fn next_gaussian(&mut self) -> f64 {
        match self {
            Self::Xoroshiro(rng) => rng.next_gaussian(),
        }
    }
}

pub struct Xoroshiro {
    pub seed_lo: u64,
    pub seed_hi: u64,
    cached_gaussian: Option<f64>,
}

impl Xoroshiro {
    fn new(seed: i64) -> Self {
        let mut seed_lo = seed as u64 ^ SILVER_RATIO_64;
        let mut seed_hi = seed_lo.wrapping_add(GOLDEN_RATIO_64);

        seed_lo = mix_stafford13(seed_lo);
        seed_hi = mix_stafford13(seed_hi);

        Self {
            seed_lo,
            seed_hi,
            cached_gaussian: None,
        }
    }

    pub fn from_seed(&self, seed: i64) -> Self {
        let seed = seed as u64;

        let (lo, hi) = (seed ^ self.seed_lo, seed ^ self.seed_hi);
        let (seed_lo, seed_hi) = if lo | hi == 0 {
            (GOLDEN_RATIO_64, SILVER_RATIO_64)
        } else {
            (lo, hi)
        };
        Self {
            seed_lo,
            seed_hi,
            cached_gaussian: None,
        }
    }

    pub fn next_int(&mut self) -> i32 {
        self.next_long() as i32
    }

    pub fn next_int_bounded(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        let bound = bound as u32;
        let mut product = self.next_int() as u32 as u64 * u64::from(bound);
        if (product as u32) < bound {
            let threshold = bound.wrapping_neg() % bound;
            while (product as u32) < threshold {
                product = self.next_int() as u32 as u64 * u64::from(bound);
            }
        }
        (product >> 32) as i32
    }

    pub fn next_long(&mut self) -> i64 {
        let lo = self.seed_lo;
        let hi = self.seed_hi;
        let result = lo.wrapping_add(hi).rotate_left(17).wrapping_add(lo);
        let hi = hi ^ lo;
        self.seed_lo = lo.rotate_left(49) ^ hi ^ (hi << 21);
        self.seed_hi = hi.rotate_left(28);
        result as i64
    }

    pub fn next_boolean(&mut self) -> bool {
        self.next_long() & 1 != 0
    }

    pub fn next_float(&mut self) -> f32 {
        (self.next_long() as u64 >> 40) as f32 / (1u32 << 24) as f32
    }

    pub fn next_double(&mut self) -> f64 {
        (self.next_long() as u64 >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(value) = self.cached_gaussian.take() {
            return value;
        }
        loop {
            let x = 2.0 * self.next_double() - 1.0;
            let y = 2.0 * self.next_double() - 1.0;
            let r = x * x + y * y;
            if r > 0.0 && r < 1.0 {
                let scale = (-2.0 * r.ln() / r).sqrt();
                self.cached_gaussian = Some(y * scale);
                return x * scale;
            }
        }
    }
}

fn mix_stafford13(mut a: u64) -> u64 {
    a = (a ^ a >> 30).wrapping_mul(0xBF58476D1CE4E5B9);
    a = (a ^ a >> 27).wrapping_mul(0x94D049BB133111EB);
    a ^ a >> 31
}
