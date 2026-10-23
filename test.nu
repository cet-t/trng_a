#!/usr/bin/env nu

# Run normal tests, SmallCrush and PractRand (1 GiB).
# Examples:
#   nu test.nu
#   nu test.nu -q
#   nu test.nu --full
#   nu test.nu --filter biski64_test
# TESTU01_RUNNER and PRACTRAND_EXE override automatic engine discovery.

def find-engine [variable: string, candidates: list<string>, command: string] {
    let configured = ($env | get -o $variable | default "")
    if $configured != "" {
        return $configured
    }
    for candidate in $candidates {
        let pattern = ($candidate | str replace --all '\' '/')
        let matches = (glob $pattern | each {|file| ls $file } | flatten)
        if not ($matches | is-empty) {
            return ($matches | sort-by modified --reverse | first | get name)
        }
    }
    let found = (which $command | get -o 0.path)
    if $found != null {
        return $found
    }
    error make {msg: $"($command) not found. Set ($variable) to its executable path."}
}

# Collect failures, optionally suppressing both stdout and stderr.
def run-cargo [args: list<string>, quiet: bool] {
    if $quiet {
        return (^cargo ...$args | complete | get exit_code)
    }
    try {
        ^cargo ...$args
        0
    } catch {
        $env.LAST_EXIT_CODE
    }
}

# Run tests sequentially in release mode and report all failures at the end.
def main [
    --quiet(-q) # Show N/M progress instead of logs; always show the results table.
    --full # Include the long-running Crush and BigCrush batteries.
    --filter: string = "" # Filter statistical tests by RNG name, e.g. biski64_test.
] {
    cd $env.FILE_PWD
    let parent = ($env.PWD | path dirname)
    let suffix = (if $nu.os-info.name == "windows" { ".exe" } else { "" })

    $env.TESTU01_RUNNER = (find-engine TESTU01_RUNNER [
        ($parent | path join cribler_testu01 target release build "*" out $"cribler-testu01-runner($suffix)")
        ($parent | path join cribler_testu01 target debug build "*" out $"cribler-testu01-runner($suffix)")
    ] $"cribler-testu01-runner($suffix)")
    $env.PRACTRAND_EXE = (find-engine PRACTRAND_EXE [
        ($parent | path join cribler_practrand .local $"RNG_test($suffix)")
    ] $"RNG_test($suffix)")

    if not $quiet {
        print $"TestU01: ($env.TESTU01_RUNNER)"
        print $"PractRand: ($env.PRACTRAND_EXE)"
    }
    let suites = (if $full {
        [small_crush practrand crush big_crush]
    } else {
        [small_crush practrand]
    })
    let listed = (^cargo test --release --all-features --test cribler_test -- --list | complete)
    if $listed.exit_code != 0 {
        print --stderr $listed.stderr
        exit $listed.exit_code
    }
    let tests = ($listed.stdout | lines | where {|line| $line | str ends-with ': test' }
        | str replace --regex ': test$' '')
    let selected = ($tests | where {|name|
        ($suites | any {|suite| $name | str starts-with $"($suite)::" }) and ($name | str contains $filter)
    })
    if ($selected | is-empty) {
        error make {msg: $"No statistical tests match filter: ($filter)"}
    }
    # Count the normal test suite as one job, followed by each statistical test.
    let total = (($selected | length) + 1)
    if $quiet { print --no-newline $"Progress: 0/($total)" }
    let normal_code = (run-cargo [test --release --all-features -- --test-threads=1] $quiet)
    mut results = [{test: "normal tests", exit_code: $normal_code}]
    if $quiet { print --no-newline $"\rProgress: 1/($total)" }

    for test in $selected {
        if not $quiet { print $"Running ($test)" }
        let code = (run-cargo [test --release --all-features --test cribler_test $test -- --exact --ignored --nocapture --test-threads=1] $quiet)
        $results = ($results | append {test: $test, exit_code: $code})
        if $quiet { print --no-newline $"\rProgress: ($results | length)/($total)" }
    }
    if $quiet { print "" }

    print "\nTest results:"
    print ($results | each {|result|
        {test: $result.test, status: (if $result.exit_code == 0 { "PASS" } else { "FAIL" }), exit_code: $result.exit_code}
    } | table --expand)
    let failures = ($results | where exit_code != 0)
    print $"Passed: (($results | length) - ($failures | length)), Failed: ($failures | length)"
    if not ($failures | is-empty) { exit 1 }
}
