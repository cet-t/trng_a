use crate::splitmix::b64::SplitMix64;

/// https://github.com/danielcota/biski64/blob/main/src/lib.rs
#[derive(Debug, Clone, Copy)]
pub struct Biski64 {
    fast_loop: u64,
    mix: u64,
    loop_mix: u64,
}

impl From<u64> for Biski64 {
    fn from(value: u64) -> Self {
        // Create a SplitMix64 instance to generate our actual state.
        let mut seeder = SplitMix64::new(value);

        // Ensure the initial state is not all zero, which can be a bad state.
        let mut s0 = 0;
        let mut s1 = 0;
        let mut s2 = 0;
        while s0 == 0 && s1 == 0 && s2 == 0 {
            s0 = seeder.next();
            s1 = seeder.next();
            s2 = seeder.next();
        }

        // Construct the initial RNG instance
        let mut rng = Self {
            fast_loop: s0,
            mix: s1,
            loop_mix: s2,
        };

        // Warm-up period: 16 iterations to further diffuse the initial state.
        // This helps ensure that the initial outputs are well mixed.
        for _ in 0..16 {
            let _ = trng::Rng::nextu(&mut rng);
        }

        // Return the warmed-up RNG
        rng
    }
}

impl trng::Rng for Biski64 {
    type Word = u64;

    fn nextu(&mut self) -> Self::Word {
        let output = self.mix.wrapping_add(self.loop_mix);

        (self.fast_loop, self.mix, self.loop_mix) = (
            self.fast_loop.wrapping_add(0x9999999999999999),
            self.mix
                .rotate_left(16)
                .wrapping_add(self.loop_mix.rotate_left(40)),
            self.fast_loop ^ self.mix,
        );

        output
    }
}
