# ReHQJ: the HexOpt paper and conspire

This note compares the per-hex objective of HexOpt in two sources.

- **Paper:** Tong and Zhang, "Fast and Robust Hexahedral Mesh Optimization via Augmented Lagrangian, L-BFGS, and Line Search," [arXiv 2410.11656](https://arxiv.org/pdf/2410.11656), Equations (2.5) and (2.6).
- **conspire:** branch `msj` at commit `515c78cee` (draft [mrbuche/conspire.rs#251](https://github.com/mrbuche/conspire.rs/pull/251)), file `src/geometry/mesh/quality/improve/msj/mod.rs`, function `terms`.

Notation: $`J`$ is the Jacobian, $`SJ`$ is the scaled Jacobian, $`\bar{e}`$ is the hex's average edge length, and $`\Theta`$ is the threshold.
Both sources hold $`\bar{e}`$ fixed when they take the gradient.

## Definitions

| case | paper, Eq. (2.5) and (2.6) | conspire |
|---|---|---|
| $`J \le 0`$ | $`J / \bar{e}`$ | $`J / \bar{e}`$ |
| $`J > 0`$, $`SJ \le \Theta`$ | $`SJ \, \bar{e}^2`$ | $`SJ \, \bar{e}^2`$ |
| $`SJ > \Theta`$ | $`\Theta`$ | $`\Theta \, \bar{e}^2`$ |

The two sources differ only in the third case.

## Units

$`J = \det(e_0, e_1, e_2)`$, where each edge vector has units of length.
$`SJ`$ and $`\Theta`$ have no units.

| case | paper | conspire |
|---|---|---|
| $`J \le 0`$ | length² | length² |
| $`J > 0`$, $`SJ \le \Theta`$ | length² | length² |
| $`SJ > \Theta`$ | none | length² |

## Properties of the conspire form

1. **Continuity at the threshold.**
   At $`SJ = \Theta`$, the second case gives $`\Theta \, \bar{e}^2`$.
   The conspire form gives the same value, so the objective is continuous there.
   The paper's form gives $`\Theta`$.
   It jumps by $`\Theta (\bar{e}^2 - 1)`$ unless $`\bar{e} = 1`$.
2. **The sign of a crossing.**
   The minimized function is $`L = -\sum \mathrm{ReHQJ} + \ldots`$, as in Eq. (2.8).
   Under the paper's form, a hex that rises above $`\Theta`$ changes $`L`$ by $`+\Theta (\bar{e}^2 - 1)`$.
   For $`\bar{e} > 1`$, $`L`$ rises.
   The Armijo condition requires $`L`$ to fall, so the jump counts against every step that lifts a hex above $`\Theta`$.
   For $`\bar{e} < 1`$, $`L`$ falls, so the jump rewards the crossing instead.
   The sign depends on the length unit of the mesh.
   On the snapped bone octree in the reference frame, $`\bar{e}`$ runs from 0.644 to 2.81, with a median of 1.253.
   On the scale-5 bone mesh in the STL's own units, $`\bar{e}`$ runs from 0.009 to 0.034.
   Under the conspire form, the crossing changes $`L`$ by zero.
3. **Units.**
   All three cases have units of length squared.
   conspire's types enforce this: each term is a `Quantity<Area>`.

For a hex that stays above $`\Theta`$ through a step, both forms are constants.
They give the same gradient and the same energy differences.
