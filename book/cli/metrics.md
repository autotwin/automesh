# Metrics

`metrics` evaluates element quality for an existing finite element mesh and
writes one quality value per element (as `csv` or `npy`), so mesh quality can
be assessed before analysis.  Poorly shaped elements — thin, skewed, or
inverted — degrade the accuracy and stability of a simulation, and these
metrics quantify that shape quality.

```sh
automesh metrics --help
<!-- cmdrun automesh metrics --help -->
```

The available metrics depend on the element type.  Each is defined, with
acceptable ranges from Knupp *et al.*[^Knupp_2006], in [Theory](../theory/metrics_hexahedral.md):

* [Hexahedral metrics](../theory/metrics_hexahedral.md) — for eight-node brick
  elements: maximum edge ratio, minimum scaled Jacobian, maximum skew, and
  element volume.
* [Tetrahedral metrics](../theory/metrics_tetrahedral.md) — for four-node
  tetrahedral elements: maximum edge ratio, minimum scaled Jacobian, maximum
  skew, and element volume.
* [Triangular metrics](../theory/metrics_triangular.md) — for three-node
  triangular surface elements: maximum edge ratio, minimum scaled Jacobian,
  maximum skew, element area, and minimum angle.

[^Knupp_2006]: Knupp PM, Ernst CD, Thompson DC, Stimpson CJ, Pebay PP. The verdict geometric quality library. SAND2007-1751. Sandia National Laboratories (SNL), Albuquerque, NM, and Livermore, CA (United States); 2006 Mar 1. [link](https://www.osti.gov/servlets/purl/901967)
