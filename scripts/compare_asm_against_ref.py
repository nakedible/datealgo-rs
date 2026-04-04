#!/usr/bin/env python3

"""Compare asm wrapper output against a git ref on the same machine."""

from __future__ import annotations

import argparse
import difflib
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
LIB_RS = ROOT / "src" / "lib.rs"
DEFAULT_GIT_REF = "origin/master"


def run(
    cmd: list[str],
    cwd: pathlib.Path,
    *,
    env: dict[str, str] | None = None,
    capture: bool = False,
) -> str | None:
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    if capture:
        return completed.stdout
    return None


def git_short_hash(cwd: pathlib.Path, ref: str = "HEAD") -> str:
    return subprocess.check_output(
        ["git", "rev-parse", "--short", ref],
        cwd=cwd,
        text=True,
    ).strip()


def create_worktree(git_ref: str) -> tuple[pathlib.Path, pathlib.Path]:
    temp_dir = pathlib.Path(tempfile.mkdtemp(prefix="datealgo-compare-asm-", dir="/tmp"))
    worktree = temp_dir / "baseline"
    run(["git", "worktree", "add", "--detach", str(worktree), git_ref], cwd=ROOT)
    return temp_dir, worktree


def remove_worktree(temp_dir: pathlib.Path, worktree: pathlib.Path) -> None:
    subprocess.run(["git", "worktree", "remove", "--force", str(worktree)], cwd=ROOT, check=False)
    shutil.rmtree(temp_dir, ignore_errors=True)


def parse_functions(lib_rs: pathlib.Path) -> list[str]:
    text = lib_rs.read_text()
    start = text.index("pub mod asm {")
    body = text[start:]
    pattern = re.compile(r"^\s*pub(?:\s+const)?\s+fn\s+([a-zA-Z0-9_]+)\(", re.MULTILINE)
    return pattern.findall(body)


def normalize_asm(text: str) -> list[str]:
    lines = []
    for raw_line in text.splitlines():
        line = raw_line.rstrip()
        if not line:
            continue
        if line.startswith("datealgo::asm::"):
            lines.append(line)
            continue
        line = re.sub(r"\.LBB\d+_", ".LBB_", line)
        line = re.sub(r"\.Ltmp\d+", ".Ltmp", line)
        lines.append(line)
    return lines


def dump_function_asm(cwd: pathlib.Path, function: str) -> list[str]:
    output = run(
        [
            "cargo",
            "asm",
            "--features=asmdump",
            "--simplify",
            "--lib",
            f"datealgo::asm::{function}",
        ],
        cwd=cwd,
        capture=True,
    )
    assert output is not None
    return normalize_asm(output)


def unified_diff(name: str, base_lines: list[str], candidate_lines: list[str]) -> str:
    return "".join(
        difflib.unified_diff(
            [line + "\n" for line in base_lines],
            [line + "\n" for line in candidate_lines],
            fromfile=f"{name}@base",
            tofile=f"{name}@candidate",
        )
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Compare asm wrapper output against a git ref.")
    parser.add_argument(
        "--git-ref",
        default=DEFAULT_GIT_REF,
        help=f"Baseline git ref to compare against. Default: {DEFAULT_GIT_REF}.",
    )
    parser.add_argument(
        "--functions",
        nargs="*",
        default=None,
        help="Optional explicit function list. Defaults to all wrappers in src/lib.rs asm module.",
    )
    parser.add_argument(
        "--show-identical",
        action="store_true",
        help="Print functions that matched exactly after normalization.",
    )
    parser.add_argument(
        "--summary-only",
        action="store_true",
        help="Only print changed/identical status and the final changed-function list.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    functions = args.functions or parse_functions(LIB_RS)
    if not functions:
        raise SystemExit("No asm wrapper functions found.")

    temp_dir, baseline_cwd = create_worktree(args.git_ref)
    candidate_cwd = ROOT
    try:
        base_hash = git_short_hash(ROOT, args.git_ref)
        candidate_hash = git_short_hash(candidate_cwd)
        print(f"baseline  {args.git_ref} ({base_hash})")
        print(f"candidate HEAD ({candidate_hash})")
        print()

        changed = []
        for function in functions:
            base_lines = dump_function_asm(baseline_cwd, function)
            candidate_lines = dump_function_asm(candidate_cwd, function)
            if base_lines == candidate_lines:
                if args.show_identical:
                    print(f"{function:24s} identical")
                continue
            changed.append(function)
            print(f"{function:24s} changed")
            if not args.summary_only:
                print(unified_diff(function, base_lines, candidate_lines), end="")

        print()
        print(f"changed functions: {len(changed)}")
        if changed:
            print(" ".join(changed))
            return 1
        return 0
    finally:
        remove_worktree(temp_dir, baseline_cwd)


if __name__ == "__main__":
    sys.exit(main())
