import numpy as np
import matplotlib.pyplot as plt

# Define nodes a, b, c (orthogonal)
a = np.array([1, 0, 0])
b = np.array([0, 1, 0])
c = np.array([0, 0, 1])

# Initial e and arbitrary direction d
e0 = np.array([0, 0, 0])
d = np.array([1, 0.5, 0.2])  # Arbitrary direction
d = d / np.linalg.norm(d)  # Normalize

t_vals = np.linspace(-2, 4, 400)


def compute_metrics(t):
    e = e0 + t * d
    u = a - e
    v = b - e
    w = c - e

    # J = (u x v) . w
    j = np.dot(np.cross(u, v), w)

    # L = ||u|| * ||v|| * ||w||
    l = np.linalg.norm(u) * np.linalg.norm(v) * np.linalg.norm(w)

    # Scaled Jacobian
    sj = j / l if l != 0 else 0

    return j, sj


results = [compute_metrics(t) for t in t_vals]
j_vals, sj_vals = zip(*results)

# Create the plot
plt.figure(figsize=(10, 6))
plt.plot(t_vals, j_vals, label="Jacobian ($J$)", color="blue", linewidth=2)
plt.plot(
    t_vals, sj_vals, label="Scaled Jacobian ($\hat{J}$)", color="red", linestyle="--", linewidth=2
)

# Styling
plt.axhline(0, color="black", linewidth=0.8, linestyle="-")
plt.axvline(0, color="gray", linewidth=0.8, linestyle=":")

# --- NEW UPDATED LINES ---
plt.axhline(1, color="lightgray", linewidth=1, linestyle="--", zorder=0)
plt.axhline(-1, color="lightgray", linewidth=1, linestyle="--", zorder=0)
# -------------------------

plt.xlabel("Displacement $t$ along arbitrary direction $\mathbf{d}$", fontsize=12)
plt.ylabel("Metric Value", fontsize=12)
plt.title("Jacobian and Scaled Jacobian as Node $e$ Moves", fontsize=14)
plt.legend()
plt.grid(True, alpha=0.3)

# Save image
plt.savefig("jacobian_and_scaled_jacobian.png")
plt.show()
