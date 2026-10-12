# Bone: before and after (issue #768)

These images compare the bone meshes on the book's Bone page (5.1.3) with the same commands run on 2026-10-12.

- **Before:** the page's Figures 5.1.3-4 and 5.1.3-5, made with `automesh` 0.4.7.
- **After:** `automesh` 0.5.1 at `407e5bd`, built against conspire `main` at `45c2b857`. That conspire is unreleased.

Both use the page's `mesh hex` commands, inputs, and frame.  The renders come from `bone_render.py` and `bone_render_quad.py` on the Bone page.

`surface_roughness.py` measures the angle between the normals of adjacent boundary quads.  Run it with `uv run --with meshio --with numpy python surface_roughness.py LABEL MESH.vtu ...`.
