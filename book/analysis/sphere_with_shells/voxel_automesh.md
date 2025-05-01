# Voxel Mesh with `automesh`

Build the latest release version of `automesh`.

```sh
cd ~/autotwin/automesh
cargo build --release
...
   Compiling automesh v0.3.3 (/Users/chovey/autotwin/automesh)
    Finished `release` profile [optimized] target(s) in 2m 59s
```

Set up.

```sh
alias automesh='/Users/chovey/autotwin/automesh/target/release/automesh'
cd ~/autotwin/automesh/book/analysis/sphere_with_shells/
```

## Mesh Creation

Use `automesh` to convert the segmentations into finite element meshes.

**Remark:** In the analysis below, we use the Exodus II output format (`.exo`) instead of the Abaqus output format (`.inp`).  The Exodus format results in faster mesh creation and smaller file size due to compression.

### Resolution 1

```sh
automesh mesh hex -i spheres_resolution_1.npy \
-o spheres_resolution_1.exo \
--remove 0 \
--xtranslate -12 --ytranslate -12 --ztranslate -12
```

```sh
    automesh 0.3.3
     Reading spheres_resolution_1.npy
        Done 5.410958ms [4 materials, 13824 voxels]
     Meshing voxels into hexes [xtranslate: -12, ytranslate: -12, ztranslate: -12]
        Done 513.125µs [3 blocks, 6272 elements, 7563 nodes]
     Writing spheres_resolution_1.exo
        Done 9.939916ms
       Total 16.381583ms
```

### Resolution 2

```sh
automesh mesh hex -i spheres_resolution_2.npy \
-o spheres_resolution_2.exo \
--remove 0 \
--xscale 0.5 --yscale 0.5 --zscale 0.5 \
--xtranslate -12 --ytranslate -12 --ztranslate -12
```

```sh
    automesh 0.3.3
     Reading spheres_resolution_2.npy
        Done 513.25µs [4 materials, 110592 voxels]
     Meshing voxels into hexes [xscale: 0.5, yscale: 0.5, zscale: 0.5, xtranslate: -12, ytranslate: -12, ztranslate: -12]
        Done 4.492458ms [3 blocks, 54088 elements, 59375 nodes]
     Writing spheres_resolution_2.exo
        Done 8.687541ms
       Total 16.362625ms
```

### Resolution 3

```sh
automesh mesh hex -i spheres_resolution_3.npy \
-o spheres_resolution_3.exo \
--remove 0 \
--xscale 0.25 --yscale 0.25 --zscale 0.25 \
--xtranslate -12 --ytranslate -12 --ztranslate -12
```

```sh
    automesh 0.3.3
     Reading spheres_resolution_3.npy
        Done 14.784291ms [4 materials, 884736 voxels]
     Meshing voxels into hexes [xscale: 0.25, yscale: 0.25, zscale: 0.25, xtranslate: -12, ytranslate: -12, ztranslate: -12]
        Done 27.708125ms [3 blocks, 448680 elements, 470203 nodes]
     Writing spheres_resolution_3.exo
        Done 33.148125ms
       Total 94.863584ms
```

### Resolution 4

```sh
automesh mesh hex -i spheres_resolution_4.npy \
-o spheres_resolution_4.exo \
--remove 0 \
--xscale 0.1 --yscale 0.1 --zscale 0.1 \
--xtranslate -12 --ytranslate -12 --ztranslate -12
```

```sh
    automesh 0.3.3
     Reading spheres_resolution_4.npy
        Done 2.846917ms [4 materials, 13824000 voxels]
     Meshing voxels into hexes [xscale: 0.1, yscale: 0.1, zscale: 0.1, xtranslate: -12, ytranslate: -12, ztranslate: -12]
        Done 430.685042ms [3 blocks, 7145880 elements, 7281019 nodes]
     Writing spheres_resolution_4.exo
        Done 464.796625ms
       Total 1.119838875s
```

## Visualization

Cubit is used for the visualizations with the following recipe:

```sh
reset
cd "/Users/chovey/autotwin/automesh/book/analysis/sphere_with_shells"

import mesh "spheres_resolution_1.exo" lite

graphics scale on

graphics clip off
view iso
graphics clip on plane location 0 -1.0 0 direction 0 1 0
view up 0 0 1
view from 100 -100 100

graphics clip manipulation off

view bottom
```

resolution | 1 vox/cm | 2 vox/cm | 4 vox/cm | 10 vox/cm
---------- | -------: | -------: | -------: | --------:
midline   | ![resolution_1.png](img/resolution_1.png) | ![resolution_2.png](img/resolution_2.png) | ![resolution_3.png](img/resolution_3.png) | ![resolution_4.png](img/resolution_4.png)
isometric  | ![resolution_1_iso.png](img/resolution_1_iso.png) | ![resolution_2_iso.png](img/resolution_2_iso.png) | ![resolution_3_iso.png](img/resolution_3_iso.png) | ![resolution_4_iso.png](img/resolution_4_iso.png)
block 1 (green) #elements | 3,648 | 31,408 | 259,408 | 4,136,832
block 2 (yellow) #elements | 1,248 | 10,400 | 86,032 | 1,369,056
block 3 (magenta) #elements | 1,376 | 12,280 | 103,240 | 1,639,992
total #elements | 6,272 | 54,088 | 448,680 | 7,145,880
Exodus file | [`spheres_resolution_1.exo`](https://1drv.ms/u/s!ApVSeeLlvsE8hIdqDek09Lk7k8KWlw?e=dnkMRZ) (401 kB) | [`spheres_resolution_2.exo`](https://1drv.ms/u/s!ApVSeeLlvsE8hIdt-bOdaS_qhgVlLQ?e=JRP9RV) (3.2 MB) | [`spheres_resolution_3.exo`](https://1drv.ms/u/s!ApVSeeLlvsE8hIdxXHN1SCJitmDgQQ?e=1mhAAn) (25.7 MB) | [`spheres_resolution_4.exo`](https://1drv.ms/u/s!ApVSeeLlvsE8hId0Z-rRjJPOisEYvQ?e=f0Lnbv) (404 MB)
Abaqus file | [`spheres_resolution_1.inp`](https://1drv.ms/u/s!ApVSeeLlvsE8hIdrwe59HupHMq485A?e=jnp8tC) (962 kB) | [`spheres_resolution_2.inp`](https://1drv.ms/u/s!ApVSeeLlvsE8hIdvMV0y35PFDc4E0Q?e=XXIa1D) (8.5 MB) | [`spheres_resolution_3.inp`](https://1drv.ms/u/s!ApVSeeLlvsE8hIdyGM8cuW115NwCCg?e=4ZslO4) (73.6 MB) | [`spheres_resolution_4.inp`](https://1drv.ms/u/s!ApVSeeLlvsE8hId1Y0wZ31lF74SrcQ?e=D7ZYbo) (1.23 GB)
