#!/usr/bin/env python3
"""Renders a mesh with ParaView, from one named view.

Run it with ParaView's `pvpython`, not with plain `python3`.  The camera is
set from the data bounds, so the top, front, and side views are exact and
do not depend on ParaView's default camera.

Two kinds of render
-------------------
A triangulated surface (`.stl`) is drawn with its triangles in one flat
color.  A hex mesh (`.vtk` or `.vtu`) needs `--metrics`.  It is drawn with
each element colored by its Minimum Scaled Jacobian, on a color scale fixed
to [0, 1].  Two renders of two meshes are therefore comparable by color.

`--metrics` takes an `automesh metrics` CSV with the header "maximum edge
ratio,minimum scaled jacobian,maximum skew,element volume".  It has one row
per cell, in the same order as the cells of the mesh file.  The `--metrics`
option of `automesh mesh hex` writes such a CSV for the mesh it writes.

Example
-------
cd ~/autotwin/automesh/book/examples/gallery/academic
# none of the inputs are committed; see the Downloads section of bone.md
PVPYTHON=/Applications/ParaView-5.10.1.app/Contents/bin/pvpython
for view in top front side iso; do
    $PVPYTHON --force-offscreen-rendering bone_render.py \
        bone_tri_cleaned.stl bone_surface_$view.png --view $view
done
uv run --with pillow bone_render_quad.py bone_surface_top.png \
    bone_surface_front.png bone_surface_side.png bone_surface_iso.png \
    bone_surface.png

automesh metrics -i bone.inp -o bone_reference_metrics.csv
for view in top front side iso; do
    $PVPYTHON --force-offscreen-rendering bone_render.py \
        bone.vtk bone_msj_$view.png --view $view \
        --metrics bone_reference_metrics.csv
done
uv run --with pillow bone_render_quad.py bone_msj_top.png bone_msj_front.png \
    bone_msj_side.png bone_msj_iso.png bone_reference_msj.png

# a cut through two meshes, side by side
for mesh in uniform octree; do
    $PVPYTHON --force-offscreen-rendering bone_render.py \
        bone_$mesh.vtu bone_cut_$mesh.png --view front --cut \
        --metrics bone_${mesh}_metrics.csv
done
uv run --with pillow bone_render_quad.py bone_cut_uniform.png \
    bone_cut_octree.png bone_cut_msj.png --columns 2 \
    --label "automesh, uniform lattice" --label "automesh, adaptive octree"

The cut
-------
`--cut` cuts a hex mesh at the middle of its extent in Y, and keeps the far
half.  Use it with `--view front`, which looks along +Y, so that the cut face
turns toward the camera.  An element is kept whole when it crosses the plane,
and keeps its own color.  The cut edge is therefore ragged.

Views
-----
    iso    oblique, elevation 20 degrees and azimuth 30 degrees
    top    looking down -Z, with +Y up
    front  looking down -Y, with +Z up
    side   looking down -X, with +Z up

Output
------
One PNG, cropped to its content.
"""
import argparse
import csv
from pathlib import Path

import numpy as np
from vtkmodules.vtkIOImage import vtkPNGReader, vtkPNGWriter
from vtkmodules.vtkCommonDataModel import vtkImageData
from vtkmodules.util.numpy_support import numpy_to_vtk, vtk_to_numpy

from paraview.simple import (
    ColorBy, GetActiveViewOrCreate, GetColorTransferFunction, GetScalarBar,
    Clip, LegacyVTKReader, ProgrammableFilter, Render, ResetCamera, SaveScreenshot,
    Show, STLReader, XMLUnstructuredGridReader,
)

SURFACE_COLOR = [0.85, 0.85, 0.85]
EDGE_COLOR = [0.15, 0.15, 0.15]


