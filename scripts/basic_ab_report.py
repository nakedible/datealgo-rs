#!/usr/bin/env python3
"""Screen and report the comparison of scripts/basic_ab.sh, as described in
ab_stats.py.

usage: basic_ab_report.py screen|report DUMP BATCH THRESHOLD BENCH...

`screen` prints the benchmarks that look changed in DUMP/screen: those whose
95% confidence interval excludes no change, or whose change is at least
THRESHOLD percent. `report` reads a description of the runs from standard
input and reports the changes confirmed in DUMP/confirm: those whose
confidence interval excludes no change at a level that keeps the chance of
any false report among all the benchmarks at 5%, and whose change is at least
THRESHOLD percent.
"""

import os
import sys

import ab_stats

mode, dump, batch, threshold, benches = sys.argv[1], sys.argv[2], int(sys.argv[3]), float(sys.argv[4]), sys.argv[5:]
screen = ab_stats.load(os.path.join(dump, "screen"), benches, batch)


def pct(bench, x):
    return x / ab_stats.base_mean(screen[bench]) * 100


if mode == "screen":
    for bench in benches:
        m, lo, hi = (pct(bench, x) for x in ab_stats.change(screen[bench], 0.95))
        if lo > 0 or hi < 0 or abs(m) >= threshold:
            print(bench)
    sys.exit()

confirm_dir = os.path.join(dump, "confirm")
suspects = [b for b in benches if os.path.exists(confirm_dir) and os.path.exists(os.path.join(confirm_dir, sorted(os.listdir(confirm_dir))[0], b + ".csv"))]
confirm = ab_stats.load(confirm_dir, suspects, batch) if suspects else {}
level = 1 - 0.05 / len(benches)

rows = []
faster, slower = [], []
for bench in benches:
    base, head = ab_stats.base_mean(screen[bench]), ab_stats.head_mean(screen[bench])
    s = [pct(bench, x) for x in ab_stats.change(screen[bench], 0.95)]
    row = f"{bench:24} {base:8.3f} {head:8.3f}  {s[0]:+6.1f}% [{s[1]:+6.1f}% {s[2]:+6.1f}%]"
    if bench in confirm:
        c = [pct(bench, x) for x in ab_stats.change(confirm[bench], level)]
        verdict = "not confirmed"
        if c[1] > 0 or c[2] < 0:
            if abs(c[0]) < threshold:
                verdict = f"under {threshold:g}%"
            else:
                verdict = "faster" if c[0] < 0 else "slower"
                (faster if c[0] < 0 else slower).append(bench)
        row += f"  {c[0]:+6.1f}% [{c[1]:+6.1f}% {c[2]:+6.1f}%]  {verdict}"
    rows.append(row)

if faster or slower:
    print("Confirmed changes: " + "; ".join(
        f"{label}: {', '.join(names)}" for label, names in (("faster", faster), ("slower", slower)) if names
    ) + ".")
else:
    print("No confirmed changes.")
if "overhead" in faster + slower:
    print("But the overhead of the benchmark itself changed too, so the results may be unreliable.")
print()
print(f"{'':24} {'base ns':>8} {'head ns':>8}  {'screening (95% CI)':26}  confirmation ({level * 100:.2f}% CI)")
for row in rows:
    print(row)
print()
print(sys.stdin.read().rstrip())
