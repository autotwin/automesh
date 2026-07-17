from pathlib import Path

import numpy as np
import matplotlib.pyplot as plt

# Parameters for the circle
RADIUS = 50
center = (0, 0)
# center = (RADIUS, RADIUS)
MARGIN = 0.1 * RADIUS

SAVE = False
EXT = ".svg"  # ".pdf" | ".png" | ".svg"

# Create an array of angles from 0 to 2 pi
N_PTS = 36
theta = np.linspace(0, 2 * np.pi, N_PTS + 1)

# Parametric equations for the circle
x = center[0] + RADIUS * np.cos(theta)
y = center[1] + RADIUS * np.sin(theta)

# Create the plot
fig, ax = plt.subplots(figsize=(6, 6))

# Plot the points with circles
plt.scatter(x, y, s=20, color="blue", edgecolor="black", zorder=2)
# Connect the points with lines
plt.plot(x, y, color="orange", linewidth=2, zorder=1)

# Adding labels to each point
for i, label in enumerate(range(N_PTS)):
    plt.text(x[i], y[i], str(i), fontsize=8, ha="left", va="bottom", zorder=3)

plt.xlim(-(RADIUS + MARGIN), RADIUS + MARGIN)
plt.ylim(-(RADIUS + MARGIN), RADIUS + MARGIN)
# plt.axhline(0, color="black", linewidth=0.5, ls="--")
# plt.axvline(0, color="black", linewidth=0.5, ls="--")
plt.gca().set_aspect("equal", adjustable="box")

ss = f"Circle Boundary from {N_PTS} Points, "
ss += f"Radius {RADIUS}, Centered at {center}"
plt.title(ss)
plt.xlabel("x")
plt.ylabel("y")
plt.grid()
# plt.legend()
plt.show()

if SAVE:
    parent = Path(__file__).parent
    stem = Path(__file__).stem + "_r_" + str(RADIUS) + "_npts_" + str(N_PTS)
    fn = parent.joinpath(stem + EXT)
    # plt.savefig(fn, dpi=DPI, bbox_inches='tight')
    # fig.savefig(fn, dpi=DPI)
    fig.savefig(fn)
    print(f"Saved {fn}")
