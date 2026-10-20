# Improve

`improve` raises the worst [minimum scaled Jacobian](../theory/metrics_hexahedral.md)
of an existing hexahedral mesh.
It moves the nodes and keeps the boundary nodes on a target surface.
The target is a `stl` file in the same frame as the mesh.

```sh
automesh improve --help
<!-- cmdrun automesh improve --help -->
```

## Ratchet and Sweeps

`improve` works in levels.
Each level sets a threshold $\Theta$ just above the current worst element, rounded up to the next 0.01.
Within a level, one sweep is one run of the L-BFGS minimizer, up to 50 steps.
The minimizer pushes every element below $\Theta$ up toward it.
A level passes when the worst element reaches $\Theta$ and the boundary fits the target.
The fit holds when every boundary node lies within $10^{-3}$ of the local element size from the target.
`improve` then raises $\Theta$ and starts the next level.

A level fails when its sweeps run out, or when a sweep lowers the energy by less than one part in $10^{6}$.
`improve` then stops and writes the mesh from the last level that passed.
If the input already fits the target, as a mesh from `mesh hex --snap` does, the output is never worse than the input.
A soft-fit input starts off the target, so its first accepted mesh can be worse than the input.

`--sweeps` sets the sweep budget for each level.
The default is 20.
On an input that fits the target, `--sweeps 0` runs no minimizer and writes the input mesh unchanged.

## Example

The unit sphere from [Remesh](../examples/remesh/sphere.md), meshed with `-s 5 --snap`, has 111 hexahedra.
Its worst element starts at 0.520.

```sh
automesh mesh hex -i sphere_radius_1.stl -o sphere.vtu -s 5 --snap
automesh improve -i sphere.vtu -o sphere_improved.vtu -t sphere_radius_1.stl --sweeps 1
```

| `--sweeps` | worst minimum scaled Jacobian |
|:----------:|:-----------------------------:|
| 0          | 0.520 (unchanged)             |
| 1          | 0.651                         |
| 20 (default) | 0.660                       |

More sweeps help on harder meshes, at a cost in time.
On the adaptive bone mesh of [issue #768](https://github.com/autotwin/automesh/issues/768), 20 sweeps reach 0.470 in 10.7 s.
50 sweeps reach 0.480 in 23.8 s.
