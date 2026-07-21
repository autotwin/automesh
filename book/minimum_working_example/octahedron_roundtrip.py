"""The purpose of this module is to show that the
segmentation data encoded in two .npy files is the same.
"""

import numpy as np

aa = np.load("minimum_working_example/octahedron.npy")
print(aa)

bb = np.load("minimum_working_example/octahedron3.npy")
print(bb)

comparison = aa == bb
print(comparison)
result = np.all(comparison)
print(f"Element-by-element equality is {result}.")
