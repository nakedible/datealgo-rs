#!/usr/bin/env bash
# Run the application benchmark (benches/app.rs) pinned to one CPU core and
# print each row's time per event and its 95% confidence interval against the
# baseline row.
#
# usage: scripts/app_bench.sh
#
# Functions and loops are aligned to 64 bytes, so that where the code of each
# row happens to be placed affects the results less. This needs its own target
# directory, as the flags rebuild everything.
set -euo pipefail
cpu=${APP_BENCH_CPU:-2}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-target/exp}
export RUSTFLAGS="${RUSTFLAGS:-} -C llvm-args=-align-loops=64 -C llvm-args=-align-all-functions=6"
ZENBENCH_NO_BUSY_GATE=1 taskset -c "$cpu" cargo bench -q --features experiments --bench app -- --format=csv |
  awk -F, 'NR > 1 && $1 != "group" {
    p = substr($16, 1, 1)
    scale = p == "G" ? 1 : p == "M" ? 1e3 : p == "K" ? 1e6 : 1e9
    ci = ($18 == "") ? "baseline" : sprintf("[%+6.1f%% %+6.1f%%]", $18, $19)
    printf "%-12s %8.2f ns  %s\n", $2, scale / $15, ci
  }'
