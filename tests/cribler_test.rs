#![cfg(any(feature = "biski", feature = "sfmt"))]

use cribler_practrand::{Outcome, PractRand};
use cribler_testu01::{Battery, TestU01};

const SEED: u32 = 0xf490_172b;

// Feed both halves of a 64-bit output, low word first, without discarding bits.
fn words(mut next: impl FnMut() -> u64, bits: u32) -> impl FnMut() -> u32 {
    let mut high = None;
    move || {
        if let Some(word) = high.take() {
            return word;
        }
        let word = next();
        if bits == 64 {
            high = Some((word >> 32) as u32);
        }
        word as u32
    }
}

fn run_testu01(name: &str, generator: impl FnMut() -> u32, battery: Battery) {
    let report = TestU01::new(battery)
        .run(generator)
        .unwrap_or_else(|error| panic!("{name}: {error:#?}"));
    println!("{name}: {report:#?}");
    assert!(!report.tests.is_empty(), "{name}: no TestU01 results");
    assert!(
        report.tests.iter().all(|test| test.p_value.is_some()),
        "{name}: unavailable TestU01 p-value"
    );
    assert!(
        !report.has_suspicious_results(),
        "{name}: suspicious TestU01 results: {:?}",
        report.suspicious_results().collect::<Vec<_>>()
    );
}

fn run_practrand(name: &str, generator: impl FnMut() -> u32) {
    let report = PractRand::new()
        .run(generator)
        .unwrap_or_else(|error| panic!("{name}: {error:#?}"));
    println!("{name}: {report:#?}");
    assert_ne!(report.outcome, Outcome::Unknown, "{name}: unknown output");
    assert!(!report.has_failures(), "{name}: PractRand FAIL");
    assert_eq!(
        report.processed_bytes,
        Some(report.requested_bytes),
        "{name}: incomplete PractRand run"
    );
}

macro_rules! rng_test {
    ($test:ident, $rng:ty, $seed:ty, $bits:expr, $run:expr) => {
        #[test]
        #[ignore = "requires external statistical engine; run explicitly in release mode"]
        fn $test() {
            use trng::Rng;

            let mut rng = <$rng>::from(SEED as $seed);
            ($run)(stringify!($rng), words(move || rng.nextu() as u64, $bits));
        }
    };
}

macro_rules! suite {
    ($module:ident, $run:expr) => {
        mod $module {
            use super::*;

            #[cfg(feature = "biski")]
            rng_test!(biski64_test, trng_a::Biski64, u64, 64, $run);

            #[cfg(feature = "sfmt")]
            mod sfmt {
                use super::*;

                rng_test!(sfmt607_test, trng_a::Sfmt607, u32, 32, $run);
                rng_test!(sfmt1279_test, trng_a::Sfmt1279, u32, 32, $run);
                rng_test!(sfmt2281_test, trng_a::Sfmt2281, u32, 32, $run);
                rng_test!(sfmt4253_test, trng_a::Sfmt4253, u32, 32, $run);
                rng_test!(sfmt11213_test, trng_a::Sfmt11213, u32, 32, $run);
                rng_test!(sfmt19937_test, trng_a::Sfmt19937, u32, 32, $run);
                rng_test!(sfmt44497_test, trng_a::Sfmt44497, u32, 32, $run);
                rng_test!(sfmt86243_test, trng_a::Sfmt86243, u32, 32, $run);
                rng_test!(sfmt132049_test, trng_a::Sfmt132049, u32, 32, $run);
                rng_test!(sfmt216091_test, trng_a::Sfmt216091, u32, 32, $run);
            }

            #[cfg(feature = "xorwow")]
            rng_test!(xorwow_test, trng_a::Xorwow, u32, 32, $run);

            #[cfg(feature = "sfc")]
            mod sfc {
                use super::*;

                rng_test!(sfc32_test, trng_a::Sfc32, u32, 32, $run);
                rng_test!(sfc64_test, trng_a::Sfc64, u64, 64, $run);
            }
        }
    };
}

// Set TESTU01_RUNNER to an installed cribler-testu01-runner executable.
// cargo test --release --all-features --test cribler_test small_crush:: -- --ignored --nocapture --test-threads=1
suite!(small_crush, |name, generator| run_testu01(
    name,
    generator,
    Battery::SmallCrush
));
suite!(crush, |name, generator| run_testu01(
    name,
    generator,
    Battery::Crush
));
suite!(big_crush, |name, generator| run_testu01(
    name,
    generator,
    Battery::BigCrush
));

// Set PRACTRAND_EXE, or put RNG_test on PATH. Uses the default 1 GiB limit.
// cargo test --release --all-features --test cribler_test practrand:: -- --ignored --nocapture --test-threads=1
suite!(practrand, run_practrand);
