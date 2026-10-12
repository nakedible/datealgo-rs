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
# AB_CPUS sets the two CPUs to run the builds on, by default the last two that
# are not on the same physical core, as the first CPU often serves more of the
# system. With AB_ISOLATE set, every other process and every interrupt is moved
# off the two CPUs with sudo before the runs, and left so, which is meant for
# machines that only run the comparison, such as those of CI.

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
  rm -rf "$ab_work/build" "$ab_work/src" "$ab_work/bin" "$ab_work/dump" "$ab_work/tango.log" "$ab_work"/link-*
  mkdir -p "$ab_work/build" "$ab_work/src/head" "$ab_work/src/base" "$ab_work/bin" "$ab_work/dump"
  tar --exclude=./target --exclude=./.git -cf - . | tar -xf - -C "$ab_work/build"
  tar -cf - src | tar -xf - -C "$ab_work/src/head"
  git archive "$ref" src | tar -xf - -C "$ab_work/src/base"
  ab_side=
  ab_isolated=
  ab_cpus=${AB_CPUS:-$(lscpu -p=cpu,core | awk -F, '!/^#/ && !seen[$2]++ { c = c (c ? "," : "") $1 } END { n = split(c, a, ","); print a[n - 1] "," a[n] }')}
}

# Move every other process and every interrupt off the CPUs of the runs, if
# AB_ISOLATE is set, once
ab_isolate() {
  [ -n "${AB_ISOLATE:-}" ] && [ -z "$ab_isolated" ] || return 0
  local other pid irq
  other=$(lscpu -p=cpu | awk -F, -v cpus=",$ab_cpus," '!/^#/ && !index(cpus, "," $1 ",") { c = c (c ? "," : "") $1 } END { print c }')
  # Some kernel threads and interrupts cannot move, which is fine
  for pid in $(ps -e -o pid=); do
    sudo taskset -a -p -c "$other" "$pid" >/dev/null 2>&1 || true
  done
  for irq in /proc/irq/*/smp_affinity_list; do
    echo "$other" | sudo tee "$irq" >/dev/null 2>&1 || true
  done
  ab_isolated=1
}

# Link the given benchmark of the given side, head or base, with each of the
# given seeds for the order of functions, as bin/BENCH-SIDE-SEED
ab_link() {
  local bench=$1 side=$2 seed link
  shift 2
  if [ "$ab_side" != "$side" ]; then
    rm -rf "$ab_work/build/src" "$ab_work"/link-*
    cp -r "$ab_work/src/$side/src" "$ab_work/build/src"
    # Cargo decides whether to rebuild by the times of the files, which come
    # from the working tree or from git, so they are set to now to always
    # rebuild the sources just copied, and the benchmarks with them.
    find "$ab_work/build/src" "$ab_work/build/benches" -exec touch {} +
    ab_side=$side
  fi
  # Another order of functions changes the arguments of the compiler, which
  # would make Cargo compile the benchmark again, so it is compiled once,
  # keeping the object files, and the command that links them, which the
  # compiler prints, is run again with each seed.
  if [ ! -f "$ab_work/link-$bench" ]; then
    (cd "$ab_work/build" && cargo rustc -q --locked --profile bench --bench "$bench" -- \
      -C save-temps --print link-args -C link-arg=-Wl,--shuffle-sections='*=0') >"$ab_work/link-$bench"
    [ -s "$ab_work/link-$bench" ] || { echo "error: no link command for $bench" >&2; exit 1; }
  fi
  link=$(cat "$ab_work/link-$bench")
  for seed in "$@"; do
    eval "$(printf '%s' "$link" | sed -E "s|\"-o\" \"[^\"]*\"|\"-o\" \"$ab_work/bin/$bench-$side-$seed\"|; s|--shuffle-sections=\\*=0\"|--shuffle-sections=*=$seed\"|")"
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
  ab_isolate
  for i in $(seq 1 "$pairs"); do
    head=$ab_work/bin/$bench-head-${head_seeds[$((i - 1))]}
    base=$ab_work/bin/$bench-base-${base_seeds[$((i - 1))]}
    candidate=$([ $((i % 2)) = 1 ] && echo head || echo base)
    ab_run "$head" "$base" "$cpu_a" "$cpu_b" "$candidate" "$ab_work/dump/$name/$i-a-$candidate" "$benches" -f "$filter" -t "$secs"
    ab_run "$head" "$base" "$cpu_b" "$cpu_a" "$candidate" "$ab_work/dump/$name/$i-b-$candidate" "$benches" -f "$filter" -t "$secs"
  done
}
