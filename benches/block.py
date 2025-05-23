from argparse import ArgumentParser
from numpy import ones, save, uint8
from PIL import Image


parser = ArgumentParser()
parser.add_argument("--num", type=int)
num = parser.parse_args().num

data = ones((num, num, num), dtype=uint8)
save(f"benches/block/block_{num}.npy", data)

for k in range(num):
    Image.fromarray(data[:, :, k]).save(f"benches/block/block_{num}_{k}.tif")
