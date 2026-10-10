#!/usr/bin/env bash
# Show static performance estimates, and optionally the assembly, of functions
# on x86-64 and AArch64.
#
# usage: scripts/asm_mca.sh [--asm] SYMBOL...
#   e.g. scripts/asm_mca.sh --asm datealgo::asm::rd_to_date datealgo::experiments::asm::rd_to_date_x
#
# llvm-mca estimates are given for Zen 2, Zen 3 (GitHub's x86-64 runners) and
# Neoverse-N2 (GitHub's AArch64 runners). Requires cargo-show-asm, llvm-mca and
# the aarch64-unknown-linux-gnu target (rustup target add aarch64-unknown-linux-gnu).
set -euo pipefail

show_asm=0
if [ "${1:-}" = "--asm" ]; then
  show_asm=1
  shift
fi

features=(--features asmdump,experiments --lib -q)

# cargo-show-asm treats the name as a substring and lists the matches when
# several functions match, so pick the index of the exact name in that list.
index_of() {
  local sym=$1
  shift
  { cargo asm "${features[@]}" "$@" "$sym" 2>&1 || true; } |
    sed -n "s/^ *\([0-9]*\) \"$sym\" .*/\1/p" | head -n 1
}

for sym in "$@"; do
  for spec in "x86_64 znver2" "x86_64 znver3" "aarch64 neoverse-n2"; do
    read -r arch cpu <<<"$spec"
    target=()
    if [ "$arch" = aarch64 ]; then target=(--target=aarch64-unknown-linux-gnu); fi
    idx=$(index_of "$sym" "${target[@]}")
    out=$(cargo asm "${features[@]}" "${target[@]}" --mca -M=-mcpu="$cpu" -M=-iterations=100 "$sym" ${idx:+$idx} 2>&1)
    insns=$(sed -n 's/^Instructions: *//p' <<<"$out")
    uops=$(sed -n 's/^Total uOps: *//p' <<<"$out")
    rthroughput=$(sed -n 's/^Block RThroughput: *//p' <<<"$out")
    printf "%-52s %-8s %-12s insns %4s  uops %4s  rthroughput %s\n" \
      "$sym" "$arch" "$cpu" "$((insns / 100))" "$((uops / 100))" "$rthroughput"
  done
  if [ "$show_asm" = 1 ]; then
    for arch in x86_64 aarch64; do
      target=()
      if [ "$arch" = aarch64 ]; then target=(--target=aarch64-unknown-linux-gnu); fi
      idx=$(index_of "$sym" "${target[@]}")
      echo "--- $arch"
      cargo asm "${features[@]}" "${target[@]}" --simplify "$sym" ${idx:+$idx} 2>&1
    done
  fi
done
