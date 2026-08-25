# Industry Models

Where [Academic Models](academic.md) are chosen for geometric pathology,
these are chosen for practical relevance: parts representative of real
mechanical design and analysis work — manufactured features, assemblies, and
engineering benchmarks. None are in the repository yet; sourcing is left
open rather than pointing at specific unverified links.

The table below tracks candidate models and their status. All entries are
currently "Planned" — a roadmap, not a claim that a model is already
available.

| Model | Description | Status |
| --- | --- | --- |
| Engine block | multi-cavity cast part; internal/external surfaces | Planned |
| Connecting rod | classic FE benchmark geometry | Planned |
| Bracket / mounting plate | sheet/machined part with holes and fillets | Planned |
| Gear | involute tooth profile; fine periodic surface detail | Planned |
| Turbine blade | thin curved aerofoil surface | Planned |
| Heat sink | dense fin array; thin-feature stress test | Planned |
| Pressure vessel | thin-shell enclosure | Planned |

## Representative Volume Element (RVE)

A Representative Volume Element (RVE) is the smallest sample volume of a heterogeneous material that accurately models the average physical and mechanical properties of the whole bulk material.

In this example we have a bulk material with two pore defects included in the bulk interior.

### Motivation

Current example uses tet4 or tet10, [image below], consisting of xxx nodes and xxx elements.  

model | tet4 | tet10
#nodes | xxx | xxx
#elements | xxx | xxx

### `automesh` Solution

1. Download the `rve.stl` model (todo: link to hosted model).

#TODO: Image of the original tesselation

2. Remesh the stl (Is a remesh needed?  If not delete this step.)

#TODO: Image of updated tessselation

3. Create an all-hexahedral volume mesh from the STL surface

```sh
automesh mesh hex ... # TODO finish command line specification
```

#TODO: Images of resulting hex mesh, including cutting planes showing the adaptivity in the interior, and likely cutting through the centers of both of the internal spherical voids.

4. Metrics

model | tet4 | tet10 | `automeh` hex
#nodes | xxx | xxx | xxx
#elements | xxx | xxx | xxx

* Four histograms showing quality measures: Max Edge Ratio, Min Scaled Jacobian, Max Skew, Element Volume