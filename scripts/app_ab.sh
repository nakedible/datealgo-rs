#!/usr/bin/env bash
# Compare the datealgo of the working tree with that of GIT_REF in the
# application benchmark (benches/app_ab.rs), with tango's paired benchmarking.
#
# usage: scripts/app_ab.sh [GIT_REF]   GIT_REF defaults to origin/master
#
# APP_AB_PAIRS sets the number of pairs of runs, 15 by default, and APP_AB_SECS
# how many seconds each run measures each of its two benchmarks for, 2 by
# default. APP_AB_CPUS sets the two CPUs to run the builds on, by default the
# first two that are not on the same physical core.
#
# The baseline is the working tree with the library sources (src/) of GIT_REF,
# built in the same directory, so that the two builds differ only by the
# library. Where the linker happens to place each function changes the speed
# of the whole program by about a percent, more than many changes to the
# library, so each build is linked once for each pair of runs, with the
# functions in a different random order each time.
#
# In each run, tango runs the two builds at the same time, one on each CPU,
# and pairs their samples, so that changes in the machine's speed affect both
# alike. The two CPUs may differ in speed, so the builds swap CPUs between the
# two runs of a pair, and each pair is one observation. The builds also swap
# the roles of candidate and baseline on every other pair.
set -euo pipefail
cd "$(dirname "$0")/.."
ref=${1:-origin/master}
pairs=${APP_AB_PAIRS:-15}
secs=${APP_AB_SECS:-2}
work=$PWD/target/app-ab
export CARGO_TARGET_DIR=$work/target
# Functions and loops are aligned to 64 bytes, so that their placement affects
# the results less. Shuffling the functions needs LLD, which the x86-64 Linux
# target links with already.
export RUSTFLAGS="${RUSTFLAGS:-} -C llvm-args=-align-loops=64 -C llvm-args=-align-all-functions=6"
if [ "$(uname -m)" != x86_64 ]; then
  command -v ld.lld >/dev/null || { echo "error: needs LLD (ld.lld) to link" >&2; exit 1; }
  RUSTFLAGS="$RUSTFLAGS -C link-arg=-fuse-ld=lld"
fi

rm -rf "$work/build" "$work/bin" "$work/dump" "$work/tango.log"
mkdir -p "$work/build" "$work/bin" "$work/dump"
tar --exclude=./target --exclude=./.git -cf - . | tar -xf - -C "$work/build"

# Link the benchmark in the build directory with each of the given seeds for
# the order of functions, as bin/NAME-SEED. Cargo decides whether to rebuild by
# the times of the files, which come from the working tree or from git, so
# they are set to now to always rebuild the library sources just copied.
build() {
  local name=$1 seed exe
  shift
  find "$work/build/src" -exec touch {} +
  for seed in "$@"; do
    exe=$(cd "$work/build" && cargo rustc -q --locked --profile bench --bench app_ab \
      --message-format=json-render-diagnostics -- -C link-arg=-Wl,--shuffle-sections="*=$seed" |
      python3 -c '
import json, sys
for line in sys.stdin:
    m = json.loads(line)
    if m.get("reason") == "compiler-artifact" and m["target"]["name"] == "app_ab" and m.get("executable"):
        print(m["executable"])')
    cp "$exe" "$work/bin/$name-$seed"
  done
}
build head $(seq 1 "$pairs")
rm -rf "$work/build/src"
git archive "$ref" src | tar -xf - -C "$work/build"
build base $(seq $((pairs + 1)) $((2 * pairs)))

cpus=${APP_AB_CPUS:-$(lscpu -p=cpu,core | awk -F, '!/^#/ && !seen[$2]++ { c = c (c ? "," : "") $1 } END { split(c, a, ","); print a[1] "," a[2] }')}
cpu_a=${cpus%,*}
cpu_b=${cpus#*,}

# Run tango with the given head and base builds, the head on the first given
# CPU and the base on the second, and the given one as the candidate. Each
# worker pins itself to its CPU. Each run measures the application, and the
# application without calendar work, to tell how much of its time is in
# calendar code. Tango's output only shows progress, so it goes to a log. It
# exits with 1 both when it finds a regression, which is reported below
# instead, and when it fails, so a run fails if it did not write both dumps.
run() {
  local head=$1 base=$2 head_cpu=$3 base_cpu=$4 candidate=$5 dump=$6 status=0
  if [ "$candidate" = head ]; then
    APP_AB_CPU_CANDIDATE=$head_cpu APP_AB_CPU_BASELINE=$base_cpu taskset -c "$cpus" \
      "$head" --color never compare "$base" -d "$dump" -f '{app,none}' -t "$secs" \
      >>"$work/tango.log" 2>&1 || status=$?
  else
    APP_AB_CPU_CANDIDATE=$base_cpu APP_AB_CPU_BASELINE=$head_cpu taskset -c "$cpus" \
      "$base" --color never compare "$head" -d "$dump" -f '{app,none}' -t "$secs" \
      >>"$work/tango.log" 2>&1 || status=$?
  fi
  if [ "$status" -gt 1 ] || [ ! -f "$dump/app.csv" ] || [ ! -f "$dump/none.csv" ]; then
    cat "$work/tango.log" >&2
    exit 1
  fi
}

for i in $(seq 1 "$pairs"); do
  head=$work/bin/head-$i
  base=$work/bin/base-$((pairs + i))
  candidate=$([ $((i % 2)) = 1 ] && echo head || echo base)
  run "$head" "$base" "$cpu_a" "$cpu_b" "$candidate" "$work/dump/$i-a-$candidate"
  run "$head" "$base" "$cpu_b" "$cpu_a" "$candidate" "$work/dump/$i-b-$candidate"
done

chunk=$(sed -n 's/^pub const CHUNK: usize = \([0-9]*\);/\1/p' benches/app_support.rs)
python3 scripts/app_ab_report.py "$work/dump" "$chunk" "$ref" "$pairs" "$secs" "$cpus"
