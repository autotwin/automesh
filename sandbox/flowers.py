"""Creates flower shaped meshes for illustration of quality metrics."""

import math

NFS = 2  # number of flowers
F0 = 3  # first flower, three petals
petals_per_flower = tuple(range(F0, F0 + NFS))  # subset used for dev

RAD_TO_DEG = 180.0 / math.pi
DEG_TO_RAD = 1.0 / RAD_TO_DEG

# angles, 120, 90, 72, etc
angles = tuple(int(360 / i) for i in petals_per_flower)
thetas = tuple(tuple(range(0, 360, k)) for k in angles)

xs = ()
ys = ()

for item in thetas:
    print(f"processing {item}")
    xi = tuple(math.cos(k * DEG_TO_RAD) for k in item)
    xs += (xi,)
    yi = tuple(math.sin(k * DEG_TO_RAD) for k in item)
    ys += (yi,)

# a3 = [dts[k] for k in range(0, vals[i]) for i in (0, 1)]

# aa = [ dts[i] * k] for i in dts for k in tuple(range())
# xs = [math.cos(a) for a in angles]
# yx = [math.sin(a) for a in angles]

breakpoint()

aa = 4
