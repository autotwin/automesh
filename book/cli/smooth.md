# Smooth

`smooth` adjusts the positions of the nodes in a finite element mesh, using
either Laplacian or Taubin smoothing.  Laplacian smoothing moves each node
toward the average
position of its neighbors, which reduces high-frequency noise but shrinks
the domain.  Taubin smoothing is a two-pass extension of Laplacian
smoothing — a smoothing pass followed by a re-expansion pass — that avoids
that shrinkage.  Hierarchical control can restrict which nodes are free to
move, so a mesh's exterior or interface geometry can be preserved during
smoothing.  The element type is detected automatically from the input mesh
file; there is no separate hex/tri subcommand to choose.  See
[Smoothing Theory](../theory/smoothing.md) for the full derivations.

```sh
automesh smooth --help
<!-- cmdrun automesh smooth --help -->
```

## Smooth Remesh

An optional `remesh` subcommand can be chained directly onto smoothing,
regardless of the input mesh's element type:

```sh
automesh smooth remesh --help
<!-- cmdrun automesh smooth remesh --help -->
```

## Examples

* [Laplace](../examples/smoothing/laplace.md) — a two-element example worked
  by hand, showing neighborhoods and node positions across iterations of
  Laplace smoothing.
* [Laplace with Hierarchical Control](../examples/smoothing/laplace_hierarchical.md) —
  the Bracket example, contrasting unrestricted Laplace smoothing with
  hierarchically-controlled smoothing that preserves prescribed geometry.
* [Taubin](../examples/smoothing/taubin.md) — a noised hexahedral sphere mesh,
  compared directly against Taubin's original paper figure.
* [Python Visualization](../examples/smoothing/python_visualization.md) — the
  source scripts used to generate the Laplace and hierarchical Laplace
  figures above.
