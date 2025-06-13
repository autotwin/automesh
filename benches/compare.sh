#!/bin/bash

mkdir -p benches/block/ benches/compare/

rm -f benches/compare/automesh_block.out
touch benches/compare/automesh_block.out

for NUM in 100 107 115 124 133 143 154 165 178 191 205 221 237 255 274 294 316 340 365 392 422 453 487
do
  python benches/block.py --num ${NUM}
  RAYON_NUM_THREADS=1 cargo run -qr -- convert segmentation -i benches/block/block_${NUM}.npy -o benches/block/block_${NUM}.spn --quiet
  echo -n "${NUM}:" >> benches/compare/automesh_block.out
  for i in `seq 1 10`
  do
    start="$(date +'%s.%N')"
    RAYON_NUM_THREADS=1 cargo run -qr -- mesh hex -i benches/block/block_${NUM}.npy -o benches/compare/compare.exo --quiet
    echo -n " $(date +"%s.%N - ${start}" | bc)" >> benches/compare/automesh_block.out
  done
  echo >> benches/compare/automesh_block.out
done

rm -f benches/compare/automesh_remov.out
touch benches/compare/automesh_remov.out

for NUM in `seq 4 19`
do
  size=$(python -c 'import numpy as np; print(len(np.load("book/analysis/sphere_with_shells/spheres_resolution_'${NUM}'.npy"))**3)')
  real=$(python -c 'import numpy as np; print(int(np.sum(np.load("book/analysis/sphere_with_shells/spheres_resolution_'${NUM}'.npy") != 0)))')
  echo -n "${size}, ${real}:" >> benches/compare/automesh_remov.out
  for i in `seq 1 10`
  do
    start="$(date +'%s.%N')"
    RAYON_NUM_THREADS=1 cargo run -qr -- mesh hex -i book/analysis/sphere_with_shells/spheres_resolution_${NUM}.npy -o benches/compare/compare.exo --remove 0 --quiet
    echo -n " $(date +"%s.%N - ${start}" | bc)" >> benches/compare/automesh_remov.out
  done
  echo >> benches/compare/automesh_remov.out
done

rm -f benches/compare/sculpt_block.out
touch benches/compare/sculpt_block.out

for NUM in 100 107 115 124 133 143 154 165 178 191 205 221 237 255 274 294 316 340 365 392
do
  echo -n "${NUM}:" >> benches/compare/sculpt_block.out
  for i in `seq 1 10`
  do
    start="$(date +'%s.%N')"
    /opt/cubit/Cubit-17.02/bin/sculpt -isp benches/block/block_${NUM}.spn -x ${NUM} -y ${NUM} -z ${NUM} -str 3 -e benches/compare/compare.exo
    echo -n " $(date +"%s.%N - ${start}" | bc)" >> benches/compare/sculpt_block.out
  done
  echo >> benches/compare/sculpt_block.out
done
