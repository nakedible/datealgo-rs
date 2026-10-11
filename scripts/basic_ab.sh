#!/usr/bin/env bash
# Compare the datealgo of the working tree with that of GIT_REF in the basic
# benchmarks of each function (benches/basic_ab.rs), as described in
# scripts/ab_lib.sh, and report the functions that changed.
#
# usage: scripts/basic_ab.sh [GIT_REF]   GIT_REF defaults to origin/master
#
# A change is only reported once confirmed. All functions are first screened
# in BASIC_AB_PAIRS pairs of runs, 10 by default, of BASIC_AB_SECS seconds per
# function, 0.3 by default. The functions that look changed are then measured
# again in BASIC_AB_CONFIRM_PAIRS new pairs of runs, 20 by default, of
# BASIC_AB_CONFIRM_SECS seconds, 0.5 by default, with new code layouts, so that
# a layout that made a function look changed in the screening does not carry
# over. A change is reported if the confirmation finds it significant at a
# level that keeps the chance of reporting any change for a pull request that
# changes nothing at 5%, and of at least BASIC_AB_THRESHOLD percent, 5 by
# default.
set -euo pipefail
cd "$(dirname "$0")/.."
. scripts/ab_lib.sh
ref=${1:-origin/master}
pairs=${BASIC_AB_PAIRS:-10}
secs=${BASIC_AB_SECS:-0.3}
confirm_pairs=${BASIC_AB_CONFIRM_PAIRS:-20}
confirm_secs=${BASIC_AB_CONFIRM_SECS:-0.5}
threshold=${BASIC_AB_THRESHOLD:-5}

ab_setup "$ref" basic-ab
ab_link basic_ab head 0
benches=$("$ab_work/bin/basic_ab-head-0" list | tr '\n' ' ')
batch=$(sed -n 's/^const BATCH: usize = \([0-9]*\);/\1/p' benches/basic_ab.rs)

ab_pairs basic_ab screen "$benches" "$pairs" "$secs" 1
suspects=$(python3 scripts/basic_ab_report.py screen "$ab_work/dump" "$batch" "$threshold" $benches | tr '\n' ' ')
if [ -n "${suspects// /}" ]; then
  ab_pairs basic_ab confirm "$suspects" "$confirm_pairs" "$confirm_secs" 100001
fi
python3 scripts/basic_ab_report.py report "$ab_work/dump" "$batch" "$threshold" $benches <<EOF
Baseline $ref, candidate the working tree, on CPUs $ab_cpus.
Each function was screened in $pairs pairs of runs of $secs s, and those that
looked changed were confirmed in $confirm_pairs new pairs of runs of $confirm_secs s,
each pair with other code layouts of both builds.
EOF
