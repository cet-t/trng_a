use crate::splitmix::SplitMix32;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Xorwow {
    x: [u32; 5],
    c: u32,
}

impl From<u32> for Xorwow {
    fn from(value: u32) -> Self {
        let mut s = SplitMix32::new(value);

        Self {
            x: [s.next(), s.next(), s.next(), s.next(), s.next()],
            c: s.next(),
        }
    }
}

impl trng::Rng for Xorwow {
    type Word = u32;

    #[inline]
    fn nextu(&mut self) -> Self::Word {
        let mut t = self.x[4];

        let s = self.x[0];
        self.x[4] = self.x[3];
        self.x[3] = self.x[2];
        self.x[2] = self.x[1];
        self.x[1] = s;

        t ^= t.wrapping_shr(2);
        t ^= t.wrapping_shl(3);
        t ^= s ^ s.wrapping_shl(4);

        self.x[0] = t;
        self.c = self.c.wrapping_add(362437);
        t.wrapping_add(self.c)
    }
}
