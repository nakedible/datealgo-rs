#!/usr/bin/env python3
"""Report the comparison of scripts/app_ab.sh, as described in ab_stats.py."""

import sys

import ab_stats

dump, chunk, ref, pairs, secs, cpus = sys.argv[1], int(sys.argv[2]), *sys.argv[3:]

obs = ab_stats.load(dump, ("app", "none"), chunk)
base_cal = ab_stats.base_mean(obs["app"]) - ab_stats.base_mean(obs["none"])
head_cal = ab_stats.head_mean(obs["app"]) - ab_stats.head_mean(obs["none"])


def describe(m, lo, hi):
    pct = lambda x: x / base_cal * 100
    return f"{m:+.3f} ns per event, {pct(m):+.1f}% of calendar time (95% CI {pct(lo):+.1f}% to {pct(hi):+.1f}%)"


app, none = ab_stats.change(obs["app"], 0.95), ab_stats.change(obs["none"], 0.95)
if app[2] < 0:
    verdict = "The candidate is faster."
elif app[1] > 0:
    verdict = "The candidate is slower."
else:
    verdict = "No significant change."
if none[1] > 0 or none[2] < 0:
    verdict += " But the application without calendar work changed too, so the result may be unreliable."

print(verdict)
print()
print(f"Calendar time per event: {base_cal:.2f} ns in the baseline, {head_cal:.2f} ns in the candidate")
print(f"Change: {describe(*app)}")
print(f"Change without calendar work, which should be none: {describe(*none)}")
print()
print(f"Baseline {ref}, candidate the working tree, {pairs} pairs of runs of {secs} s on CPUs {cpus},")
print("each pair with another code layout of both")
