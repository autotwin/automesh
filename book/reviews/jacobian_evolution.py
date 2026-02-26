import numpy as np
import matplotlib.pyplot as plt

# Define nodes a, b, c (orthogonal)
a = np.array([1, 0, 0])
b = np.array([0, 1, 0])
c = np.array([0, 0, 1])

# Initial x and arbitrary direction d
x0 = np.array([0, 0, 0])
d = np.array([1, 0.5, 0.2])  # Arbitrary direction
d = d / np.linalg.norm(d)  # Normalize

t_vals = np.linspace(-2, 4, 400)


def compute_metrics(t):
    """Compute Jacobian and Scaled Jacobian for a given displacement t."""
    x = x0 + t * d
    u = a - x
    v = b - x
    w = c - x

    # J = (u x v) . w
    j = np.dot(np.cross(u, v), w)

    # L = ||u|| * ||v|| * ||w||
    l_denom = np.linalg.norm(u) * np.linalg.norm(v) * np.linalg.norm(w)

    # Scaled Jacobian
    sj = j / l_denom if l_denom != 0 else 0

    return j, sj


results = [compute_metrics(t) for t in t_vals]
j_vals, sj_vals = zip(*results)

# Create a combined plot of Jacobian and Scaled Jacobian where scaled Jacobian
# is usesd if the Jacobian is positive, and the Jacobian is used if it is negative.
# Create combined_j_sj_vals: use sj if sj > 0, else use j
combined_j_sj_vals = [sj if sj > 0 else j for j, sj in zip(j_vals, sj_vals)]

# Create the plot
plt.figure(figsize=(10, 6))
plt.plot(
    t_vals,
    combined_j_sj_vals,
    label="Combined ($J$ or $\hat{J}$)",
    color="orange",
    linewidth=7,
    alpha=0.5,
)
plt.plot(t_vals, j_vals, label="Jacobian ($J$)", color="blue", linewidth=2, alpha=0.7)
plt.plot(
    t_vals,
    sj_vals,
    label="Scaled Jacobian ($\hat{J}$)",
    color="magenta",
    linestyle="--",
    linewidth=2,
    alpha=0.7,
)

# Styling
plt.axhline(0, color="black", linewidth=0.8, linestyle="-")
# plt.axvline(0, color="gray", linewidth=0.8, linestyle="-")

# Ideal target line (Green at y = +1)
plt.axhline(
    1, color="green", linewidth=1.0, linestyle="--", zorder=0, label="Ideal ($y=1$)"
)

# Lower bound limit (Red at y = -1)
plt.axhline(
    -1,
    color="red",
    linewidth=1.0,
    linestyle="--",
    zorder=0,
    label="Inverted Limit ($y=-1$)",
)

plt.xlabel("Displacement $t$ along arbitrary direction $\mathbf{d}$", fontsize=12)
plt.ylabel("Metric Value ($J$ and $\hat{J}$)", fontsize=12)
plt.title("Jacobian and Scaled Jacobian as Node $\mathbf{x}$ Moves", fontsize=14)
plt.legend()
plt.grid(True, alpha=0.3)

# Set the y-axis ticks explicitly
plt.yticks([-4, -3, -2, -1, 0, 1, 2, 3, 4])

# Save image
plt.savefig("jacobian_and_scaled_jacobian.png")
plt.show()
