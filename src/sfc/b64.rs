use crate::splitmix::SplitMix64;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sfc64 {
    a: u64,
    b: u64,
    c: u64,
    counter: u64,
}

impl From<u64> for Sfc64 {
    fn from(value: u64) -> Self {
        let mut s = SplitMix64::new(value);

        Self {
            a: s.next(),
            b: s.next(),
            c: s.next(),
            counter: 1,
        }
    }
}

impl trng::Rng for Sfc64 {
    type Word = u64;

    #[inline]
    fn nextu(&mut self) -> Self::Word {
        let t = self.a.wrapping_add(self.b).wrapping_add(self.counter);

        self.counter = self.counter.wrapping_add(1);
        self.a = self.b ^ self.b.wrapping_shr(11);
        self.b = self.c ^ self.c.wrapping_shl(3);
        self.c = self.c.rotate_right(24);
        self.c = self.c.wrapping_add(t);

        t
    }
}
