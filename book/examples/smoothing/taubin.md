# Taubin Smoothing

We examine the Taubin smoothing algorithm on a sphere composed of hexahedral elements.
We created a two block (green inner volume, yellow outer volume) mesh in Cubit, then added normal noise to the hemisphere where the $x$ coordinate was positive.  We then applied Taubin smoothing to the noised model.

The Cubit and Python code used to generate the noised input file and figures is included [below](#source).  Alternatively, the `.inp` files can be downloaded directly from the table below:

name | size (MB) | md5 checksum
--- | :---: | :---:
[`sphere_res_1cm.inp`](https://1drv.ms/u/c/3cc1bee5e2795295/ER8M2M-kE7BDsv6q6HqyqcIB7e1R5TcKhc-rZt4_Q4QXSg?e=Cxa6ef) | `1.5` | `644ef573c257222bfd61dcfda7131c6a`
[`sphere_res_1cm_noised.inp`](https://1drv.ms/u/c/3cc1bee5e2795295/Eb-evDgLk-1GjUI-h-BjWE8B2boLnG5E8Adj5dfgtynCyw?e=u2kMWH) | `1.5` | `7031df475b972b15cf28bf2c5b69c162` 

The two-material `sphere_res_1cm_noised.inp` file is visualized below with and without a midplane cut.

iso | iso midplane | `xz` midplane
:---: | :---: | :---:
![sphere_10k.png](sphere_10k.png) | ![sphere_10k_iso_midplane.png](sphere_10k_iso_midplane.png) | ![sphere_10k_xz_midplane.png](sphere_10k_xz_midplane.png)
![sphere_10k_noised.png](sphere_10k_noised.png) | ![sphere_10k_iso_midplane_noised.png](sphere_10k_iso_midplane_noised.png) | ![sphere_10k_xz_midplane_noised.png](sphere_10k_xz_midplane_noised.png)

Figure: (Top row) sphere original configuration.  (Bottom row) noised sphere configuration, with normal random nodal displacement of the coordinates where $x > 0$.

## Taubin example

![sphere_surface_w_noise.png](sphere_surface_w_noise.png)

Figure: Excerpt from Taubin[^Taubin_1995b], Figure 3, showing a surface mesh original configuration, and after 10, 50, and 200 steps.

## automesh

We compare our volumetric results to the surface mesh presented by Taubin.[^Taubin_1995b] A *step* is either a "shrinking step" (deflating, smoothing $\lambda$ step) or an "un-shrinking step" (reinflating $\mu$ step).

The smoothing parameters used were the `autotwin` defaults,[^autotwin_defaults] the same as used in Taubin's Figure 3 example.

```sh
automesh smooth hex -i sphere_res_1cm_noised.inp -o s10.exo -n 10
```

```sh
automesh smooth hex -i sphere_res_1cm_noised.inp -o s50.exo -n 50
```

```sh
automesh smooth hex -i sphere_res_1cm_noised.inp -o s200.exo -n 200
```

front | iso | `xz` midplane
:---: | :---: | :---:
![s10.png](s10.png) | ![s10_iso.png](s10_iso.png) | ![s10_iso_half.png](s10_iso_half.png)
![s50.png](s50.png) | ![s50_iso.png](s50_iso.png) | ![s50_iso_half.png](s50_iso_half.png)
![s200.png](s200.png) | ![s200_iso.png](s200_iso.png) | ![s200_iso_half.png](s200_iso_half.png)

Figure. Smoothing results after 10 (top row), 50 (middle row), and 200 (bottom row) iterations.

The results demonstrate that our implementation of Taubin smoothing on volumetric meshes composed of hexahedral elements performs well. All smoothing operations completed within 7.5 ms. Unlike Laplace smoothing, which drastically reduces volume (e.g., -16% in 10 iterations), Taubin smoothing preserves volumes (e.g., +1% in 200 iterations).  Thus, the noise in the $x > 0$ hemisphere was effectively removed, with very small volumetric change. The $x < 0$ hemisphere did not degrade from its original configuration.

## Source

### `sphere.jou`

```sh
<!-- cmdrun cat sphere.jou -->
```

### `noise_augmentation.py`

```python
<!-- cmdrun cat noise_augmentation.py -->
```

## References

[^Taubin_1995b]: Taubin G. A signal processing approach to fair surface design. In *Proceedings of the 22nd annual conference on Computer graphics and interactive techniques* 1995 Sep 15 (pp. 351-358). [paper](https://dl.acm.org/doi/pdf/10.1145/218380.218473)

[^autotwin_defaults]: `autotwin` default Taubin parameters: $k_{\rm{\tiny PB}} = 0.1$, $\lambda = 0.6307$ $\implies$ $\mu = −0.6732$.
