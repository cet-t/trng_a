#![cfg(any(feature = "biski", feature = "sfmt"))]

use cribler_core::*;

const SEED: u32 = 0xf490_172b;
// Keep significance thresholds constant: harder suites gain sensitivity by
// drawing more samples, without increasing the false rejection rate.
const ALPHA: f64 = 0.0001;
const Z_LIMIT: f64 = 5.0;

#[derive(Clone, Copy)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
    Hell,
}

struct SuiteConfig {
    ks: KsConfig,
    mcpi: McPiConfig,
    nist: NistConfig,
    chisq: ChiSqConfig,
    serial: SerialConfig,
    runs: RunsConfig,
    birthday: BirthdayConfig,
    paranoid: ParanoidConfig,
    paranoid_trial: ChiSqConfig,
}

// Hell is opt-in because KS stores and sorts all samples in memory.
// Defaults: 100 million samples, 10,000 trials of 1 million samples each.
// Override these values to match the available memory and execution time.
fn hell_setting(name: &str, default: usize, minimum: usize) -> usize {
    let value = match std::env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("{name} must be a positive integer")),
        Err(std::env::VarError::NotPresent) => default,
        Err(error) => panic!("{name}: {error}"),
    };
    assert!(value >= minimum, "{name} must be at least {minimum}");
    value
}

impl Difficulty {
    fn config(self) -> SuiteConfig {
        let (samples, birthday_points, birthday_bits, trials, trial_samples) = match self {
            Self::Easy => (100_000, 2_048, 15, 20, 5_000),
            Self::Normal => (1_000_000, 4_096, 16, 100, 10_000),
            Self::Hard => (10_000_000, 8_192, 16, 1_000, 100_000),
            Self::Hell => (
                hell_setting("CRIBLER_HELL_SAMPLES", 100_000_000, 10_000),
                16_384,
                16,
                hell_setting("CRIBLER_HELL_TRIALS", 10_000, 2),
                hell_setting("CRIBLER_HELL_TRIAL_SAMPLES", 1_000_000, 10_000),
            ),
        };

        SuiteConfig {
            ks: KsConfig {
                samples,
                alpha: ALPHA,
            },
            mcpi: McPiConfig {
                pairs: samples,
                // Scale the allowed error with the sampling standard error.
                max_error_pct: 0.3 * (1_000_000.0 / samples as f64).sqrt(),
            },
            nist: NistConfig {
                bits: samples,
                alpha: ALPHA,
                block_freq_m: 128,
            },
            chisq: ChiSqConfig {
                samples,
                bins: 256,
                z_limit: Z_LIMIT,
            },
            serial: SerialConfig {
                tuples: samples,
                dim: 2,
                bins_per_axis: 16,
                z_limit: Z_LIMIT,
            },
            runs: RunsConfig {
                samples,
                z_limit: Z_LIMIT,
            },
            birthday: BirthdayConfig {
                points: birthday_points,
                dim: 2,
                bits_per_axis: birthday_bits,
                z_limit: Z_LIMIT,
            },
            paranoid: ParanoidConfig {
                trials,
                trial_alpha: ALPHA,
                uniformity_alpha: ALPHA,
            },
            paranoid_trial: ChiSqConfig {
                samples: trial_samples,
                bins: 256,
                z_limit: Z_LIMIT,
            },
        }
    }
}

// Keep the seed fixed so statistical regressions are reproducible.
fn run_tests(
    name: &str,
    mut sample_f: impl FnMut() -> f64,
    mut sample_u: impl FnMut() -> u64,
    bits: u32,
    config: SuiteConfig,
) {
    macro_rules! assert_pass {
        ($test:expr) => {{
            let result = $test.unwrap_or_else(|error| panic!("{name}: {error}"));
            assert_eq!(result.verdict, Verdict::Pass, "{name}: {result:#?}");
        }};
    }

    assert_pass!(run_ks(name, &mut sample_f, config.ks));
    assert_pass!(run_mcpi(name, &mut sample_f, config.mcpi));
    assert_pass!(run_nist(name, &mut sample_u, bits, config.nist));
    assert_pass!(run_chisq(name, &mut sample_f, config.chisq));
    assert_pass!(run_serial(name, &mut sample_f, config.serial));
    assert_pass!(run_runs(name, &mut sample_f, config.runs));
    assert_pass!(run_birthday(name, &mut sample_u, config.birthday));

    // Paranoid consumes p-values from repeated tests, not raw RNG samples.
    // Each trial consumes a fresh, non-overlapping portion of the stream.
    let mut trial = |_: usize| {
        let result = run_chisq(name, &mut sample_f, config.paranoid_trial)
            .unwrap_or_else(|error| panic!("{name}: paranoid trial: {error}"));
        // Use the chi-squared survival function rather than a normal approximation.
        igamc(result.df / 2.0, result.chi2 / 2.0)
    };
    assert_pass!(run_paranoid(name, config.paranoid, &mut trial));
}

macro_rules! rng_test {
    ($(#[$attribute:meta])* $test:ident, $rng:ty, $seed:ty, $bits:expr, $difficulty:expr) => {
        $(#[$attribute])*
        #[test]
        fn $test() {
            use trng::Rng;

            let mut float_rng = <$rng>::from(SEED as $seed);
            let mut word_rng = <$rng>::from(SEED as $seed);
            run_tests(
                stringify!($rng),
                move || float_rng.nextf() as f64,
                move || word_rng.nextu() as u64,
                $bits,
                $difficulty.config(),
            );
        }
    };
}

macro_rules! suite {
    ($(#[$attribute:meta])* $module:ident, $difficulty:expr) => {
        mod $module {
            use super::*;

            #[cfg(feature = "biski")]
            rng_test!($(#[$attribute])* biski64_test, trng_a::Biski64, u64, 64, $difficulty);

            #[cfg(feature = "sfmt")]
            mod sfmt {
                use super::*;

                rng_test!($(#[$attribute])* sfmt607_test, trng_a::Sfmt607, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt1279_test, trng_a::Sfmt1279, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt2281_test, trng_a::Sfmt2281, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt4253_test, trng_a::Sfmt4253, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt11213_test, trng_a::Sfmt11213, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt19937_test, trng_a::Sfmt19937, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt44497_test, trng_a::Sfmt44497, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt86243_test, trng_a::Sfmt86243, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt132049_test, trng_a::Sfmt132049, u32, 32, $difficulty);
                rng_test!($(#[$attribute])* sfmt216091_test, trng_a::Sfmt216091, u32, 32, $difficulty);
            }
        }
    };
}

// Select a level with: cargo test --all-features --test cribler_test normal::
// Omitting the filter runs easy, normal and hard; hell requires --ignored.
suite!(easy, Difficulty::Easy);
suite!(normal, Difficulty::Normal);
suite!(hard, Difficulty::Hard);

// Run one RNG at a time to avoid multiplying KS memory usage:
// cargo test --release --all-features --test cribler_test hell:: -- --ignored --test-threads=1
// KS alone needs roughly 8 bytes per sample plus sorting workspace.
// e.g. CRIBLER_HELL_SAMPLES=1000000000 raises its sample storage to 8 GB.
suite!(
    #[ignore = "resource-intensive; run explicitly with --ignored --test-threads=1"]
    hell,
    Difficulty::Hell
);
