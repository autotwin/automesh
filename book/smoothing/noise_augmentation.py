r"""This module, noise_augmentation.py, adds noise to a finite element mesh
in the .inp format.

Example:
--------
source ~/autotwin/automesh/.venv/bin/activate
cd ~/autotwin/automesh/book/smoothing
python noise_augmentation.py
"""

from pathlib import Path
from typing import Final
import random


FILE_INPUT: Final[Path] = Path(__file__).parent.joinpath("sphere_res_1cm.inp")
FILE_OUTPUT: Final[Path] = Path(__file__).parent.joinpath("sphere_res_1cm_noised.inp")
SEED_VALUE: Final[int] = 42  # set a seed value for reproducibility
random.seed(SEED_VALUE)
# AMP: Final[float] = 0.0  # the amplitude of the noise, debug
AMP: Final[float] = 0.5  # the amplitude of the noise


def has_e_plus_minus(string_in: str) -> bool:
    """Utility function, if the input string has the format
    "E+", "e+", "E-", or "E-", then return True, otherwise False.
    """
    return (
        "E+" in string_in or "e+" in string_in or "E-" in string_in or "e-" in string_in
    )


def has_four_entries(string_in: str) -> bool:
    """Utility function that evaluates a string.  If the input has four
    entries, which is the format of the nodal coordinates, then return
    True, otherwise, return False."""
    return len(string_in.split(",")) == 4


with (
    open(FILE_INPUT, "r", encoding="utf-8") as fin,
    open(FILE_OUTPUT, "w", encoding="utf-8") as fout,
):
    for line in fin:
        # print(line)  # debugging
        if has_four_entries(line):
            # This might be a coordinate, investigate further.
            items = line.split(",")
            node, px, py, pz = tuple(i.strip() for i in items)

            # if all coordinates have the E+/e+ or E-/e- notation
            if all(has_e_plus_minus(k) for k in [px, py, pz]):
                # we noise only coordinates on the positive x half-space
                if float(px) > 0.0:
                    # Pick three random number between -1 and 1
                    rx = random.uniform(-1, 1)
                    ry = random.uniform(-1, 1)
                    rz = random.uniform(-1, 1)
                    # create the noise values
                    nx = AMP * rx
                    ny = AMP * ry
                    nz = AMP * rz
                    # create noisy values
                    qx = float(px) + nx
                    qy = float(py) + ny
                    qz = float(pz) + nz
                    formatted_line = (
                        f"{node:>8}, {qx:>15.6e}, {qy:>15.6e}, {qz:>15.6e}\n"
                    )
                    line = formatted_line  # overwrite with new noised line

        fout.write(line)

print(f"Wrote {FILE_OUTPUT}")
print("Done.")
