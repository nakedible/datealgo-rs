#!/usr/bin/env bash
# Compare the datealgo of the working tree with that of GIT_REF in the
# application benchmark (benches/app_ab.rs), as described in
# scripts/ab_lib.sh, and report how much the time spent in calendar code per
# event changed in each domain of the application and overall, with
# confidence intervals.
#
# usage: scripts/app_ab.sh [GIT_REF]   GIT_REF defaults to origin/master
#
# APP_AB_PAIRS sets the number of pairs of runs, 15 by default, and APP_AB_SECS
# how many seconds each run measures each benchmark for, 0.5 by default. Each
# run measures each domain, and each domain without calendar work, to tell how
# much of its time is in calendar code.
set -euo pipefail
cd "$(dirname "$0")/.."
. scripts/ab_lib.sh
ref=${1:-origin/master}
pairs=${APP_AB_PAIRS:-15}
secs=${APP_AB_SECS:-0.5}

ab_setup "$ref" app-ab
ab_link app_ab head 0
benches=$("$ab_work/bin/app_ab-head-0" list | tr '\n' ' ')
ab_pairs app_ab app "$benches" "$pairs" "$secs" 1

chunk=$(sed -n 's/^pub const CHUNK: usize = \([0-9]*\);/\1/p' benches/app/mod.rs)
python3 scripts/app_ab_report.py "$ab_work/dump/app" "$chunk" "$ref" "$pairs" "$secs" "$ab_cpus"
