const STEP: u64 = 0x9E37_79B9_7F4A_7C15;
const A: u64 = 0xBF58_476D_1CE4_E5B9;
const B: u64 = 0x94D0_49BB_1331_11EB;

#[derive(Debug, Clone, Copy)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline(always)]
    pub const fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(STEP);

        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(A);
        z = (z ^ (z >> 27)).wrapping_mul(B);
        z ^ (z >> 31)
    }
}
