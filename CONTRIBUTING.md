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
- Ensure documentation is comprehensive

## Testing your change

Tests should always be run before submitting changes. The CI will run the same
tests in Github Actions. Tests can be run simply with `cargo test`. It will run
unit tests as well as use `quickcheck` to ensure a random sampling of values
will produce the same results as other known date libraries.

Performance testing is important for all changes. Usually it is a good idea to
benchmark the code against current released version.

The default benchmark harness is Zenbench. Running `cargo bench` will execute
the primary Zenbench suites.

To compare your change with `origin/master` the way the pull request checks
do, run `scripts/basic_ab.sh` for each function on its own, and
`scripts/app_ab.sh` for the application benchmark. Both build the two
versions with several code layouts, run them at the same time on two CPUs
with tango's paired benchmarking, and compare them over pairs of runs.
`scripts/basic_ab.sh` screens all functions and measures again those that look
changed, and reports a change only if that confirms it, of at least `5%`, at a
level that keeps the chance of reporting any change for a pull request that
changes nothing at `5%`.

For ad hoc local exploration, the common Zenbench workflow is:

1. Checkout `master` branch.
2. Run `cargo bench --bench basic -- --save-baseline=master` to run the Zenbench basic suite and save a baseline.
3. Run `cargo bench --bench basic -- --baseline=master` to rerun the Zenbench basic suite against that baseline. This is to verify your benchmarking setup doesn't have too much noise.
4. Checkout the branch with your changes.
5. Run `cargo bench --bench basic -- --baseline=master`. You can rerun this after changes to see how they affect performance.

Criterion benchmarks are still available, but they are opt-in and not run by
default:

- `cargo bench --features criterion-benches --bench basic_criterion`
- `cargo bench --features criterion-benches --bench compare_criterion`

In addition to this it might be beneficial to use
[Gungraun](https://gungraun.github.io/gungraun/) to count how many instructions
each method executes. Instruction counts are usually a good rough guide on
performance, but in the case of this library they are nearly useless.

Gungraun needs Valgrind and a `gungraun-runner` binary of the same version as
the `gungraun` dev-dependency in `Cargo.lock`, for example
`cargo binstall gungraun-runner@0.20.0`. Reinstall the runner whenever the
`gungraun` dependency is updated.

- `cargo bench --features gungraun-bench --bench gungraun` runs the benchmarks
  and compares against the previous run.
- `python3 scripts/compare_gungraun_against_ref.py --git-ref origin/master`
  compares the current worktree against `origin/master`. Pull requests get the
  same comparison as a comment.
- `cargo bench --features gungraun-bench --bench gungraun -- --save-baseline=master`
  and `-- --baseline=master` save and compare against a named baseline, like
  with Zenbench. Instruction counts don't vary between runs with the same
  compiler, so this comparison stays reliable even when the runs are far apart.

## Trying out a faster algorithm

The experiment tooling makes it quick to compare candidate implementations of a
function against the current one, from static analysis to measurements on both
x86-64 and AArch64:

1. Add each candidate to `src/experiments.rs` (enabled by the `experiments`
   feature) as an `#[inline] pub const fn` next to the current function, with a
   non-inlined wrapper in its `asm` module. Candidates then compile the same way
   as the real function.
2. Look at the assembly and llvm-mca's estimates for Zen 2, Zen 3 and
   Neoverse-N2 (GitHub's x86-64 and AArch64 runners):
   `scripts/asm_mca.sh [--asm] datealgo::asm::rd_to_date datealgo::experiments::asm::<candidate>`.
   It needs cargo-show-asm, llvm-mca and the AArch64 target
   (`rustup target add aarch64-unknown-linux-gnu`). The tooling builds the
   AArch64 code without running it, so no ARM machine is needed.
3. Check the candidate against the current function over the whole supported
   range with `check` in `tests/exhaustive.rs`, and run
   `cargo test --release --features experiments --test exhaustive -- --ignored`.
   These tests take seconds, and CI runs the ones for the current functions.
4. Add a row for the candidate in `benches/exp.rs` and run
   `scripts/exp_bench.sh`. It compares all rows interleaved in the same rounds,
   for several input distributions, both with the result returned as a whole
   and with its fields consumed separately, and prints each row's 95%
   confidence interval against the current function. A run takes seconds, and
   `gh workflow run full-bench.yml --ref <branch> -f runner=all -f harness=experiments`
   runs it on GitHub's x86-64 and AArch64 runners. It measures the function on
   its own in a hot loop, which shows where its time goes but favors tables and
   code that needs warm caches. Where each row's code lands in memory can still
   shift results by a few percent, so check the assembly before trusting a
   small difference.
5. Decide with the application benchmark, which measures the change the way it
   would run in software using the library: replace the function in
   `src/lib.rs` and run `scripts/app_ab.sh`, which compares the working tree
   with `origin/master`, or with a given git ref. It prints how much the time
   spent in calendar code per event changed, with a 95% confidence interval,
   and pull requests run it on GitHub's x86-64 and AArch64 runners. A change is
   better if it is better on both architectures.
6. Open a pull request with the winner and without the experiment code. The
   pull request checks run the application benchmark, verify the basic
   benchmark, compare assembly and count instructions on both architectures.

## Commit messages

Commit messages must conform to Conventional Commits. Changelog is automatically
generated based on them.
