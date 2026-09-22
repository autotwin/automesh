#!/usr/bin/env python3
"""Sweeps the `automesh mesh hex` options on the bone, and tabulates quality.

Each run meshes `bone_tri_cleaned.stl` and puts the mesh in the frame of the
Tong 2024 reference.  That frame scales the longest side of the bone to 100
units, and centers the bone in a cube of that side.  Lengths and volumes then
compare directly with the reference.  `automesh` writes its own quality CSV
for each mesh, through `--metrics`.

Lengths below are in the frame of the reference.  The script converts the
uniform spacing and the curvature tolerance to the units of the STL, which are
smaller by the factor SCALE.  The octree scale is a pure number and needs no
conversion.

The sweeps cover these levers.

    octree     the octree refinement scale, --scale
    balance    weak or strong balancing, and soft or snapped fit
    tolerance  the curvature tolerance, --tolerance
    uniform    a uniform lattice of a given spacing, --uniform
    smooth     Taubin or Laplace smoothing, with and without --hierarchical

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# bone_tri_cleaned.stl is not committed; see the Downloads section of bone.md
python3 bone_sweep.py bone_sweep.tsv              # every lever, about 2 minutes
python3 bone_sweep.py bone_sweep.tsv uniform      # one lever

Output
------
A tab-separated table, one row per run, and a summary of the worst element of
each lever on the terminal.
"""
import csv
import re
import statistics
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

STL = "bone_tri_cleaned.stl"
SCALE = 105.5645168
FRAME = [
    "--xscale", str(SCALE), "--yscale", str(SCALE), "--zscale", str(SCALE),
    "--xtranslate", "-3.0485977", "--ytranslate", "-2.8101274", "--ztranslate", "-2.7887506",
]
COLUMNS = [
    "lever", "label", "elements", "nodes", "min_msj", "msj_p1", "msj_p5", "msj_median",
    "below_0.6", "not_valid", "max_ar", "ar_above_10", "max_skew", "volume", "flags",
]


def native(*, length):
    """Converts a length in the frame of the reference to the units of the STL."""
    return f"{length / SCALE:.8f}"


def runs_build():
    """Returns the (lever, label, flags) triples of every sweep."""
    runs = []
    for scale in (4, 5, 6, 7, 8, 10):
        runs.append(("octree", f"scale {scale}", ["--scale", str(scale)]))
    for fit, fit_flags in (("soft", []), ("snap", ["--snap"])):
        for balance, balance_flags in (("weak", []), ("strong", ["--strong"])):
            runs.append(("balance", f"scale 7, {fit}, {balance}",
                         ["--scale", "7"] + fit_flags + balance_flags))
    for scale in (5, 6, 7):
        for tolerance in (1, 0.3, 0.1, 0.03, 0.01):
            runs.append(("tolerance", f"scale {scale}, tolerance {tolerance}",
                         ["--scale", str(scale), "--tolerance", native(length=tolerance)]))
    for spacing in (1.2, 1.3, 1.4, 1.5, 1.6, 1.65, 1.7, 1.8, 1.9, 2.0, 2.2):
        runs.append(("uniform", f"spacing {spacing}", ["--uniform", native(length=spacing)]))
    for base, base_label in ((["--uniform", native(length=1.65)], "uniform 1.65"),
                             (["--scale", "7"], "scale 7")):
        for method, iterations in (("Taubin", 5), ("Taubin", 20), ("Taubin", 50),
                                   ("Taubin", 100), ("Laplace", 5), ("Laplace", 20)):
            for hierarchical in (False, True):
                if method == "Laplace" and hierarchical:
                    continue
                flags = base + ["smooth", "-m", method, "-n", str(iterations)]
                if hierarchical:
                    flags.append("--hierarchical")
                label = f"{base_label}, {method} {iterations}" + (", hierarchical" if hierarchical else "")
                runs.append(("smooth", label, flags))
    return runs


def run_one(*, lever, label, flags, directory):
    """Meshes the bone with `flags`, and returns one row of the table."""
    path = Path(directory) / (re.sub(r"[^A-Za-z0-9.]+", "_", f"{lever}_{label}") + ".csv")
    # The smoothing options come after the mesh options, as a subcommand.
    split = flags.index("smooth") if "smooth" in flags else len(flags)
    command = (["automesh", "mesh", "hex", "-i", STL, "-o", str(path.with_suffix(".vtu"))]
               + flags[:split] + FRAME + ["--metrics", str(path)] + flags[split:])
    done = subprocess.run(command, capture_output=True, text=True)
    text = re.sub(r"\x1b\[[0-9;]*m", "", done.stdout)
    counts = re.findall(r"\[(\d+) elements, (\d+) nodes\]", text)
    if done.returncode != 0 or not path.exists() or not counts:
        return {"lever": lever, "label": label, "elements": "FAILED", "flags": " ".join(flags)}
    rows = list(csv.reader(open(path)))[1:]
    ar, msj, skew, volume = ([float(r[i]) for r in rows] for i in range(4))
    ordered = sorted(msj)
    n = len(msj)
    return {
        "lever": lever, "label": label, "elements": n, "nodes": int(counts[-1][1]),
        "min_msj": round(ordered[0], 4), "msj_p1": round(ordered[int(0.01 * (n - 1))], 3),
        "msj_p5": round(ordered[int(0.05 * (n - 1))], 3), "msj_median": round(statistics.median(msj), 3),
        "below_0.6": sum(x < 0.6 for x in msj), "not_valid": sum(x <= 0 for x in msj),
        "max_ar": round(max(ar), 2), "ar_above_10": sum(x > 10 for x in ar),
        "max_skew": round(max(skew), 3), "volume": round(sum(volume)), "flags": " ".join(flags),
    }


def main():
    if len(sys.argv) < 2:
        print(f"usage: {sys.argv[0]} <out.tsv> [lever ...]", file=sys.stderr)
        sys.exit(1)
    out, wanted = sys.argv[1], set(sys.argv[2:])
    runs = [r for r in runs_build() if not wanted or r[0] in wanted]
    with tempfile.TemporaryDirectory() as directory:
        with ThreadPoolExecutor(max_workers=4) as pool:
            rows = list(pool.map(lambda r: run_one(lever=r[0], label=r[1], flags=r[2],
                                                   directory=directory), runs))
    with open(out, "w") as f:
        f.write("\t".join(COLUMNS) + "\n")
        for row in rows:
            f.write("\t".join(str(row.get(c, "")) for c in COLUMNS) + "\n")
    print(f"{len(rows)} runs -> {out}")
    for lever in dict.fromkeys(r["lever"] for r in rows):
        group = [r for r in rows if r["lever"] == lever and r["elements"] != "FAILED"]
        if group:
            worst = [r["min_msj"] for r in group]
            print(f"  {lever:10} {len(group):3} runs   min Minimum Scaled Jacobian "
                  f"{min(worst):.3f} to {max(worst):.3f}   "
                  f"elements {min(r['elements'] for r in group)} to {max(r['elements'] for r in group)}")


if __name__ == "__main__":
    main()
