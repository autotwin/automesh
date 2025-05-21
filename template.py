import numpy as np
data = np.ones((6, 6, 6), dtype="uint8")
data[3, 2:4, 2:4] = 2
data[0:2, 2:4, 2:4] = 3
data[2:4, 0:2, 2:4] = 4
data[4:6, 2:4, 2:4] = 5
data[2:4, 4:6, 2:4] = 6
data[2:4, 2:4, 0:2] = 7
data[2:4, 2:4, 4:6] = 8
np.save('template.npy', data)