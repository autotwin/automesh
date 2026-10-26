"""Write the bone surface in the frame of the meshes, for automesh improve.

The meshes are scaled first and then translated, so the surface is too.
"""

import meshio
import numpy as np

SCALE = 105.5645168
TRANSLATE = np.array([-3.0485977, -2.8101274, -2.7887506])

surface = meshio.read("bone_tri_cleaned.stl")
meshio.write(
    "bone_frame.stl",
    meshio.Mesh(surface.points * SCALE + TRANSLATE, surface.cells),
    binary=True,
)
