"""Plots the per-hex term psi for the HexOpt note on one axis: psi against J
for an inverted hex (J <= 0), then psi_p and psi_c against J-hat for J > 0.
Braces under the axis mark the two domains.

uv run --with numpy --with matplotlib python hexopt_psi.py
"""
import sys

import matplotlib
matplotlib.use("agg")
import matplotlib.pyplot as plt
import matplotlib.transforms as transforms
import numpy as np

PAPER, CONSPIRE, INK, GRID = "#2a78d6", "#eb6834", "#0b0b0b", "#d9d8d4"
plt.rcParams.update({"font.size": 9, "axes.edgecolor": GRID, "axes.labelcolor": INK,
                     "xtick.color": INK, "ytick.color": INK, "axes.grid": True,
                     "grid.color": GRID, "grid.linewidth": 0.6})


def brace_draw(*, ax, x0, x1, y, height, label):
    """Draws an underbrace from x0 to x1 in data units, hanging from height y
    in axes units, with its point at the middle."""
    trans = transforms.blended_transform_factory(ax.transData, ax.transAxes)
    t = np.linspace(0.0, 0.5, 300)
    k = 40.0
    half = 1.0 / (1.0 + np.exp(-k * t)) + 1.0 / (1.0 + np.exp(-k * (t - 0.5)))
    curve = np.concatenate([half, half[::-1]])
    curve = (curve - curve.min()) / (curve.max() - curve.min())
    x = np.linspace(x0, x1, curve.size)
    ax.plot(x, y - height * curve, color=INK, lw=0.9, transform=trans, clip_on=False)
    ax.text((x0 + x1) / 2, y - height - 0.03, label, ha="center", va="top",
            transform=trans, color=INK)


theta, ell = 0.32, 1.253
fig, ax = plt.subplots(figsize=(6.4, 3.4))

j = np.linspace(-1.0, 0.0, 50)
ax.plot(j, j / ell, color=INK, lw=2, label=r"both: $J/\ell$ for $J \leq 0$, $\ell^2 \hat J$ up to $\Theta$")
jh = np.linspace(0.0, 1.0, 400)
below = jh <= theta
ax.plot(jh[below], ell**2 * jh[below], color=INK, lw=2)
ax.plot(jh[~below], np.full((~below).sum(), theta * ell**2), color=CONSPIRE, lw=2,
        label=r"$\psi_{\mathrm{c}} = \ell^2 \min(\hat J, \Theta)$")
ax.plot(jh[~below], np.full((~below).sum(), theta), color=PAPER, lw=2, ls="--",
        label=r"$\psi_{\mathrm{p}} = \Theta$ above $\Theta$")
ax.axvline(0.0, color=INK, lw=0.8)
ax.axvline(theta, color=INK, lw=0.8, ls=":")
ax.text(theta + 0.02, -0.75, r"$\Theta = 0.32$", color=INK, fontsize=8)
ax.annotate("", xy=(0.7, theta), xytext=(0.7, theta * ell**2),
            arrowprops=dict(arrowstyle="<->", color=INK, lw=0.8))
ax.text(0.72, theta * (1 + ell**2) / 2, r"jump $\Theta(\ell^2 - 1)$", va="center", fontsize=8)
ax.set_xlim(-1.0, 1.0)
ax.set_ylim(-0.85, 0.65)
ax.set_ylabel(r"$\psi$")
ax.legend(frameon=False, loc="upper left", fontsize=8)
brace_draw(ax=ax, x0=-0.99, x1=-0.01, y=-0.12, height=0.07,
           label=r"$J$ (length$^3$), inverted hex, $J \leq 0$")
brace_draw(ax=ax, x0=0.01, x1=0.99, y=-0.12, height=0.07,
           label=r"$\hat J$, with $J > 0$")
fig.subplots_adjust(left=0.1, right=0.98, top=0.97, bottom=0.26)
out = sys.argv[1] if len(sys.argv) > 1 else "hexopt_psi.pdf"
fig.savefig(out, dpi=200)
