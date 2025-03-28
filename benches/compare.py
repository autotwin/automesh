import matplotlib.pyplot as plt
import numpy as np


numb_a = ()
data_a = ()

with open('benches/compare/automesh_block.out', 'r') as file:
    for line in file:
        input = line.strip().split(sep=': ')
        numb_a += (int(input[0])**3,)
        count = 0
        sum = 0
        for entry in input[1].split():
            count += 1
            sum += float(entry)
        data_a += (sum / count,)

np.savetxt('benches/compare/automesh.csv', np.vstack((numb_a, data_a)).T)

numb_s = ()
data_s = ()

with open('benches/compare/sculpt_block.out', 'r') as file:
    for line in file:
        input = line.strip().split(sep=': ')
        numb_s += (int(input[0])**3,)
        count = 0
        sum = 0
        for entry in input[1].split():
            count += 1
            sum += float(entry)
        data_s += (sum / count,)

np.savetxt('benches/compare/sculpt.csv', np.vstack((numb_s, data_s)).T)

plt.loglog(numb_a, data_a, label='automesh')
plt.loglog(numb_s, data_s, label='SCULPT')
plt.xlabel('Voxels')
plt.ylabel('Time [seconds]')
plt.legend()
plt.show()