def image_crop(*, path, background=(255, 255, 255), pad=40):
    """Trims the uniform margins from a saved PNG, in place.

    No camera can fit an irregular silhouette snugly.  An elongated bone
    at an angle leaves empty corners however the view is zoomed.  This
    crops the rendered pixels to their content plus a small pad.
    """
    reader = vtkPNGReader()
    reader.SetFileName(path)
    reader.Update()
    img = reader.GetOutput()
    w, h, _ = img.GetDimensions()
    ncomp = img.GetPointData().GetScalars().GetNumberOfComponents()
    arr = vtk_to_numpy(img.GetPointData().GetScalars()).reshape(h, w, ncomp)

    bg = np.array(background, dtype=int)
    diff = np.abs(arr[:, :, :3].astype(int) - bg).sum(axis=2)
    mask = diff > 10
    rows = np.where(mask.any(axis=1))[0]
    cols = np.where(mask.any(axis=0))[0]
    if rows.size == 0 or cols.size == 0:
        return
    # Not min() or max(): under pvpython those names are numpy's, and the
    # second argument becomes an axis.  The noqa keeps ruff from "fixing" it.
    r0 = int(rows[0]) - pad if int(rows[0]) - pad > 0 else 0  # noqa: FURB136
    r1 = int(rows[-1]) + pad if int(rows[-1]) + pad < h - 1 else h - 1  # noqa: FURB136
    c0 = int(cols[0]) - pad if int(cols[0]) - pad > 0 else 0  # noqa: FURB136
    c1 = int(cols[-1]) + pad if int(cols[-1]) + pad < w - 1 else w - 1  # noqa: FURB136
    cropped = arr[r0:r1 + 1, c0:c1 + 1, :]
    ch, cw = cropped.shape[:2]

    out = vtkImageData()
    out.SetDimensions(cw, ch, 1)
    vtk_arr = numpy_to_vtk(cropped.reshape(-1, ncomp), deep=True)
    vtk_arr.SetName("PNGImage")
    out.GetPointData().SetScalars(vtk_arr)

    writer = vtkPNGWriter()
    writer.SetFileName(path)
    writer.SetInputData(out)
    writer.Write()


def camera_set(*, view, source, name):
    """Points the camera of `view` at a named orientation, then fits it.

    The "iso" view is a rotation from ParaView's default orientation.  The
    "top", "front", and "side" views are absolute.  They come from the
    bounds of `source`.
    """
    cam = view.GetActiveCamera()
    if name == "iso":
        ResetCamera()
        cam.Elevation(20)
        cam.Azimuth(30)
        ResetCamera()
        return

    xmin, xmax, ymin, ymax, zmin, zmax = source.GetDataInformation().GetBounds()
    cx, cy, cz = (xmin + xmax) / 2, (ymin + ymax) / 2, (zmin + zmax) / 2
    diag = ((xmax - xmin) ** 2 + (ymax - ymin) ** 2 + (zmax - zmin) ** 2) ** 0.5
    d = diag * 3 if diag > 0 else 1.0
    directions = {
        # (position offset from the center, view-up)
        "top": ((0, 0, d), (0, 1, 0)),
        "front": ((0, -d, 0), (0, 0, 1)),
        "side": ((d, 0, 0), (0, 0, 1)),
    }
    (ox, oy, oz), up = directions[name]
    cam.SetFocalPoint(cx, cy, cz)
    cam.SetPosition(cx + ox, cy + oy, cz + oz)
    cam.SetViewUp(*up)
    ResetCamera()


