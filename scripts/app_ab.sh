#!/usr/bin/env bash
# Compare the datealgo of the working tree with that of GIT_REF in the
# application benchmark (benches/app_ab.rs), as described in
# scripts/ab_lib.sh, and report how much the time spent in calendar code per
# event changed, with a 95% confidence interval.
#
# usage: scripts/app_ab.sh [GIT_REF]   GIT_REF defaults to origin/master
#
# APP_AB_PAIRS sets the number of pairs of runs, 15 by default, and APP_AB_SECS
# how many seconds each run measures each of its two benchmarks for, 2 by
# default. Each run measures the application, and the application without
# calendar work, to tell how much of its time is in calendar code.
set -euo pipefail
cd "$(dirname "$0")/.."
. scripts/ab_lib.sh
ref=${1:-origin/master}
pairs=${APP_AB_PAIRS:-15}
secs=${APP_AB_SECS:-2}

ab_setup "$ref" app-ab
ab_pairs app_ab app "app none" "$pairs" "$secs" 1

chunk=$(sed -n 's/^pub const CHUNK: usize = \([0-9]*\);/\1/p' benches/app_support.rs)
python3 scripts/app_ab_report.py "$ab_work/dump/app" "$chunk" "$ref" "$pairs" "$secs" "$ab_cpus"
