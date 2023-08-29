# Contributing to datealgo-rs

## Legal Notice 

When contributing to this project, you must agree that you have authored 100% of
the content, that you have the necessary rights to the content and that the
content you contribute may be provided under the project license.

## Principles

- No custom datatypes, those belong to other libraries using this library
- Debug builds must panic on invalid input values using `debug_assert!`
- Release builds must never panic, unless panic comes from inside a library (like hypothetically inside `SystemTime`)
- Invalid inputs are allowed to cause incorrect results in release builds
- Checking for invalid inputs in release builds should be done only to improve compiler performance or to avoid panics
- Use the smallest datatypes that fit the values, unless performance loss is unpalatable
- Use `#[inline]` on all methods, as the idea is that the methods are wrapped by some other library
- Ensure documentation is commprehensive

## Testing your change

Tests should always be run before submitting changes. The CI will run the same
tests in Github Actions. Tests can be run simply with `cargo test`. It will run
unit tests as well as use `quickcheck` to ensure a random sampling of values
will produce the same results as other known date libraries.

Performance testing is important for all changes. Usually it is a good idea to
benchmark the code against current released version.

The common way to test performance changes:

1. Checkout `master` branch.
2. Run `cargo bench --bench basic -- --save-baseline master` to run benchmarks and save results. This is your baseline to compare against.
3. Run `cargo bench --bench basic -- --baseline master` to run benchmarks again and compare. This is to verify your benchmarking setup doesn't have too much noise. All tests should have "No change in performance detected."
4. Checkout the branch with your changes.
5. Run `cargo bench --bench basic -- --baseline master`. It should report the change in performance. You can rerun this command after you make some changes to see how it affects performance.

In addition to this it might be beneficial use `iai-callgrind` to benchmark how
many instructions each method will use. This can be done with `cargo bench
--bench iai`. It will always compare against the last invocation and has no
command-line options. Instruction counts are usually a good rough guide on
performance, but in the case of this library they are nearly useless.

## Commit messages

Commit messages must conform to Conventional Commits. Changelog is automatically
generated based on them.