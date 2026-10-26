"""Write the bone surface in the frame of the meshes, and print its volume.

The meshes are scaled first and then translated, so the surface is too.
`automesh improve` and `automesh hausdorff` read the result.
"""

import meshio
import numpy as np

SCALE = 105.5645168
TRANSLATE = np.array([-3.0485977, -2.8101274, -2.7887506])

surface = meshio.read("bone_tri_cleaned.stl")
points = surface.points * SCALE + TRANSLATE
meshio.write("bone_frame.stl", meshio.Mesh(points, surface.cells), binary=True)

a, b, c = (points[surface.cells_dict["triangle"][:, i]] for i in range(3))
volume = np.einsum("ij,ij->i", a, np.cross(b, c)).sum() / 6
print(f"enclosed volume {volume:.1f}")
