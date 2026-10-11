# Functions for comparing the datealgo of the working tree with that of a git
# ref in a tango benchmark, sourced by scripts/app_ab.sh and
# scripts/basic_ab.sh.
#
# The baseline is the working tree with the library sources (src/) of the ref,
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
# the roles of candidate and baseline on every other pair. Each run starts new
# processes, whose speed also depends on where their memory happens to be, so
# the comparison is over the pairs, and samples within a run are not
# independent observations.
#
# AB_CPUS sets the two CPUs to run the builds on, by default the first two that
# are not on the same physical core.

# Prepare to compare with the given git ref, in target/NAME
ab_setup() {
  local ref=$1 name=$2
  ab_work=$PWD/target/$name
  export CARGO_TARGET_DIR=$ab_work/target
  # Functions and loops are aligned to 64 bytes, so that their placement
  # affects the results less. Shuffling the functions needs LLD, which the
  # x86-64 Linux target links with already.
  export RUSTFLAGS="${RUSTFLAGS:-} -C llvm-args=-align-loops=64 -C llvm-args=-align-all-functions=6"
  if [ "$(uname -m)" != x86_64 ]; then
    command -v ld.lld >/dev/null || { echo "error: needs LLD (ld.lld) to link" >&2; exit 1; }
    RUSTFLAGS="$RUSTFLAGS -C link-arg=-fuse-ld=lld"
  fi
  rm -rf "$ab_work/build" "$ab_work/src" "$ab_work/bin" "$ab_work/dump" "$ab_work/tango.log"
  mkdir -p "$ab_work/build" "$ab_work/src/head" "$ab_work/src/base" "$ab_work/bin" "$ab_work/dump"
  tar --exclude=./target --exclude=./.git -cf - . | tar -xf - -C "$ab_work/build"
  tar -cf - src | tar -xf - -C "$ab_work/src/head"
  git archive "$ref" src | tar -xf - -C "$ab_work/src/base"
  ab_side=
  ab_cpus=${AB_CPUS:-$(lscpu -p=cpu,core | awk -F, '!/^#/ && !seen[$2]++ { c = c (c ? "," : "") $1 } END { split(c, a, ","); print a[1] "," a[2] }')}
}

# Link the given benchmark of the given side, head or base, with each of the
# given seeds for the order of functions, as bin/BENCH-SIDE-SEED
ab_link() {
  local bench=$1 side=$2 seed exe
  shift 2
  if [ "$ab_side" != "$side" ]; then
    rm -rf "$ab_work/build/src"
    cp -r "$ab_work/src/$side/src" "$ab_work/build/src"
    # Cargo decides whether to rebuild by the times of the files, which come
    # from the working tree or from git, so they are set to now to always
    # rebuild the library sources just copied.
    find "$ab_work/build/src" -exec touch {} +
    ab_side=$side
  fi
  for seed in "$@"; do
    exe=$(cd "$ab_work/build" && cargo rustc -q --locked --profile bench --bench "$bench" \
      --message-format=json-render-diagnostics -- -C link-arg=-Wl,--shuffle-sections="*=$seed" |
      python3 -c '
import json, sys
for line in sys.stdin:
    m = json.loads(line)
    if m.get("reason") == "compiler-artifact" and m["target"]["name"] == sys.argv[1] and m.get("executable"):
        print(m["executable"])' "$bench")
    cp "$exe" "$ab_work/bin/$bench-$side-$seed"
  done
}

# Run tango with the given head and base builds, the head on the first given
# CPU and the base on the second, the given one as the candidate, and the
# given arguments. Each worker pins itself to its CPU. Tango's output only
# shows progress, so it goes to a log. It exits with 1 both when it finds a
# regression, which the reports show instead, and when it fails, so a run
# fails if it did not write a dump for each given benchmark.
ab_run() {
  local head=$1 base=$2 head_cpu=$3 base_cpu=$4 candidate=$5 dump=$6 benches=$7 status=0 bench
  shift 7
  if [ "$candidate" = head ]; then
    AB_CPU_CANDIDATE=$head_cpu AB_CPU_BASELINE=$base_cpu taskset -c "$ab_cpus" \
      "$head" --color never compare "$base" -d "$dump" "$@" >>"$ab_work/tango.log" 2>&1 || status=$?
  else
    AB_CPU_CANDIDATE=$base_cpu AB_CPU_BASELINE=$head_cpu taskset -c "$ab_cpus" \
      "$base" --color never compare "$head" -d "$dump" "$@" >>"$ab_work/tango.log" 2>&1 || status=$?
  fi
  for bench in $benches; do
    [ -f "$dump/$bench.csv" ] || status=2
  done
  if [ "$status" -gt 1 ]; then
    cat "$ab_work/tango.log" >&2
    exit 1
  fi
}

# Compare the builds of the given Cargo benchmark target in the given
# benchmarks of it, separated by spaces, in the given number of pairs of runs
# measuring each benchmark for the given seconds, with code layouts from the
# given seed on. The dumps go to dump/NAME/PAIR-CPU-CANDIDATE, where CPU tells
# whether the head was on the first or second CPU, and CANDIDATE which build
# was tango's candidate.
ab_pairs() {
  local bench=$1 name=$2 benches=$3 pairs=$4 secs=$5 first_seed=$6 i filter
  local -a head_seeds=() base_seeds=()
  for i in $(seq 0 $((pairs - 1))); do
    head_seeds+=($((first_seed + 2 * i)))
    base_seeds+=($((first_seed + 2 * i + 1)))
  done
  ab_link "$bench" head "${head_seeds[@]}"
  ab_link "$bench" base "${base_seeds[@]}"
  filter="{$(echo "$benches" | tr ' ' ',')}"
  local cpu_a=${ab_cpus%,*} cpu_b=${ab_cpus#*,} head base candidate
  for i in $(seq 1 "$pairs"); do
    head=$ab_work/bin/$bench-head-${head_seeds[$((i - 1))]}
    base=$ab_work/bin/$bench-base-${base_seeds[$((i - 1))]}
    candidate=$([ $((i % 2)) = 1 ] && echo head || echo base)
    ab_run "$head" "$base" "$cpu_a" "$cpu_b" "$candidate" "$ab_work/dump/$name/$i-a-$candidate" "$benches" -f "$filter" -t "$secs"
    ab_run "$head" "$base" "$cpu_b" "$cpu_a" "$candidate" "$ab_work/dump/$name/$i-b-$candidate" "$benches" -f "$filter" -t "$secs"
  done
}
