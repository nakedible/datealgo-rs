#!/usr/bin/env python3
"""Report the comparison of scripts/app_ab.sh, as described in ab_stats.py.

The change of each domain is the change of its time per event, as a share of
the time it spends in calendar code: the difference to the same domain without
calendar work. The overall change is the mean of the domains' changes,
weighted by their weights, computed for each pair of runs and compared over
the pairs like the domains. A domain's change is reported at a level that
keeps the chance of reporting any domain falsely at 5%.
"""

import glob
import re
import statistics
import sys

import ab_stats

dump, chunk, ref, pairs, secs, cpus = sys.argv[1], int(sys.argv[2]), *sys.argv[3:]

# The domains and their weights, from their sources
weights = {}
for path in sorted(glob.glob("benches/app/*.rs")):
    src = open(path).read()
    name = re.search(r'const NAME: &\'static str = "(\w+)"', src)
    weight = re.search(r"const WEIGHT: u32 = (\d+)", src)
    if name and weight:
        weights[name[1]] = int(weight[1])
domains = sorted(weights, key=lambda d: (-weights[d], d))
total = sum(weights.values())

obs = ab_stats.load(dump, [d for d in domains] + [d + "_none" for d in domains], chunk)
pair_ids = sorted(obs[domains[0]])
cal = {d: ab_stats.base_mean(obs[d]) - ab_stats.base_mean(obs[d + "_none"]) for d in domains}
cal_head = {d: ab_stats.head_mean(obs[d]) - ab_stats.head_mean(obs[d + "_none"]) for d in domains}


def per_pair(bench, domain):
    """Change of a benchmark in each pair, as percent of the domain's
    calendar time"""
    return [statistics.fmean(h - b for b, h in obs[bench][p]) / cal[domain] * 100 for p in pair_ids]


def interval(values, confidence):
    return ab_stats.change({i: [(0, v)] for i, v in enumerate(values)}, confidence)


def verdict(lo, hi):
    return "faster" if hi < 0 else "slower" if lo > 0 else ""


changes = {d: per_pair(d, d) for d in domains}
controls = {d: per_pair(d + "_none", d) for d in domains}
overall = [sum(weights[d] * changes[d][i] for d in domains) / total for i in range(len(pair_ids))]
control = [sum(weights[d] * controls[d][i] for d in domains) / total for i in range(len(pair_ids))]

m, lo, hi = interval(overall, 0.95)
v = verdict(lo, hi)
print(f"The candidate is {v}." if v else "No significant change.")
cm, clo, chi = interval(control, 0.95)
if clo > 0 or chi < 0:
    print("But the application without calendar work changed too, so the result may be unreliable.")
print()
print(f"Overall change of calendar time: {m:+.1f}% (95% CI {lo:+.1f}% to {hi:+.1f}%)")
print(f"Change without calendar work, which should be none: {cm:+.1f}% (95% CI {clo:+.1f}% to {chi:+.1f}%)")
print()
level = 1 - 0.05 / len(domains)
print(f"{'domain':10} {'weight':>6} {'base ns':>8} {'head ns':>8}   change ({level * 100:.1f}% CI)")
for d in domains:
    m, lo, hi = interval(changes[d], level)
    print(f"{d:10} {weights[d]:5}% {cal[d]:8.2f} {cal_head[d]:8.2f}   {m:+6.1f}% [{lo:+6.1f}% {hi:+6.1f}%]  {verdict(lo, hi)}")
print()
print(f"Baseline {ref}, candidate the working tree, {pairs} pairs of runs of {secs} s per benchmark on CPUs {cpus},")
print("each pair with another code layout of both. Times are calendar time per event.")
