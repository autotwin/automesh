# Conforming Mesh

In this section, we develop a traditional conforming mesh, manually
constructed with Cubit.  We compare the results from the conforming
resolutions to the results obtained from the voxel mesh resolutions.

## Mesh Creation and Visualization

With [conforming_spheres.jou](conforming_spheres.jou) in Cubit, we create three conforming meshes to match the three voxel meshes of resolution 0.5, 0.25, and 0.1 cm (2 vox/cm, 4 vox/cm, and 10 vox/cm, respectively).

resolution | 2 vox/cm | 4 vox/cm | 10 vox/cm
---------- | -------: | -------: | --------:
midline   |  ![resolution_2c.png](img/resolution_2c.png) | ![resolution_3c.png](img/resolution_3c.png) | ![resolution_4c.png](img/resolution_4c.png)
isometric  | ![resolution_2c_iso.png](img/resolution_2c_iso.png) | ![resolution_3c_iso.png](img/resolution_3c_iso.png) | ![resolution_4c_iso.png](img/resolution_4c_iso.png)
block 1 (green) #elements | 57,344 | 458,752 | 7,089,776
block 2 (yellow) #elements | 18,432 | 98,304 | 1,497,840
block 3 (magenta) #elements | 18,432 | 98,304 | 1,497,840
total #elements | 94,208 | 655,360 | 10,085,456

Copy from local to HPC:

```sh
# for example, manual copy from local to HPC
# macOS local finder, command+K to launch "Connect to Server"
# smb://cee/chovey
# copy [local]~/autotwin/automesh/book/analysis/sphere_with_shells/conf_0.5cm.g
# to
# [HPC]~/autotwin/ssm/geometry/sr2c/
```

We consider three simulations using the following three meshes (in the HPC ~/autotwin/ssm/geometry folder):

*  `sr2c/conf_0.5cm.g`
*  `sr3c/conf_0.25cm.g`
*  `sr4/spheres_reolution_4.exo`

## Tracers

View the tracer locations in Cubit:

```sh
graphics clip on plane location 0 0 1 direction 0 0 -1
view up 0 1 0
view from 0 0 100
graphics clip manipulation off
```

![tracers_sr_2_3_4_conf.png](img/tracers_sr_2_3_4_conf.png)

Figure: Tracer numbers `[0, 1, 2, ... 11]` at $\Delta x$ distance `[0, 1, 2, ... 11]` centimeters from point `(0, 0, 0)` along the x-axis at resolutions `sr2c`, `sr3c`, and `sr4c` (top to bottom, respectively).

## Simulation

We created three input decks:

