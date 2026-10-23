const STEP: u32 = 0x9E37_79B9;
const A: u32 = 0x85EB_CA6B;
const B: u32 = 0xC2B2_AE35;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplitMix32 {
    state: u32,
}

impl SplitMix32 {
    pub const fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub const fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_add(STEP);

        let mut z = self.state;
        z = (z ^ (z >> 16)).wrapping_mul(A);
        z = (z ^ (z >> 13)).wrapping_mul(B);
        z ^ (z >> 16)
    }
}
