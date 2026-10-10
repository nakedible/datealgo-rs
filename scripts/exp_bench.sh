#!/usr/bin/env bash
# Run the experiment suite (benches/exp.rs) pinned to one CPU core and print
# each row's mean and its 95% confidence interval against the baseline row.
#
# usage: scripts/exp_bench.sh [GROUP]   e.g. scripts/exp_bench.sh rd_to_date_full
set -euo pipefail
cpu=${EXP_BENCH_CPU:-2}
ZENBENCH_NO_BUSY_GATE=1 taskset -c "$cpu" cargo bench -q --features experiments --bench exp -- \
  ${1:+--group=$1} --format=csv 2>/dev/null |
  awk -F, 'NR > 1 && $1 != "group" {
    ci = ($18 == "") ? "baseline" : sprintf("[%+6.1f%% %+6.1f%%]", $18, $19)
    printf "%-28s %-14s %7.3f ns  %s\n", $1, $2, $4 / 64, ci
  }'
