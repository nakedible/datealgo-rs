"""Statistics of the comparisons of scripts/ab_lib.sh from tango's dumps.

The difference between the builds in a run is the difference between their
mean times over the paired samples of the run. Each pair of runs, in which the
builds swap CPUs, is one observation: the mean of the two differences, in
which a difference in speed between the CPUs cancels out. Each pair uses its
own code layouts, so the effect of the layout varies between observations as
it would between real builds. Comparisons are t-tests over the pairs, as
samples within a run are not independent of each other: they share the
memory placement and CPUs of the run's processes.
"""

import csv
import math
import os
import statistics


def times(path, per):
    """Mean baseline and candidate time per unit in a tango dump, where each
    iteration is `per` units"""
    with open(path) as f:
        rows = [(float(b), float(c), int(k)) for b, c, k in csv.reader(f)]
    return [statistics.fmean(r[i] / (r[2] * per) for r in rows) for i in (0, 1)]


def load(dump, benches, per):
    """Base and head time per unit of each benchmark in each run of a
    comparison, by benchmark and pair. The directory of a run is named by its
    pair, the CPU of the head build, and which build was the candidate."""
    obs = {bench: {} for bench in benches}
    for run in os.listdir(dump):
        pair = int(run.split("-")[0])
        for bench, pairs in obs.items():
            b, c = times(os.path.join(dump, run, bench + ".csv"), per)
            pairs.setdefault(pair, []).append((b, c) if run.endswith("-head") else (c, b))
    return obs


def base_mean(pairs):
    """Mean time of the base build"""
    return statistics.fmean(b for runs in pairs.values() for b, _ in runs)


def head_mean(pairs):
    """Mean time of the head build"""
    return statistics.fmean(h for runs in pairs.values() for _, h in runs)


def t_quantile(p, df):
    """Quantile of Student's t distribution (Cornish-Fisher expansion, within
    a percent for at least 10 degrees of freedom)"""
    z = statistics.NormalDist().inv_cdf(p)
    return (
        z
        + (z**3 + z) / (4 * df)
        + (5 * z**5 + 16 * z**3 + 3 * z) / (96 * df**2)
        + (3 * z**7 + 19 * z**5 + 17 * z**3 - 15 * z) / (384 * df**3)
    )


def change(pairs, confidence):
    """Mean change from base to head over the pairs, and its confidence
    interval at the given level"""
    diffs = [statistics.fmean(h - b for b, h in runs) for runs in pairs.values()]
    m = statistics.fmean(diffs)
    h = t_quantile(1 - (1 - confidence) / 2, len(diffs) - 1) * statistics.stdev(diffs) / math.sqrt(len(diffs))
    return m, m - h, m + h
