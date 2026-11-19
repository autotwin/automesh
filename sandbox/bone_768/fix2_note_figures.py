"""Figures for fix2_note.tex: one hex's contribution to the augmented
Lagrangian, the stall-test ratio for one L-BFGS run, and the ratchet on the
snapped bone octree.  The numbers come from the msj debug logs on 2026-10-14.

uv run --with numpy --with matplotlib python fix2_note_figures.py
"""
import matplotlib
matplotlib.use("pdf")
import matplotlib.pyplot as plt
import numpy as np

SHIP, FIX, INK, GRID = "#eb6834", "#2f9e5b", "#0b0b0b", "#d9d8d4"
plt.rcParams.update({"font.size": 9, "axes.edgecolor": GRID, "axes.labelcolor": INK,
                     "xtick.color": INK, "ytick.color": INK, "axes.grid": True,
                     "grid.color": GRID, "grid.linewidth": 0.6})

# Figure 1: one hex's contribution to L, shipped and with fix 2.
theta, ell = 0.32, 1.253
sj = np.linspace(0.0, 1.0, 400)
shipped = -ell**2 * np.minimum(sj, theta)
fixed = ell**2 * np.maximum(theta - sj, 0.0)
fig, ax = plt.subplots(figsize=(5.2, 3.0))
ax.plot(sj, shipped, color=SHIP, lw=2, label=r"as shipped: $-\psi = -\ell^2 \min(\hat{J},\Theta)$")
ax.plot(sj, fixed, color=FIX, lw=2, ls="--", label=r"fix 2: $w = \ell^2 \langle\Theta-\hat{J}\rangle$")
ax.axvline(theta, color=INK, lw=0.8, ls=":")
ax.text(theta + 0.01, 0.22, r"$\Theta = 0.32$", color=INK)
ax.annotate("", xy=(0.75, 0.0), xytext=(0.75, -theta * ell**2),
            arrowprops=dict(arrowstyle="<->", color=INK, lw=0.8))
ax.text(0.77, -theta * ell**2 / 2, r"$\Theta\,\ell^2 = 0.502$", va="center", color=INK)
ax.set_xlabel(r"scaled Jacobian $\hat{J}$ of one hex")
ax.set_ylabel(r"contribution to $\mathcal{L}$ (length$^2$)")
ax.legend(frameon=False, loc="upper right")
fig.tight_layout()
fig.savefig("fix2_term.pdf")

# Figure 2: drop, total, and ratio at theta = 0.32, sweep 0.
drop = 1.186e-3
fig, (a, b) = plt.subplots(1, 2, figsize=(6.2, 2.8))
x = np.arange(2)
a.bar(x - 0.18, [drop, drop], 0.34, color=[SHIP, FIX], alpha=0.45, label="energy drop")
a.bar(x + 0.18, [6353.693, 1.186e-3], 0.34, color=[SHIP, FIX], label="|total energy|")
a.set_yscale("log")
a.set_xticks(x, ["as shipped", "fix 2"])
a.set_ylabel(r"length$^2$")
a.set_title("one L-BFGS run", color=INK)
for xi, v in zip(x - 0.18, [drop, drop]):
    a.text(xi, v * 1.6, "drop\n1.19e-3", ha="center", fontsize=7)
for xi, v, s in zip(x + 0.18, [6353.693, 1.186e-3], ["total\n6,354", "total\n1.19e-3"]):
    a.text(xi, v * 1.6, s, ha="center", fontsize=7)
a.set_ylim(1e-4, 1e5)
b.bar(x, [1.867e-7, 1.0], 0.5, color=[SHIP, FIX])
b.axhline(1e-6, color=INK, lw=1, ls="--")
b.text(0.5, 2.0e-6, "STALL = 1e-6", ha="center", fontsize=8, bbox=dict(facecolor="white", edgecolor="none", pad=1))
b.set_yscale("log")
b.set_ylim(1e-8, 10)
b.set_xticks(x, ["as shipped", "fix 2"])
b.set_ylabel("drop / |total|")
b.set_title("stall test", color=INK)
b.text(0, 1.5e-8 * 1.5, "1.9e-7\nstops", ha="center", fontsize=7)
b.text(1, 2.0, "1.0\ncontinues", ha="center", fontsize=7)
fig.tight_layout()
fig.savefig("fix2_ratio.pdf")

# Figure 3: the ratchet, worst element at each accept.
# Worst scaled Jacobian at each accepted level, from the msj debug logs of
# improve on the snapped octree: as shipped, and with fixes 1 and 2.
ship = [0.2911, 0.3188]
fixd = [0.2911, 0.3188, 0.322, 0.3396, 0.3419, 0.3514, 0.3686, 0.3701, 0.3802,
        0.3946, 0.4, 0.4103, 0.4203, 0.4302, 0.4401, 0.45, 0.46, 0.47]
fig, ax = plt.subplots(figsize=(5.2, 2.8))
ax.plot(range(1, len(fixd) + 1), fixd, "o-", color=FIX, ms=4, lw=1.5, label="fixes 1+2: 18 levels, ends at 0.470")
ax.plot(range(1, len(ship) + 1), ship, "s-", color=SHIP, ms=5, lw=2, label="as shipped: 2 levels, stalls at 0.319")
ax.axhline(0.263, color=INK, lw=0.8, ls=":")
ax.text(18, 0.268, "input 0.263", ha="right", fontsize=8)
ax.set_xlabel("accepted level")
ax.set_ylabel("worst scaled Jacobian")
ax.set_xticks(range(1, 19, 2))
ax.legend(frameon=False, loc="upper left")
fig.tight_layout()
fig.savefig("fix2_ratchet.pdf")
