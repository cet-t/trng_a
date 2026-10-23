#![cfg(feature = "sfmt")]

use trng::Rng;

// Expected FNV-1a hashes of little-endian output bytes from the official scalar
// MersenneTwister-Lab/SFMT C implementation (master archive, 2026-10-22).
// Its initial state was filled with this crate's SplitMix32, then certified by
// the official period_certification(). Standard sfmt_init_gen_rand uses a
// different initializer and intentionally does not define these test vectors.
// Each case consumes three whole state blocks plus 17 words, exercising refills.
// Both certification branches are represented for every parameter set.
// Source: https://github.com/MersenneTwister-Lab/SFMT
// SFMT.c SHA-256: 9bd208c9842a0747c6f669d3a24ebc93b9f3fc6e86ca9328c0fbece119b6cedf
fn check<R: Rng<Word = u32> + From<u32>>(n: usize, cases: &[(u32, u64)]) {
    for &(seed, expected) in cases {
        let mut rng = R::from(seed);
        let mut hash = 0xcbf29ce484222325u64;
        for _ in 0..(n * 4 * 3 + 17) {
            for byte in rng.nextu().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        assert_eq!(hash, expected, "seed = {seed:#010x}");
    }
}

#[test]
fn sfmt607_matches_official_c() {
    check::<trng_a::Sfmt607>(
        5,
        &[
            (0x00000000, 0xde071314872c35c9),
            (0x00000001, 0x141ea4d6f2c6a4c0),
            (0x00000002, 0x3e74d1d535c2dba5),
            (0x000004d2, 0x3acfb9daab305587),
            (0xffffffff, 0x2d5b4f44e15fa019),
            (0x80000000, 0xd18274df1e728340),
            (0xdeadbeef, 0x21c8c6ab1629c80c),
            (0xf490172b, 0xdcefe8d1caaf43ff),
        ],
    );
}

#[test]
fn sfmt1279_matches_official_c() {
    check::<trng_a::Sfmt1279>(
        10,
        &[
            (0x00000000, 0xa94440b10a538763),
            (0x00000001, 0x8455907cbd109fb5),
            (0x00000002, 0x90eb1a485670199a),
            (0x000004d2, 0xe82797bf0bcfd247),
            (0xffffffff, 0x737416f9aa4e6551),
            (0x80000000, 0x8aebc876ffbee928),
            (0xdeadbeef, 0xfc79984e4aca5f25),
            (0xf490172b, 0x72dc334bddb5c0a1),
        ],
    );
}

#[test]
fn sfmt2281_matches_official_c() {
    check::<trng_a::Sfmt2281>(
        18,
        &[
            (0x00000000, 0x18174d6a1a2992a4),
            (0x00000001, 0x832f759d59788c67),
            (0x00000002, 0xfefdc5160d2bad84),
            (0x000004d2, 0x77e4178fc51aa199),
            (0xffffffff, 0x89a3bd9a330a89f9),
            (0x80000000, 0xfd5cce265deff8b1),
            (0xdeadbeef, 0xbc59862ff5b26025),
            (0xf490172b, 0xc944683965e860ae),
        ],
    );
}

#[test]
fn sfmt4253_matches_official_c() {
    check::<trng_a::Sfmt4253>(
        34,
        &[
            (0x00000000, 0x8a02e243c1ab298c),
            (0x00000001, 0xcfebae371627e8c4),
            (0x00000002, 0xbf844fc235532672),
            (0x000004d2, 0x3aefb72147e7c9d2),
            (0xffffffff, 0x57241019849552c5),
            (0x80000000, 0x5d3192c717f6f844),
            (0xdeadbeef, 0xe4e114de4ff3d2cc),
            (0xf490172b, 0xc10e60b3cfd97a23),
        ],
    );
}

#[test]
fn sfmt11213_matches_official_c() {
    check::<trng_a::Sfmt11213>(
        88,
        &[
            (0x00000000, 0x3801a0f29da1a619),
            (0x00000001, 0x4e7126c9f338dc6f),
            (0x00000002, 0x25ac587f6517a15f),
            (0x000004d2, 0x588b6da2042a7544),
            (0xffffffff, 0x5e408434fc285446),
            (0x80000000, 0x2c5ba2c666c775d2),
            (0xdeadbeef, 0xcc0a2070541f43e6),
            (0xf490172b, 0xf75c6d3cfb48ce16),
        ],
    );
}

#[test]
fn sfmt19937_matches_official_c() {
    check::<trng_a::Sfmt19937>(
        156,
        &[
            (0x00000000, 0x3725cdde9ca1c305),
            (0x00000001, 0xe23c2976fc95d4f2),
            (0x00000002, 0x4301ffba17253ae3),
            (0x000004d2, 0xc0b87ffdb01a5121),
            (0xffffffff, 0xa190cf43fa709347),
            (0x80000000, 0xdffeab208fd13712),
            (0xdeadbeef, 0x9b8a4a496bc49e0a),
            (0xf490172b, 0xeb89f53e435d2fe8),
        ],
    );
}

#[test]
fn sfmt44497_matches_official_c() {
    check::<trng_a::Sfmt44497>(
        348,
        &[
            (0x00000000, 0xa68085f8b1e834b4),
            (0x00000001, 0x57138601557ac7e0),
            (0x00000002, 0xff95ae7b1ea32a85),
            (0x000004d2, 0x9f36e204138c511b),
            (0xffffffff, 0x601c1dfbb78b9276),
            (0x80000000, 0xaf299a15dea8f079),
            (0xdeadbeef, 0xb2d48390264dbb9b),
            (0xf490172b, 0x19e37ceba8239ded),
        ],
    );
}

#[test]
fn sfmt86243_matches_official_c() {
    check::<trng_a::Sfmt86243>(
        674,
        &[
            (0x00000000, 0x7fd844e1cee076e5),
            (0x00000001, 0xc762a7d8e8e3e456),
            (0x00000002, 0x81f286f6b23f979c),
            (0x000004d2, 0xc0d48e6d5fbe5f1a),
            (0xffffffff, 0x6732d5e418fb9596),
            (0x80000000, 0x810ff316b7b1ee86),
            (0xdeadbeef, 0x45ad9d029c845053),
            (0xf490172b, 0x481308bc3438b399),
        ],
    );
}

#[test]
fn sfmt132049_matches_official_c() {
    check::<trng_a::Sfmt132049>(
        1032,
        &[
            (0x00000000, 0xb66683d00d17bec2),
            (0x00000001, 0xf0a3183cf8cbf28f),
            (0x00000002, 0xf353068f3e557f46),
            (0x000004d2, 0x2e7aecaede70abd8),
            (0xffffffff, 0x63c9acfd534571ba),
            (0x80000000, 0xb807eadaa4918708),
            (0xdeadbeef, 0xc73ed707976f1796),
            (0xf490172b, 0xcc35b0d7533a6ae1),
        ],
    );
}

#[test]
fn sfmt216091_matches_official_c() {
    check::<trng_a::Sfmt216091>(
        1689,
        &[
            (0x00000000, 0x70749b0c8b58bcec),
            (0x00000001, 0xb598d6867e1927ef),
            (0x00000002, 0x3174a0d616286cac),
            (0x000004d2, 0x901457e9c7b0457e),
            (0xffffffff, 0x2d8b6fb7ff87dbb3),
            (0x80000000, 0xc394fb49d03416e0),
            (0xdeadbeef, 0x9d0cf0a73fcea175),
            (0xf490172b, 0x6bb98f4a3b2b57c6),
        ],
    );
}
