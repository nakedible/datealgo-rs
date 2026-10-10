#!/usr/bin/env python3
"""Report the comparison of scripts/app_ab.sh from the tango dumps of its runs.

The difference between the builds in a run is the difference between their
mean times per event over the paired samples of the run. Each pair of runs,
in which the builds swap CPUs, is one observation: the mean of the two
differences, in which a difference in speed between the CPUs cancels out. Each
pair uses its own code layouts, so the effect of the layout varies between
observations as it would between real builds. The
comparison is a t-test over the pairs, as samples within a run are not
independent of each other: they share the memory placement and CPUs of the
run's processes.
"""

import csv
import math
import os
import statistics
import sys

dump, chunk, ref, pairs, secs, cpus = sys.argv[1], int(sys.argv[2]), *sys.argv[3:]


def times(path):
    """Mean baseline and candidate time per event in a tango dump"""
    with open(path) as f:
        rows = [(float(b), float(c), int(k)) for b, c, k in csv.reader(f)]
    return [statistics.fmean(r[i] / (r[2] * chunk) for r in rows) for i in (0, 1)]


def t95(df):
    """Two-sided 95% quantile of Student's t distribution (Cornish-Fisher)"""
    z = 1.959964
    return z + (z**3 + z) / (4 * df) + (5 * z**5 + 16 * z**3 + 3 * z) / (96 * df**2)


# Base and head time per event of each benchmark in each run, by pair. The
# directory of a run is named by its pair, the CPU of the head build, and
# which build was the candidate.
obs = {"app": {}, "none": {}}
for run in os.listdir(dump):
    pair = int(run.split("-")[0])
    for bench, values in obs.items():
        b, c = times(os.path.join(dump, run, bench + ".csv"))
        values.setdefault(pair, []).append((b, c) if run.endswith("-head") else (c, b))


def mean(bench, i):
    return statistics.fmean(o[i] for runs in obs[bench].values() for o in runs)


base_cal = mean("app", 0) - mean("none", 0)
head_cal = mean("app", 1) - mean("none", 1)


def change(bench):
    """Mean change per event, and its 95% confidence interval"""
    diffs = [statistics.fmean(c - b for b, c in runs) for runs in obs[bench].values()]
    m = statistics.fmean(diffs)
    h = t95(len(diffs) - 1) * statistics.stdev(diffs) / math.sqrt(len(diffs))
    return m, m - h, m + h


def describe(m, lo, hi):
    pct = lambda x: x / base_cal * 100
    return f"{m:+.3f} ns per event, {pct(m):+.1f}% of calendar time (95% CI {pct(lo):+.1f}% to {pct(hi):+.1f}%)"


app, none = change("app"), change("none")
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
