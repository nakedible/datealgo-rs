#!/usr/bin/env python3

"""Compare gungraun instruction counts against a git ref on the same machine.

Instruction counts are deterministic for a given compiler, so a single run of
the baseline and the candidate is enough to compare them. The baseline is saved
as a gungraun baseline and the candidate is run against it. Only instructions
are compared: the cache simulation behind gungraun's other metrics, such as
estimated cycles, depends on memory layout and differs between two builds of
the same code.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
BENCH_SOURCE = pathlib.Path("benches") / "gungraun.rs"
DEFAULT_GIT_REF = "origin/master"
BASELINE_NAME = "base"

# Exit codes: no changes, changes found, baseline could not be benchmarked.
EXIT_UNCHANGED = 0
EXIT_CHANGED = 1
EXIT_NO_BASELINE = 3


def git_short_hash(cwd: pathlib.Path, ref: str = "HEAD") -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "--short", ref],
        cwd=cwd,
        text=True,
    ).strip()


def create_worktree(git_ref: str) -> tuple[pathlib.Path, pathlib.Path]:
    temp_dir = pathlib.Path(tempfile.mkdtemp(prefix="datealgo-compare-gungraun-", dir="/tmp"))
    worktree = temp_dir / "baseline"
    subprocess.run(["git", "worktree", "add", "--detach", str(worktree), git_ref], cwd=ROOT, check=True)
    return temp_dir, worktree


def remove_worktree(temp_dir: pathlib.Path, worktree: pathlib.Path) -> None:
    subprocess.run(["git", "worktree", "remove", "--force", str(worktree)], cwd=ROOT, check=False)
    shutil.rmtree(temp_dir, ignore_errors=True)


def run_bench(cwd: pathlib.Path, env: dict[str, str], bench_args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "bench", "--features", "gungraun-bench", "--bench", "gungraun", "--", *bench_args],
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def last_lines(text: str, count: int = 15) -> str:
    return "\n".join(text.rstrip().splitlines()[-count:])


def benchmark_name(summary: dict) -> str:
    name = summary["function_name"]
    if summary.get("id"):
        name += f".{summary['id']}"
    return name


def instructions(summary: dict) -> tuple[int | None, int | None]:
    values = summary["profiles"][0]["data"]["total"]["metrics"]["Ir"]["values"]
    return values.get("old"), values.get("new")


def load_summaries(gungraun_home: pathlib.Path) -> list[dict]:
    summaries = [json.loads(path.read_text()) for path in gungraun_home.rglob("summary.json")]
    return sorted(summaries, key=lambda s: (s["module_path"], s.get("id") or ""))


def format_change(old: int | None, new: int | None) -> str:
    if old is None:
        return f"{'new':>8s} -> {new:>8d}"
    if old == new:
        return f"{old:>8d}"
    pct = (new - old) / old * 100 if old else float("inf")
    return f"{old:>8d} -> {new:>8d} ({pct:+.2f}%)"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Compare gungraun instruction counts against a git ref.")
    parser.add_argument(
        "--git-ref",
        default=DEFAULT_GIT_REF,
        help=f"Baseline git ref to compare against. Default: {DEFAULT_GIT_REF}.",
    )
    parser.add_argument(
        "--show-identical",
        action="store_true",
        help="Print benchmarks whose counts did not change.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    temp_dir, baseline_cwd = create_worktree(args.git_ref)
    try:
        base_hash = git_short_hash(ROOT, args.git_ref)
        candidate_hash = git_short_hash(ROOT)
        print(f"baseline  {args.git_ref} ({base_hash})")
        print(f"candidate HEAD ({candidate_hash})")
        print()

        # Both runs share a private gungraun home for the baseline data. The
        # baseline builds in its own target dir: in a shared one, cargo gives
        # both checkouts the same artifact names and can reuse the baseline
        # binary for the candidate.
        gungraun_home = temp_dir / "gungraun"
        env = os.environ.copy()
        env["GUNGRAUN_HOME"] = str(gungraun_home)
        base_env = env | {"CARGO_TARGET_DIR": str(temp_dir / "target")}

        have_baseline = (baseline_cwd / BENCH_SOURCE).exists()
        if have_baseline:
            base = run_bench(baseline_cwd, base_env, [f"--save-baseline={BASELINE_NAME}"])
            if base.returncode != 0:
                have_baseline = False
                print("baseline benchmark run failed:")
                print(last_lines(base.stdout))
                print()
        else:
            print(f"baseline has no {BENCH_SOURCE}")
            print()

        candidate_args = ["--save-summary=json"]
        if have_baseline:
            candidate_args.append(f"--baseline={BASELINE_NAME}")
        candidate = run_bench(ROOT, env, candidate_args)
        if candidate.returncode != 0:
            print("candidate benchmark run failed:")
            print(last_lines(candidate.stdout))
            raise SystemExit(2)

        changed = []
        for summary in load_summaries(gungraun_home):
            name = benchmark_name(summary)
            old, new = instructions(summary)
            is_changed = have_baseline and old != new
            if is_changed:
                changed.append(name)
            if is_changed or args.show_identical or not have_baseline:
                counts = format_change(old, new) if have_baseline else f"{new:>8d}"
                print(f"{name:32s} {counts}")

        print()
        if not have_baseline:
            print("no baseline to compare against; showing candidate counts only")
            return EXIT_NO_BASELINE
        print(f"changed benchmarks: {len(changed)}")
        if changed:
            print(" ".join(changed))
            return EXIT_CHANGED
        return EXIT_UNCHANGED
    finally:
        remove_worktree(temp_dir, baseline_cwd)


if __name__ == "__main__":
    sys.exit(main())