def msj_add(*, source, metrics_path):
    """Attaches the Minimum Scaled Jacobian column to the cells of `source`."""
    pf = ProgrammableFilter(Input=source)
    pf.Script = f"""
import csv
import numpy as np
with open({str(metrics_path)!r}) as f:
    rows = list(csv.reader(f))
header, data = rows[0], rows[1:]
idx = header.index('minimum scaled jacobian')
msj = np.array([float(r[idx]) for r in data], dtype='float64')
output.ShallowCopy(inputs[0].VTKObject)
output.CellData.append(msj, 'MSJ')
"""
    return pf


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("mesh", help="a triangulated surface (.stl) or a hex mesh (.vtk or .vtu)")
    parser.add_argument("out_png")
    parser.add_argument("--view", choices=["iso", "top", "front", "side"],
                        default="iso")
    parser.add_argument("--metrics", help="an `automesh metrics` CSV; required for a hex mesh")
    parser.add_argument("--cut", action="store_true",
                        help="cut the hex mesh at the middle of Y; use with --view front")
    args = parser.parse_args()
    mesh_path, out_path = Path(args.mesh), args.out_png

    if mesh_path.suffix == ".stl":
        if args.metrics:
            parser.error("--metrics applies to a hex mesh, not a .stl surface")
        source = STLReader(FileNames=[str(mesh_path)])
    elif mesh_path.suffix in (".vtk", ".vtu"):
        if not args.metrics:
            parser.error("a hex mesh needs --metrics")
        if mesh_path.suffix == ".vtk":
            source = LegacyVTKReader(FileNames=[str(mesh_path)])
        else:
            source = XMLUnstructuredGridReader(FileName=[str(mesh_path)])
    else:
        parser.error(f"unsupported mesh type {mesh_path.suffix!r} (use .stl, .vtk, or .vtu)")
    if args.cut and (not args.metrics or args.view != "front"):
        parser.error("--cut needs a hex mesh with --metrics, and --view front")

    view = GetActiveViewOrCreate("RenderView")
    view.ViewSize = [1800, 1400]
    view.UseColorPaletteForBackground = 0
    view.Background = [1, 1, 1]
    view.OrientationAxesVisibility = 0

    if args.metrics:
        shown = msj_add(source=source, metrics_path=Path(args.metrics))
        if args.cut:
            shown.UpdatePipeline()
            xmin, xmax, ymin, ymax, zmin, zmax = shown.GetDataInformation().GetBounds()
            clip = Clip(Input=shown)
            clip.ClipType = "Plane"
            clip.ClipType.Origin = [(xmin + xmax) / 2, (ymin + ymax) / 2, (zmin + zmax) / 2]
            clip.ClipType.Normal = [0, 1, 0]
            clip.Invert = 0        # keep the +Y half, the side the normal points to
            clip.Crinkleclip = 1   # keep whole elements, so each keeps its own color
            shown = clip
        rep = Show(shown, view)
        ColorBy(rep, ("CELLS", "MSJ"))
        lut = GetColorTransferFunction("MSJ")
        lut.ApplyPreset("Viridis (matplotlib)", True)
        lut.RescaleTransferFunction(0.0, 1.0)
        rep.SetScalarBarVisibility(view, True)
        sb = GetScalarBar(lut, view)
        sb.Title = "Minimum Scaled Jacobian"
        sb.ComponentTitle = ""
        sb.TitleColor = [0.05, 0.05, 0.05]
        sb.LabelColor = [0.05, 0.05, 0.05]
        with open(args.metrics) as f:
            rows = list(csv.reader(f))
        column = rows[0].index("minimum scaled jacobian")
        values = [float(r[column]) for r in rows[1:]]
        note = (f"element MSJ range [{min(values):.6f}, {max(values):.6f}] in the full mesh, "
                f"color scale fixed to [0, 1]")
        if args.cut:
            shown.UpdatePipeline()
            note += f"; cut at y = {(ymin + ymax) / 2:.2f}, {shown.GetDataInformation().GetNumberOfCells()} elements kept"
    else:
        shown = source
        rep = Show(shown, view)
        # ColorBy(rep, None) fails in ParaView 5.10.1.  Clearing the array works.
        rep.ColorArrayName = [None, ""]
        rep.DiffuseColor = SURFACE_COLOR
        note = f"{source.GetDataInformation().GetNumberOfCells()} triangles"
    rep.Representation = "Surface With Edges"
    rep.EdgeColor = EDGE_COLOR
    rep.LineWidth = 1.0

    camera_set(view=view, source=shown, name=args.view)
    Render()
    SaveScreenshot(out_path, view)
    image_crop(path=out_path)
    print(f"{out_path}: {note}")


if __name__ == "__main__":
    main()
