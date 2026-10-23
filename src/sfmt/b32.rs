use crate::splitmix::SplitMix32;

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn sfmt_recursion(
    a: [u32; 4],
    b: [u32; 4],
    r1: [u32; 4],
    r2: [u32; 4],
    mask: [u32; 4],
    sl1: u32,
    sl2: u32,
    sr1: u32,
    sr2: u32,
) -> [u32; 4] {
    #[inline(always)]
    fn xor4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
        [a[0] ^ b[0], a[1] ^ b[1], a[2] ^ b[2], a[3] ^ b[3]]
    }

    #[inline(always)]
    fn and4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
        [a[0] & b[0], a[1] & b[1], a[2] & b[2], a[3] & b[3]]
    }

    #[inline(always)]
    fn shl4(a: [u32; 4], shift: u32) -> [u32; 4] {
        [a[0] << shift, a[1] << shift, a[2] << shift, a[3] << shift]
    }

    #[inline(always)]
    fn shr4(a: [u32; 4], shift: u32) -> [u32; 4] {
        [a[0] >> shift, a[1] >> shift, a[2] >> shift, a[3] >> shift]
    }

    #[inline(always)]
    fn pack128(a: [u32; 4]) -> u128 {
        // SFMT treats lane 0 as the least significant word on every target.
        u128::from(a[0])
            | (u128::from(a[1]) << 32)
            | (u128::from(a[2]) << 64)
            | (u128::from(a[3]) << 96)
    }

    #[inline(always)]
    fn unpack128(a: u128) -> [u32; 4] {
        [
            a as u32,
            (a >> 32) as u32,
            (a >> 64) as u32,
            (a >> 96) as u32,
        ]
    }

    #[inline(always)]
    fn lshift128(a: [u32; 4], bytes: u32) -> [u32; 4] {
        unpack128(pack128(a) << (bytes * 8))
    }

    #[inline(always)]
    fn rshift128(a: [u32; 4], bytes: u32) -> [u32; 4] {
        unpack128(pack128(a) >> (bytes * 8))
    }

    xor4(
        xor4(xor4(a, lshift128(a, sl2)), and4(shr4(b, sr1), mask)),
        xor4(rshift128(r1, sr2), shl4(r2, sl1)),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct Sfmt19937 {
    state: [[u32; 4]; SFMT_N],
    idx: usize,
}

const SFMT_N: usize = 156;
const SFMT_POS1: usize = 122;
const SFMT_SL1: u32 = 18;
const SFMT_SR1: u32 = 11;

const SFMT_MSK: [u32; 4] = [0xdfffffef, 0xddfecb7f, 0xbffaffff, 0xbffffff6];
const SFMT_PARITY: [u32; 4] = [0x00000001, 0x00000000, 0x00000000, 0x13c9e684];

impl From<u32> for Sfmt19937 {
    fn from(value: u32) -> Self {
        let mut seedgen = SplitMix32::new(value);

        let mut raw_state = [0u32; SFMT_N << 2];
        for i in 0..(SFMT_N << 1) {
            raw_state[2 * i] = seedgen.next();
            raw_state[2 * i + 1] = seedgen.next();
        }

        let mut state = [[0u32; 4]; SFMT_N];
        for i in 0..SFMT_N {
            state[i] = [
                raw_state[4 * i],
                raw_state[4 * i + 1],
                raw_state[4 * i + 2],
                raw_state[4 * i + 3],
            ];
        }

        let mut rng = Self {
            state,
            idx: SFMT_N << 2,
        };

        rng.period_certification();
        rng
    }
}

impl Sfmt19937 {
    fn gen_rand_all(&mut self) {
        unsafe {
            let ptr = self.state.as_mut_ptr();
            let mut r1 = *ptr.add(SFMT_N - 2);
            let mut r2 = *ptr.add(SFMT_N - 1);

            let mask = SFMT_MSK;

            for i in 0..(SFMT_N - SFMT_POS1) {
                let p_i = ptr.add(i);
                let a = *p_i;
                let b = *ptr.add(i + SFMT_POS1);
                let r = sfmt_recursion(a, b, r1, r2, mask, SFMT_SL1, 1, SFMT_SR1, 1);

                *p_i = r;
                r1 = r2;
                r2 = r;
            }

            for i in (SFMT_N - SFMT_POS1)..SFMT_N {
                let p_i = ptr.add(i);
                let a = *p_i;
                let b = *ptr.add(i + SFMT_POS1 - SFMT_N);
                let r = sfmt_recursion(a, b, r1, r2, mask, SFMT_SL1, 1, SFMT_SR1, 1);

                *p_i = r;
                r1 = r2;
                r2 = r;
            }
        }
    }

    fn period_certification(&mut self) {
        let mut inner = 0;
        let psfmt32 =
            unsafe { core::slice::from_raw_parts(self.state.as_ptr() as *const u32, SFMT_N << 2) };
        const PARITY: [u32; 4] = SFMT_PARITY;

        for i in 0..4 {
            inner ^= psfmt32[i] & PARITY[i];
        }
        let mut i = 16;
        while i > 0 {
            inner ^= inner >> i;
            i >>= 1;
        }
        inner &= 1;

        // Verification passed
        if inner == 1 {
            return;
        }

        // Modification for period certification
        let psfmt32_mut = unsafe {
            core::slice::from_raw_parts_mut(self.state.as_mut_ptr() as *mut u32, SFMT_N << 2)
        };

        for i in 0..4 {
            let mut work = 1;
            for _ in 0..32 {
                if (work & PARITY[i]) != 0 {
                    psfmt32_mut[i] ^= work;
                    return;
                }
                work <<= 1;
            }
        }
    }
}

impl trng::Rng for Sfmt19937 {
    type Word = u32;

    #[inline]
    fn nextu(&mut self) -> Self::Word {
        if self.idx >= SFMT_N * 4 {
            self.gen_rand_all();
            self.idx = 0;
        }

        let s: &[u32] = bytemuck::cast_slice(&self.state);
        let val = s[self.idx];
        self.idx += 1;
        val
    }
}

macro_rules! define_sfmt_variant {
    (
        mexp = $mexp:literal,
        n = $n:literal,
        pos1 = $pos1:literal,
        sl1 = $sl1:literal,
        sl2 = $sl2:literal,
        sr1 = $sr1:literal,
        sr2 = $sr2:literal,
        msk1 = $msk1:expr,
        msk2 = $msk2:expr,
        msk3 = $msk3:expr,
        msk4 = $msk4:expr,
        parity1 = $parity1:expr,
        parity2 = $parity2:expr,
        parity3 = $parity3:expr,
        parity4 = $parity4:expr $(,)?
    ) => {
        ::pastey::paste! {
            #[derive(Debug, Clone, Copy)]
            pub struct [<Sfmt $mexp>] {
                state: [[u32; 4]; $n],
                idx: usize,
            }

            impl From<u32> for [<Sfmt $mexp>] {
                fn from(value: u32) -> Self {
                    let mut seedgen = SplitMix32::new(value);
                    let mut raw_state = [0u32; $n << 2];
                    for i in 0..($n * 2) {
                        raw_state[2 * i]     = seedgen.next();
                        raw_state[2 * i + 1] = seedgen.next();
                    }
                    let mut state = [[0u32; 4]; $n];
                    for i in 0..$n {
                        state[i] = [
                            raw_state[4 * i],
                            raw_state[4 * i + 1],
                            raw_state[4 * i + 2],
                            raw_state[4 * i + 3],
                        ];
                    }
                    let mut rng = Self { state, idx: $n << 2 };
                    rng.period_certification();
                    rng
                }
            }

            impl [<Sfmt $mexp>] {
                fn gen_rand_all(&mut self) {
                    unsafe {
                        let ptr = self.state.as_mut_ptr();
                        let mut r1 = *ptr.add($n - 2);
                        let mut r2 = *ptr.add($n - 1);
                        let mask = [$msk1, $msk2, $msk3, $msk4];

                        for i in 0..($n - $pos1) {
                            let p_i = ptr.add(i);
                            let a = *p_i;
                            let b = *ptr.add(i + $pos1);
                            let r = sfmt_recursion(
                                a,
                                b,
                                r1,
                                r2,
                                mask,
                                $sl1 as u32,
                                $sl2 as u32,
                                $sr1 as u32,
                                $sr2 as u32,
                            );
                            *p_i = r;
                            r1 = r2;
                            r2 = r;
                        }

                        for i in ($n - $pos1)..$n {
                            let p_i = ptr.add(i);
                            let a = *p_i;
                            let b = *ptr.add(i + $pos1 - $n);
                            let r = sfmt_recursion(
                                a,
                                b,
                                r1,
                                r2,
                                mask,
                                $sl1 as u32,
                                $sl2 as u32,
                                $sr1 as u32,
                                $sr2 as u32,
                            );
                            *p_i = r;
                            r1 = r2;
                            r2 = r;
                        }
                    }
                }

                fn period_certification(&mut self) {
                    let mut inner = 0u32;
                    let psfmt32 = unsafe {
                        core::slice::from_raw_parts(self.state.as_ptr() as *const u32, $n << 2)
                    };
                    let parity = [$parity1, $parity2, $parity3, $parity4];
                    for i in 0..4 {
                        inner ^= psfmt32[i] & parity[i];
                    }
                    let mut shift = 16u32;
                    while shift > 0 {
                        inner ^= inner >> shift;
                        shift >>= 1;
                    }
                    inner &= 1;
                    if inner == 1 {
                        return;
                    }
                    let psfmt32_mut = unsafe {
                        core::slice::from_raw_parts_mut(self.state.as_mut_ptr() as *mut u32, $n << 2)
                    };
                    for i in 0..4 {
                        let mut work = 1u32;
                        for _ in 0..32 {
                            if (work & parity[i]) != 0 {
                                psfmt32_mut[i] ^= work;
                                return;
                            }
                            work <<= 1;
                        }
                    }
                }
            }


            impl trng::Rng for [<Sfmt $mexp>] {
                type Word = u32;

                #[inline]
                fn nextu(&mut self) -> Self::Word {
                    if self.idx >= $n << 2 {
                        self.gen_rand_all();
                        self.idx = 0;
                    }
                    let s: &[u32] = bytemuck::cast_slice(&self.state);
                    let val = s[self.idx];
                    self.idx += 1;
                    val
                }
            }
        }
    };
}

define_sfmt_variant!(
    mexp = 607,
    n = 5,
    pos1 = 2,
    sl1 = 15,
    sl2 = 3,
    sr1 = 13,
    sr2 = 3,
    msk1 = 0xfdff37ffu32,
    msk2 = 0xef7f3f7du32,
    msk3 = 0xff777b7du32,
    msk4 = 0x7ff7fb2fu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0x00000000u32,
    parity4 = 0x5986f054u32,
);

define_sfmt_variant!(
    mexp = 1279,
    n = 10,
    pos1 = 7,
    sl1 = 14,
    sl2 = 3,
    sr1 = 5,
    sr2 = 1,
    msk1 = 0xf7fefffdu32,
    msk2 = 0x7fefcfffu32,
    msk3 = 0xaff3ef3fu32,
    msk4 = 0xb5ffff7fu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0x00000000u32,
    parity4 = 0x20000000u32,
);

define_sfmt_variant!(
    mexp = 2281,
    n = 18,
    pos1 = 12,
    sl1 = 19,
    sl2 = 1,
    sr1 = 5,
    sr2 = 1,
    msk1 = 0xbff7ffbfu32,
    msk2 = 0xfdfffffeu32,
    msk3 = 0xf7ffef7fu32,
    msk4 = 0xf2f7cbbfu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0x00000000u32,
    parity4 = 0x41dfa600u32,
);

define_sfmt_variant!(
    mexp = 4253,
    n = 34,
    pos1 = 17,
    sl1 = 20,
    sl2 = 1,
    sr1 = 7,
    sr2 = 1,
    msk1 = 0x9f7bffffu32,
    msk2 = 0x9fffff5fu32,
    msk3 = 0x3efffffbu32,
    msk4 = 0xfffff7bbu32,
    parity1 = 0xa8000001u32,
    parity2 = 0xaf5390a3u32,
    parity3 = 0xb740b3f8u32,
    parity4 = 0x6c11486du32,
);

define_sfmt_variant!(
    mexp = 11213,
    n = 88,
    pos1 = 68,
    sl1 = 14,
    sl2 = 3,
    sr1 = 7,
    sr2 = 3,
    msk1 = 0xeffff7fbu32,
    msk2 = 0xffffffefu32,
    msk3 = 0xdfdfbfffu32,
    msk4 = 0x7fffdbfdu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0xe8148000u32,
    parity4 = 0xd0c7afa3u32,
);

define_sfmt_variant!(
    mexp = 44497,
    n = 348,
    pos1 = 330,
    sl1 = 5,
    sl2 = 3,
    sr1 = 9,
    sr2 = 3,
    msk1 = 0xeffffffbu32,
    msk2 = 0xdfbebfffu32,
    msk3 = 0xbfbf7befu32,
    msk4 = 0x9ffd7bffu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0xa3ac4000u32,
    parity4 = 0xecc1327au32,
);

define_sfmt_variant!(
    mexp = 86243,
    n = 674,
    pos1 = 366,
    sl1 = 6,
    sl2 = 7,
    sr1 = 19,
    sr2 = 1,
    msk1 = 0xfdbffbffu32,
    msk2 = 0xbff7ff3fu32,
    msk3 = 0xfd77efffu32,
    msk4 = 0xbf9ff3ffu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0x00000000u32,
    parity4 = 0xe9528d85u32,
);

define_sfmt_variant!(
    mexp = 132049,
    n = 1032,
    pos1 = 110,
    sl1 = 19,
    sl2 = 1,
    sr1 = 21,
    sr2 = 1,
    msk1 = 0xffffbb5fu32,
    msk2 = 0xfb6ebf95u32,
    msk3 = 0xfffefffau32,
    msk4 = 0xcff77fffu32,
    parity1 = 0x00000001u32,
    parity2 = 0x00000000u32,
    parity3 = 0xcb520000u32,
    parity4 = 0xc7e91c7du32,
);

define_sfmt_variant!(
    mexp = 216091,
    n = 1689,
    pos1 = 627,
    sl1 = 11,
    sl2 = 3,
    sr1 = 10,
    sr2 = 1,
    msk1 = 0xbff7bff7u32,
    msk2 = 0xbfffffffu32,
    msk3 = 0xbffffa7fu32,
    msk4 = 0xffddfbfbu32,
    parity1 = 0xf8000001u32,
    parity2 = 0x89e80709u32,
    parity3 = 0x3bd2b64bu32,
    parity4 = 0x0c64b1e4u32,
);
