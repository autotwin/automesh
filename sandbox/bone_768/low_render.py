"""Renders only the elements below a minimum scaled Jacobian, on a faint bone.

Run with pvpython:  low_render.py MESH.vtu METRICS.csv OUT.png --view front
"""
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "book/examples/gallery/academic"))
from bone_render import camera_set, image_crop, msj_add  # noqa: E402

from paraview.simple import (  # noqa: E402
    ColorBy, ExtractSurface, GetActiveViewOrCreate, GetColorTransferFunction,
    GetScalarBar, Render, SaveScreenshot, Show, Threshold, XMLUnstructuredGridReader,
)

parser = argparse.ArgumentParser()
parser.add_argument("mesh")
parser.add_argument("metrics")
parser.add_argument("out_png")
parser.add_argument("--view", default="front")
parser.add_argument("--below", type=float, default=0.6)
parser.add_argument("--vmin", type=float, default=0.0)
parser.add_argument("--ylorrd", action="store_true", help="reversed YlOrRd on [0, below]")
args = parser.parse_args()

view = GetActiveViewOrCreate("RenderView")
view.ViewSize = [1600, 1000]
view.Background = [1, 1, 1]
view.OrientationAxesVisibility = 0
try:
    view.UseColorPaletteForBackground = 0
except AttributeError:
    pass

reader = XMLUnstructuredGridReader(FileName=[args.mesh])
withmsj = msj_add(source=reader, metrics_path=args.metrics)

ghost = Show(ExtractSurface(Input=withmsj), view)
ghost.DiffuseColor = [0.6, 0.6, 0.6]
ghost.AmbientColor = [0.6, 0.6, 0.6]
ghost.Opacity = 0.12
ghost.ColorArrayName = ["POINTS", ""]

low = Threshold(Input=withmsj)
low.Scalars = ["CELLS", "MSJ"]
try:
    low.LowerThreshold, low.UpperThreshold = -1.0, args.below
except AttributeError:
    low.ThresholdRange = [-1.0, args.below]
shown = Show(low, view)
shown.SetRepresentationType("Surface With Edges")
shown.EdgeColor = [0.15, 0.15, 0.15]
if args.ylorrd:
    shown.EdgeColor = [0.6, 0.6, 0.6]
    shown.Specular = 0.0
    shown.Ambient = 0.35
    shown.Diffuse = 0.75
ColorBy(shown, ("CELLS", "MSJ"))
lut = GetColorTransferFunction("MSJ")
if args.ylorrd:
    hexes = ["800026", "bd0026", "e31a1c", "fc4e2a", "fd8d3c", "feb24c", "fed976", "ffeda0", "ffffcc"]
    points = []
    for i, h in enumerate(hexes):
        points += [args.vmin + (args.below - args.vmin) * i / (len(hexes) - 1)] + [int(h[k:k + 2], 16) / 255 for k in (0, 2, 4)]
    lut.RGBPoints = points
    lut.ColorSpace = "Lab"
    lut.RescaleTransferFunction(args.vmin, args.below)
else:
    lut.ApplyPreset("Viridis (matplotlib)", True)
    lut.RescaleTransferFunction(0.0, 1.0)
lut.AutomaticRescaleRangeMode = "Never"
bar = GetScalarBar(lut, view)
bar.Title = "Minimum Scaled Jacobian"
bar.ComponentTitle = ""
bar.TitleColor = bar.LabelColor = [0, 0, 0]
shown.SetScalarBarVisibility(view, True)

camera_set(view=view, source=reader, name=args.view)
Render(view)
SaveScreenshot(args.out_png, view, ImageResolution=[1600, 1000])
image_crop(path=args.out_png)
