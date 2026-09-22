#!/usr/bin/env python3
"""Composes bone_render.py images into one labeled grid.

This is image compositing with Pillow.  Run it with plain Python, after
bone_render.py has written the PNGs.

With four panels and no `--label`, the grid is 2x2 and the panels read Top,
Front, Side, and Isometric, left to right and top to bottom.  With any other
number of panels, give one `--label` per panel, in order.  `--columns` sets the
width of the grid, and defaults to 2.

Example
-------
uv run --with pillow bone_render_quad.py \
    bone_top.png bone_front.png bone_side.png bone_iso.png bone_reference_msj.png

uv run --with pillow bone_render_quad.py bone_cut_uniform.png \
    bone_cut_octree.png bone_cut_msj.png --columns 2 \
    --label "automesh, uniform lattice" --label "automesh, adaptive octree"

Output
------
One PNG.
"""
import argparse
import math

from PIL import Image, ImageDraw, ImageFont

try:
    RESAMPLE = Image.Resampling.LANCZOS
except AttributeError:
    RESAMPLE = Image.LANCZOS

INK = (13, 13, 13)
SURFACE = (255, 255, 255)
LABELS = ["Top", "Front", "Side", "Isometric"]
CELL_PAD = 24
LABEL_H = 32


def font_load(*, size):
    for name in ("Helvetica.ttc", "Arial.ttf", "DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            continue
    return ImageFont.load_default()


def cell_build(*, path, label, cell_w, cell_h, font):
    """Returns one labeled panel of the grid."""
    img = Image.open(path).convert("RGB")
    img.thumbnail((cell_w, cell_h - LABEL_H), RESAMPLE)
    cell = Image.new("RGB", (cell_w, cell_h), SURFACE)
    draw = ImageDraw.Draw(cell)
    bbox = draw.textbbox((0, 0), label, font=font)
    tw = bbox[2] - bbox[0]
    draw.text(((cell_w - tw) // 2, 4), label, fill=INK, font=font)
    x = (cell_w - img.width) // 2
    y = LABEL_H + (cell_h - LABEL_H - img.height) // 2
    cell.paste(img, (x, y))
    return cell


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("paths", nargs="+", help="the panel PNGs, then the output PNG")
    parser.add_argument("--label", action="append", dest="labels",
                        help="a panel label; give one per panel, in order")
    parser.add_argument("--columns", type=int, default=2)
    args = parser.parse_args()
    if len(args.paths) < 2:
        parser.error("give at least one panel and the output path")
    paths, out_path = args.paths[:-1], args.paths[-1]
    if args.labels is not None:
        labels = args.labels
    elif len(paths) == 4:
        labels = LABELS
    else:
        parser.error("give one --label per panel when the number of panels is not four")
    if len(labels) != len(paths):
        parser.error(f"{len(paths)} panels but {len(labels)} labels")

    imgs = [Image.open(p) for p in paths]
    cell_w = max(im.width for im in imgs) + 2 * CELL_PAD
    cell_h = max(im.height for im in imgs) + 2 * CELL_PAD + LABEL_H
    font = font_load(size=18)

    cells = [cell_build(path=p, label=label, cell_w=cell_w, cell_h=cell_h,
                        font=font)
             for p, label in zip(paths, labels)]

    columns = min(args.columns, len(cells))
    rows = math.ceil(len(cells) / columns)
    grid = Image.new("RGB", (cell_w * columns, cell_h * rows), SURFACE)
    for i, cell in enumerate(cells):
        grid.paste(cell, ((i % columns) * cell_w, (i // columns) * cell_h))

    grid.save(out_path)
    print(f"{out_path}: {grid.width}x{grid.height}, cells {cell_w}x{cell_h}")


if __name__ == "__main__":
    main()
