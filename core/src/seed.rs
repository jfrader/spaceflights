use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn from_text(input: &str) -> Self {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in input.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0001_0000_01b3);
        }
        Self(hash)
    }

    #[must_use]
    pub fn nth_value(self, index: u64) -> u64 {
        splitmix64(
            self.0
                .wrapping_add(index.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
        )
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::Seed;

    #[test]
    fn seed_from_text_is_deterministic() {
        let a = Seed::from_text("spaceflights");
        let b = Seed::from_text("spaceflights");
        let c = Seed::from_text("another");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn nth_value_is_deterministic_per_index() {
        let seed = Seed::new(12345);
        let first = seed.nth_value(0);
        let second = seed.nth_value(1);

        assert_eq!(first, seed.nth_value(0));
        assert_eq!(second, seed.nth_value(1));
        assert_ne!(first, second);
    }
}
