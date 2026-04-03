#!/usr/bin/env python3

"""Verify meaningful changes in the Zenbench basic suite against a git ref.

The built-in `zenbench self-compare` flow is useful for exploration, but for
these nanosecond-scale microbenchmarks it is too noisy to use as a release gate.
This helper compares one basic benchmark group at a time, pins execution to a
single CPU core, primes both sides, and only confirms changes that still clear a
5% threshold after interleaved repeated measurement with strong directional
agreement.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
from dataclasses import dataclass


ROOT = pathlib.Path(__file__).resolve().parents[1]
BENCH_SOURCE = ROOT / "benches" / "basic_zen.rs"
DEFAULT_GIT_REF = "origin/master"
DEFAULT_CPU = 0
DEFAULT_PRIME_RUNS = 1
DEFAULT_MEASURE_RUNS = 1
DEFAULT_CONFIRM_RUNS = 3
DEFAULT_THRESHOLD_PCT = 5.0
DEFAULT_CONFIRM_AGREEMENT_NUM = 4
DEFAULT_CONFIRM_AGREEMENT_DEN = 5


@dataclass
class GroupResult:
    name: str
    baseline_runs: list[float]
    candidate_runs: list[float]

    @property
    def baseline_median(self) -> float:
        return statistics.median(self.baseline_runs)

    @property
    def candidate_median(self) -> float:
        return statistics.median(self.candidate_runs)

    @property
    def delta_ns(self) -> float:
        return self.candidate_median - self.baseline_median

    @property
    def delta_pct(self) -> float:
        if self.baseline_median == 0.0:
            return 0.0
        return self.delta_ns / self.baseline_median * 100.0

    @property
    def pair_deltas(self) -> list[float]:
        return [cand - base for base, cand in zip(self.baseline_runs, self.candidate_runs)]


def run(
    cmd: list[str],
    cwd: pathlib.Path,
    env: dict[str, str] | None = None,
    stdout=None,
    stderr=None,
) -> None:
    subprocess.run(cmd, cwd=cwd, env=env, stdout=stdout, stderr=stderr, check=True)


def git_short_hash(cwd: pathlib.Path) -> str:
    out = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], cwd=cwd, text=True)
    return out.strip()


def git_short_hash_for_ref(cwd: pathlib.Path, git_ref: str) -> str:
    out = subprocess.check_output(["git", "rev-parse", "--short", git_ref], cwd=cwd, text=True)
    return out.strip()


def parse_groups() -> list[str]:
    groups = []
    pattern = re.compile(r'suite\.group\("([^"]+)"')
    for line in BENCH_SOURCE.read_text().splitlines():
        match = pattern.search(line)
        if match:
            groups.append(match.group(1))
    return groups


def create_worktree(git_ref: str) -> tuple[pathlib.Path, pathlib.Path]:
    temp_dir = pathlib.Path(tempfile.mkdtemp(prefix="datealgo-verify-basic-bench-", dir="/tmp"))
    worktree = temp_dir / "baseline"
    run(["git", "worktree", "add", "--detach", str(worktree), git_ref], cwd=ROOT)
    return temp_dir, worktree


def remove_worktree(temp_dir: pathlib.Path, worktree: pathlib.Path) -> None:
    subprocess.run(["git", "worktree", "remove", "--force", str(worktree)], cwd=ROOT, check=False)
    shutil.rmtree(temp_dir, ignore_errors=True)


def bench_json_path(tmp_dir: pathlib.Path, group: str, side: str, kind: str, idx: int) -> pathlib.Path:
    safe_group = group.replace("/", "_")
    return tmp_dir / f"{safe_group}.{side}.{kind}.{idx}.json"


def bench_group(cwd: pathlib.Path, cpu: int, group: str, out_path: pathlib.Path) -> float:
    if out_path.exists():
        out_path.unlink()

    env = os.environ.copy()
    env["ZENBENCH_RESULT_PATH"] = str(out_path)
    run(
        ["taskset", "-c", str(cpu), "cargo", "bench", "--bench", "basic", "--", f"--group={group}"],
        cwd=cwd,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    with out_path.open() as fh:
        result = json.load(fh)
    return result["comparisons"][0]["benchmarks"][0]["summary"]["mean"]


def compare_group(
    baseline_cwd: pathlib.Path,
    candidate_cwd: pathlib.Path,
    tmp_dir: pathlib.Path,
    cpu: int,
    group: str,
    prime_runs: int,
    measure_runs: int,
) -> GroupResult:
    for side, cwd in (("base", baseline_cwd), ("cand", candidate_cwd)):
        for i in range(prime_runs):
            bench_group(cwd, cpu, group, bench_json_path(tmp_dir, group, side, "prime", i))

    baseline_runs = [
        bench_group(baseline_cwd, cpu, group, bench_json_path(tmp_dir, group, "base", "run", i))
        for i in range(measure_runs)
    ]
    candidate_runs = [
        bench_group(candidate_cwd, cpu, group, bench_json_path(tmp_dir, group, "cand", "run", i))
        for i in range(measure_runs)
    ]
    return GroupResult(group, baseline_runs, candidate_runs)


def compare_group_paired(
    baseline_cwd: pathlib.Path,
    candidate_cwd: pathlib.Path,
    tmp_dir: pathlib.Path,
    cpu: int,
    group: str,
    prime_runs: int,
    measure_runs: int,
) -> GroupResult:
    for i in range(prime_runs):
        if i % 2 == 0:
            bench_group(baseline_cwd, cpu, group, bench_json_path(tmp_dir, group, "base", "prime", i))
            bench_group(candidate_cwd, cpu, group, bench_json_path(tmp_dir, group, "cand", "prime", i))
        else:
            bench_group(candidate_cwd, cpu, group, bench_json_path(tmp_dir, group, "cand", "prime", i))
            bench_group(baseline_cwd, cpu, group, bench_json_path(tmp_dir, group, "base", "prime", i))

    baseline_runs = []
    candidate_runs = []
    for i in range(measure_runs):
        if i % 2 == 0:
            baseline = bench_group(
                baseline_cwd,
                cpu,
                group,
                bench_json_path(tmp_dir, group, "base", "confirm", i),
            )
            candidate = bench_group(
                candidate_cwd,
                cpu,
                group,
                bench_json_path(tmp_dir, group, "cand", "confirm", i),
            )
        else:
            candidate = bench_group(
                candidate_cwd,
                cpu,
                group,
                bench_json_path(tmp_dir, group, "cand", "confirm", i),
            )
            baseline = bench_group(
                baseline_cwd,
                cpu,
                group,
                bench_json_path(tmp_dir, group, "base", "confirm", i),
            )
        baseline_runs.append(baseline)
        candidate_runs.append(candidate)

    return GroupResult(group, baseline_runs, candidate_runs)


def exceeds_threshold(result: GroupResult, pct_threshold: float) -> bool:
    return abs(result.delta_pct) >= pct_threshold


def confirm_agreement_required(confirm_runs: int) -> int:
    return max(
        1,
        (confirm_runs * DEFAULT_CONFIRM_AGREEMENT_NUM + (DEFAULT_CONFIRM_AGREEMENT_DEN - 1))
        // DEFAULT_CONFIRM_AGREEMENT_DEN,
    )


def has_directional_agreement(result: GroupResult, required: int) -> bool:
    if result.delta_ns == 0.0:
        return False

    expected_positive = result.delta_ns > 0.0
    agreeing_pairs = sum(
        (delta > 0.0) == expected_positive
        for delta in result.pair_deltas
        if delta != 0.0
    )
    return agreeing_pairs >= required


def print_result(result: GroupResult, status: str) -> None:
    print(
        f"{result.name:24s} {status:10s} "
        f"{result.baseline_median:8.3f}ns -> {result.candidate_median:8.3f}ns "
        f"({result.delta_pct:+6.2f}%, {result.delta_ns:+.3f}ns)"
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Verify meaningful changes in the Zenbench basic suite against a git ref."
    )
    parser.add_argument(
        "--git-ref",
        default=DEFAULT_GIT_REF,
        help=f"Baseline git ref to compare against. Default: {DEFAULT_GIT_REF}.",
    )
    parser.add_argument(
        "--cpu",
        type=int,
        default=DEFAULT_CPU,
        help=f"CPU core to pin benchmark runs to. Default: {DEFAULT_CPU}.",
    )
    parser.add_argument(
        "--prime-runs",
        type=int,
        default=DEFAULT_PRIME_RUNS,
        help=f"Priming runs per side and group. Default: {DEFAULT_PRIME_RUNS}.",
    )
    parser.add_argument(
        "--measure-runs",
        type=int,
        default=DEFAULT_MEASURE_RUNS,
        help=f"Initial measured runs per side and group. Default: {DEFAULT_MEASURE_RUNS}.",
    )
    parser.add_argument(
        "--confirm-runs",
        type=int,
        default=DEFAULT_CONFIRM_RUNS,
        help=f"Measured runs per side for suspect groups. Default: {DEFAULT_CONFIRM_RUNS}.",
    )
    parser.add_argument(
        "--threshold-pct",
        type=float,
        default=DEFAULT_THRESHOLD_PCT,
        help=f"Minimum percent change to confirm. Default: {DEFAULT_THRESHOLD_PCT:.1f}.",
    )
    parser.add_argument(
        "--include-overhead",
        action="store_true",
        help="Include the overhead benchmark in pass/fail decisions.",
    )
    parser.add_argument(
        "--groups",
        nargs="*",
        default=None,
        help="Optional explicit group list. Defaults to all groups in benches/basic_zen.rs.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    groups = args.groups or parse_groups()
    if not groups:
        raise SystemExit("No benchmark groups found in benches/basic_zen.rs.")
    if not args.include_overhead:
        groups = [group for group in groups if group != "overhead"]

    tmp_dir, baseline_cwd = create_worktree(args.git_ref)
    candidate_cwd = ROOT

    try:
        print(f"baseline  {args.git_ref} ({git_short_hash_for_ref(ROOT, args.git_ref)})")
        print(f"candidate HEAD ({git_short_hash(candidate_cwd)})")
        confirm_required = confirm_agreement_required(args.confirm_runs)
        print(
            f"policy    core={args.cpu} threshold={args.threshold_pct:.1f}% "
            f"prime={args.prime_runs} initial={args.measure_runs} "
            f"confirm={args.confirm_runs} paired>={confirm_required}/{args.confirm_runs}"
        )
        print()

        confirmed: list[GroupResult] = []
        provisional: list[GroupResult] = []

        for group in groups:
            result = compare_group(
                baseline_cwd,
                candidate_cwd,
                tmp_dir,
                args.cpu,
                group,
                args.prime_runs,
                args.measure_runs,
            )

            if exceeds_threshold(result, args.threshold_pct):
                provisional.append(result)
                print_result(result, "suspect")
            else:
                print_result(result, "ok")

        if provisional:
            print()
            print("confirming flagged groups...")
            print()

        for result in provisional:
            confirmed_result = compare_group_paired(
                baseline_cwd,
                candidate_cwd,
                tmp_dir,
                args.cpu,
                result.name,
                args.prime_runs,
                args.confirm_runs,
            )
            if exceeds_threshold(confirmed_result, args.threshold_pct) and has_directional_agreement(
                confirmed_result,
                confirm_required,
            ):
                confirmed.append(confirmed_result)
                print_result(confirmed_result, "confirmed")
            else:
                print_result(confirmed_result, "cleared")

        print()
        if confirmed:
            print(f"meaningful changes: {len(confirmed)}")
            return 1

        print("meaningful changes: 0")
        return 0
    finally:
        remove_worktree(tmp_dir, baseline_cwd)


if __name__ == "__main__":
    sys.exit(main())
