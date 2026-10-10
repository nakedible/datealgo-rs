#!/usr/bin/env bash
# usage: exp_bench.sh [GROUP_FILTER]  -- runs the experiment suite pinned to one core, prints mean and CI vs datealgo
ZENBENCH_NO_BUSY_GATE=1 taskset -c 2 cargo bench -q --features experiments --bench exp -- ${1:+--group=$1} --format=csv 2>/dev/null \
  | awk -F, 'NR>1 && $1!="group" {printf "%-20s %-10s %7.2f ns  vs datealgo [%+6.1f%% %+6.1f%%]\n", $1, $2, $4, $18, $19}'
