#!/usr/bin/env bash
# Run the application benchmark (benches/app.rs) pinned to one CPU core and
# print each row's time per event, the part of it spent in calendar code, which
# is the difference to the row that does no calendar work, and how that part
# compares to the baseline row's.
#
# usage: scripts/app_bench.sh
#
# APP_BENCH_SECS sets how many seconds to measure for, 30 by default, and
# APP_BENCH_CPU the CPU core to run on.
#
# Functions and loops are aligned to 64 bytes, so that where the code of each
# row happens to be placed affects the results less. This needs its own target
# directory, as the flags rebuild everything.
set -euo pipefail
cpu=${APP_BENCH_CPU:-2}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-target/exp}
export RUSTFLAGS="${RUSTFLAGS:-} -C llvm-args=-align-loops=64 -C llvm-args=-align-all-functions=6"
ZENBENCH_NO_BUSY_GATE=1 taskset -c "$cpu" cargo bench -q --locked --features experiments --bench app -- --format=csv 2>/dev/null |
  awk -F, '
    # Rows are compared with the baseline in pairs of runs, so all numbers are
    # derived from the paired differences rather than from the means of rows.
    NR > 1 && $1 != "group" {
      n++; name[n] = $2; pct[n] = $17 + 0; lo[n] = $18; hi[n] = $19
      if (n == 1) {
        p = substr($16, 1, 1)
        base = (p == "G" ? 1 : p == "M" ? 1e3 : p == "K" ? 1e6 : 1e9) / $15
      }
      if ($2 == "none") none = n
    }
    END {
      cal = -pct[none] / 100 * base
      printf "%-12s %9s %10s  %s\n", "", "per event", "calendar", "calendar vs " name[1] " (95% CI)"
      for (i = 1; i <= n; i++) {
        ci = i == 1 ? "baseline" : i == none ? "" : sprintf("[%+7.1f%% %+7.1f%%]", lo[i] * base / cal, hi[i] * base / cal)
        printf "%-12s %8.2f ns %7.2f ns  %s\n", name[i], base * (1 + pct[i] / 100), cal + pct[i] / 100 * base, ci
      }
    }'