* [sr2c.i](https://github.com/autotwin/ssm/blob/main/input/sr2c/sr2c.i) (for mesh conf_0.5cm.g)
* [sr3c.i](https://github.com/autotwin/ssm/blob/main/input/sr3c/sr3c.i) (for mesh conf_0.25cm.g)
* sr4c.i (for mesh conf_0.1cm.g)

## Results

Compute time:

item | sim | T_sim (ms) | HPC | #proc | cpu time (hh:mm)
:---: | :---: | :---: | :---: | :---: | :---:
0 | sr2c.i | 20 | gho | 160 | 00:02 min
1 | sr3c.i | 20 | gho | 160 | 00:04 (xxx)
2 | sr4.i | 20 | att | 160 | 03:48 (xxx)

### Rigid Body

We verified the rigid body kinematics match those from the [voxel mesh](simulation.md#rigid-body), but we don't repeat those time history plots here.

### Deformable Body

resolution | 2 vox/cm | 4 vox/cm | 10 vox/cm
---------- | -------- | -------- | ---------
midline   | ![resolution_2c.png](img/resolution_2c.png) | ![resolution_3c.png](img/resolution_3c.png) | ![resolution_4c.png](img/resolution_4c.png)
t=0.000 s | ![max_prin_log_strain_sr2c_0000.png](img/max_prin_log_strain_sr3c_0000.png) | ![max_prin_log_strain_sr3c_0000.png](img/max_prin_log_strain_sr3c_0000.png) | ![max_prin_log_strain_sr4c_0000.png](img/max_prin_log_strain_sr4c_0000.png)
t=0.002 s | ![max_prin_log_strain_sr2c_0002.png](img/max_prin_log_strain_sr2c_0002.png) | ![max_prin_log_strain_sr3c_0002.png](img/max_prin_log_strain_sr3c_0002.png) | ![max_prin_log_strain_sr4c_0002.png](img/max_prin_log_strain_sr4c_0002.png)
t=0.004 s | ![max_prin_log_strain_sr2c_0004.png](img/max_prin_log_strain_sr2c_0004.png) | ![max_prin_log_strain_sr3c_0004.png](img/max_prin_log_strain_sr3c_0004.png) | ![max_prin_log_strain_sr4c_0004.png](img/max_prin_log_strain_sr4c_0004.png)
t=0.006 s | ![max_prin_log_strain_sr2c_0006.png](img/max_prin_log_strain_sr2c_0006.png) | ![max_prin_log_strain_sr3c_0006.png](img/max_prin_log_strain_sr3c_0006.png) | ![max_prin_log_strain_sr4c_0006.png](img/max_prin_log_strain_sr4c_0006.png)
t=0.008 s | ![max_prin_log_strain_sr2c_0008.png](img/max_prin_log_strain_sr2c_0008.png) | ![max_prin_log_strain_sr3c_0008.png](img/max_prin_log_strain_sr3c_0008.png) | ![max_prin_log_strain_sr4c_0008.png](img/max_prin_log_strain_sr4c_0008.png)
t=0.010 s | ![max_prin_log_strain_sr2c_0010.png](img/max_prin_log_strain_sr2c_0010.png) | ![max_prin_log_strain_sr3c_0010.png](img/max_prin_log_strain_sr3c_0010.png) | ![max_prin_log_strain_sr4c_0010.png](img/max_prin_log_strain_sr4c_0010.png)
t=0.012 s | ![max_prin_log_strain_sr2c_0012.png](img/max_prin_log_strain_sr2c_0012.png) | ![max_prin_log_strain_sr3c_0012.png](img/max_prin_log_strain_sr3c_0012.png) | ![max_prin_log_strain_sr4c_0012.png](img/max_prin_log_strain_sr4c_0012.png)
t=0.014 s | ![max_prin_log_strain_sr2c_0014.png](img/max_prin_log_strain_sr2c_0014.png) | ![max_prin_log_strain_sr3c_0014.png](img/max_prin_log_strain_sr3c_0014.png) | ![max_prin_log_strain_sr4c_0014.png](img/max_prin_log_strain_sr4c_0014.png)
t=0.016 s | ![max_prin_log_strain_sr2c_0016.png](img/max_prin_log_strain_sr2c_0016.png) | ![max_prin_log_strain_sr3c_0016.png](img/max_prin_log_strain_sr3c_0016.png) | ![max_prin_log_strain_sr4c_0016.png](img/max_prin_log_strain_sr4c_0016.png)
t=0.018 s | ![max_prin_log_strain_sr2c_0018.png](img/max_prin_log_strain_sr2c_0018.png) | ![max_prin_log_strain_sr3c_0018.png](img/max_prin_log_strain_sr3c_0018.png) | ![max_prin_log_strain_sr4c_0018.png](img/max_prin_log_strain_sr4c_0018.png)
t=0.020 s | ![max_prin_log_strain_sr2c_0020.png](img/max_prin_log_strain_sr2c_0020.png) | ![max_prin_log_strain_sr3c_0020.png](img/max_prin_log_strain_sr3c_0020.png) | ![max_prin_log_strain_sr4c_0020.png](img/max_prin_log_strain_sr4c_0020.png)
displacement | ![displacement_sr2c.svg](img/displacement_sr2c.svg) | ![displacement_sr3c.svg](img/displacement_sr3c.svg) | ![displacement_sr4c.svg](img/displacement_sr4c.svg)
recipe | [displacement_sr2c.yml](xyfigure_recipes/displacement_sr2c.yml) | [displacement_sr3c.yml](xyfigure_recipes/displacement_sr3c.yml) | [displacement_sr4c.yml](xyfigure_recipes/displacement_sr4c.yml)
log strain | ![log_strain_sr2c.svg](img/log_strain_sr2c.svg) | ![log_strain_sr3c.svg](img/log_strain_sr3c.svg) | ![log_strain_sr4c.svg](img/log_strain_sr4c.svg)
recipe | [log_strain_sr2c.yml](xyfigure_recipes/log_strain_sr2c.yml) | [log_strain_sr3c.yml](xyfigure_recipes/log_strain_sr3c.yml) | [log_strain_sr4c.yml](xyfigure_recipes/log_strain_sr4c.yml)
rate of deformation | ![rate_of_deformation_sr2c.svg](img/rate_of_deformation_sr2c.svg) | ![rate_of_deformation_sr3c.svg](img/rate_of_deformation_sr3c.svg) | ![rate_of_deformation_sr4c.svg](img/rate_of_deformation_sr4c.svg)
recipe | [rate_of_deformation_sr2c.yml](xyfigure_recipes/rate_of_deformation_sr2c.yml) | [rate_of_deformation_sr3c.yml](xyfigure_recipes/rate_of_deformation_sr3c.yml) | [rate_of_deformation_sr4c.yml](xyfigure_recipes/rate_of_deformation_sr4c.yml)

Figure: Conforming mesh midline section, with maximum principal log strain at selected times from 0.000 s to 0.020 s (1,000 Hz sample rate, $\Delta t$ = 0.001 s), and tracer plots at 1 cm interval along the $y=x$ axis for displacement magnitude, log strain, and rate of deformation (4,000 Hz acquisition rate, $\Delta t$ = 0.00025 s).