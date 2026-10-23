use crate::splitmix::SplitMix32;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sfc32 {
    a: u32,
    b: u32,
    c: u32,
    counter: u32,
}

impl From<u32> for Sfc32 {
    fn from(value: u32) -> Self {
        let mut s = SplitMix32::new(value);

        Self {
            a: s.next(),
            b: s.next(),
            c: s.next(),
            counter: 1,
        }
    }
}

impl trng::Rng for Sfc32 {
    type Word = u32;

    #[inline]
    fn nextu(&mut self) -> Self::Word {
        let t = self.a.wrapping_add(self.b).wrapping_add(self.counter);

        self.counter = self.counter.wrapping_add(1);
        self.a = self.b ^ self.b.wrapping_shr(9);
        self.b = self.c ^ self.c.wrapping_shl(3);
        self.c = self.c.rotate_right(11);
        self.c = self.c.wrapping_add(t);

        t
    }
}
