#!/usr/bin/env bash
# usage: exp_asm.sh [--asm] SYMBOL...   e.g. datealgo::asm::rd_to_date datealgo::experiments::asm::rd_to_date_j64
# Prints llvm-mca estimates for Zen 2 (local), Zen 3 (CI x86_64) and Neoverse-N2 (CI arm64), and with --asm the asm itself.
set -euo pipefail
show_asm=0
if [ "${1:-}" = "--asm" ]; then show_asm=1; shift; fi
F=(--features asmdump,experiments --lib -q)
for sym in "$@"; do
  for spec in "x86_64:znver2:" "x86_64:znver3:" "aarch64:neoverse-n2:--target=aarch64-unknown-linux-gnu"; do
    IFS=: read -r arch cpu target <<<"$spec"
    out=$(cargo asm "${F[@]}" $target --mca -M=-mcpu=$cpu -M=-iterations=100 "$sym" 2>&1)
    cyc=$(sed -n 's/^Total Cycles: *//p' <<<"$out"); uops=$(sed -n 's/^Total uOps: *//p' <<<"$out")
    rt=$(sed -n 's/^Block RThroughput: *//p' <<<"$out"); ins=$(sed -n 's/^Instructions: *//p' <<<"$out")
    printf "%-55s %-8s %-12s insns/it %5s  uops/it %5s  cycles/it %6s  rthroughput %s\n" "$sym" "$arch" "$cpu" "$((ins/100))" "$((uops/100))" "$(awk "BEGIN{printf \"%.1f\", $cyc/100}")" "$rt"
  done
  if [ $show_asm = 1 ]; then
    echo "--- x86_64"; cargo asm "${F[@]}" --simplify "$sym" 2>&1
    echo "--- aarch64"; cargo asm "${F[@]}" --simplify --target=aarch64-unknown-linux-gnu "$sym" 2>&1
  fi
done
