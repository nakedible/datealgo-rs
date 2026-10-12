"""Statistics of the comparisons of scripts/ab_lib.sh from tango's dumps.

The difference between the builds in a run is the mean of the middle half of
the differences between their paired samples. On shared machines, one of the
CPUs is now and then slowed for a while, as when another virtual machine
competes for it, which makes a share of a run's samples of one build much
slower; the middle half leaves them out. Each pair of runs, in which the
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
    iteration is `per` units, and the mean of the middle half of the
    differences of the candidate from the baseline in the paired samples"""
    with open(path) as f:
        rows = [(float(b) / (int(k) * per), float(c) / (int(k) * per)) for b, c, k in csv.reader(f)]
    diffs = sorted(c - b for b, c in rows)
    quarter = len(diffs) // 4
    middle = statistics.fmean(diffs[quarter : len(diffs) - quarter])
    return statistics.fmean(b for b, _ in rows), statistics.fmean(c for _, c in rows), middle


def load(dump, benches, per):
    """Base and head time per unit of each benchmark in each run of a
    comparison, and the difference of the head from the base, by benchmark
    and pair. The directory of a run is named by its pair, the CPU of the head
    build, and which build was the candidate."""
    obs = {bench: {} for bench in benches}
    for run in os.listdir(dump):
        pair = int(run.split("-")[0])
        for bench, pairs in obs.items():
            b, c, d = times(os.path.join(dump, run, bench + ".csv"), per)
            pairs.setdefault(pair, []).append((b, c, d) if run.endswith("-head") else (c, b, -d))
    return obs


def base_mean(pairs):
    """Mean time of the base build"""
    return statistics.fmean(b for runs in pairs.values() for b, _, _ in runs)


def head_mean(pairs):
    """Mean time of the head build"""
    return statistics.fmean(h for runs in pairs.values() for _, h, _ in runs)


def diffs(pairs):
    """Difference of the head from the base in each pair"""
    return [statistics.fmean(d for _, _, d in runs) for _, runs in sorted(pairs.items())]


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


def interval(values, confidence, fitted=0):
    """Mean of values from independent observations and its confidence
    interval at the given level, when `fitted` parameters were estimated from
    the same observations to compute them"""
    m = statistics.fmean(values)
    df = len(values) - 1 - fitted
    h = t_quantile(1 - (1 - confidence) / 2, df) * statistics.stdev(values) * math.sqrt((len(values) - 1) / df / len(values))
    return m, m - h, m + h


def trimmed_interval(values, confidence, fitted=0, trim=0.2):
    """Mean of values from independent observations, without the given share
    of them at each end, and its confidence interval at the given level by
    Yuen's test, when `fitted` parameters were estimated from the same
    observations to compute them"""
    n = len(values)
    k = int(trim * n)
    ordered = sorted(values)
    m = statistics.fmean(ordered[k : n - k])
    winsorized = [min(max(v, ordered[k]), ordered[n - k - 1]) for v in values]
    h = n - 2 * k
    se = math.sqrt((n - 1) * statistics.variance(winsorized) / (h * (h - 1 - fitted)))
    q = t_quantile(1 - (1 - confidence) / 2, h - 1 - fitted)
    return m, m - q * se, m + q * se


def change(pairs, confidence):
    """Mean change from base to head over the pairs, and its confidence
    interval at the given level"""
    return interval(diffs(pairs), confidence)
