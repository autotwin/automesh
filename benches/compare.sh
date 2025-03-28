#!/bin/bash

mkdir -p benches/block/ benches/compare/

rm -f benches/compare/automesh_block.out
touch benches/compare/automesh_block.out

for NUM in 100 107 115 124 133 143 154 165 178 191 205 221 237 255 274 294 316 340 365 392 422 453 487
do
  python benches/block.py --num ${NUM}
  cargo run -qr -- convert -i benches/block/block_${NUM}.npy -o benches/block/block_${NUM}.spn --quiet
  echo -n "${NUM}:" >> benches/compare/automesh_block.out
  for i in `seq 1 10`
  do
    start="$(date +'%s.%N')"
    cargo run -qr -- mesh -i benches/block/block_${NUM}.npy -o benches/compare/compare.exo --quiet
    echo -n " $(date +"%s.%N - ${start}" | bc)" >> benches/compare/automesh_block.out
  done
  echo >> benches/compare/automesh_block.out
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
