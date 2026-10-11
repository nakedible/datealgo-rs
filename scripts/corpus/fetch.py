#!/usr/bin/env python3
"""Download the sources of the most downloaded crates that depend on chrono,
time and jiff from crates.io, for studying how they use date libraries.

usage: fetch.py [DIR]   DIR defaults to target/corpus

Writes DIR/crates.json, with the version, downloads and the date libraries of
each crate, and the sources to DIR/src/NAME-VERSION/. Crates that only
dev-depend on a library are left out, as their use is in tests. Sources
already downloaded are kept. The sources are only to be read, never built.
"""

import json
import os
import sys
import tarfile
import time
import urllib.request

# How many dependents to take of each library, in order of downloads
LIBRARIES = {"chrono": 1000, "time": 500, "jiff": 300}
HEADERS = {"User-Agent": "datealgo-corpus-study (https://github.com/nakedible/datealgo-rs)"}


def get(url):
    with urllib.request.urlopen(urllib.request.Request(url, headers=HEADERS)) as r:
        return r.read()


def dependents(lib, count):
    """The non-dev dependents of a crate among its `count` most downloaded"""
    out = []
    for page in range(1, (count + 99) // 100 + 1):
        d = json.loads(get(f"https://crates.io/api/v1/crates/{lib}/reverse_dependencies?per_page=100&page={page}"))
        versions = {v["id"]: v for v in d["versions"]}
        for dep in d["dependencies"]:
            v = versions.get(dep["version_id"])
            if v and dep["kind"] != "dev":
                out.append((v["crate"], v["num"], dep["downloads"]))
        # crates.io asks for at most one request per second
        time.sleep(1)
    return out


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "target/corpus"
    os.makedirs(os.path.join(root, "src"), exist_ok=True)
    crates = {}
    for lib, count in LIBRARIES.items():
        for name, version, downloads in dependents(lib, count):
            c = crates.setdefault(name, {"version": version, "downloads": downloads, "libraries": []})
            c["libraries"].append(lib)
    for name, c in sorted(crates.items()):
        dest = os.path.join(root, "src", f"{name}-{c['version']}")
        if os.path.exists(dest):
            continue
        try:
            data = get(f"https://static.crates.io/crates/{name}/{name}-{c['version']}.crate")
            path = dest + ".crate"
            with open(path, "wb") as f:
                f.write(data)
            with tarfile.open(path) as t:
                t.extractall(os.path.join(root, "src"), filter="data")
            os.remove(path)
        except Exception as e:
            print(f"failed: {name} {c['version']}: {e}", file=sys.stderr)
    with open(os.path.join(root, "crates.json"), "w") as f:
        json.dump(crates, f, indent=1, sort_keys=True)
    print(f"{len(crates)} crates in {root}")


if __name__ == "__main__":
    main()
