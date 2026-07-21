"""This module creates a 7x7x7 octahedron segmentation."""

from pathlib import Path

import numpy as np

segmentation = np.array(
    [
        [  # slice 1
            [0, 0, 0, 0, 0, 0, 0],  # row 1
            [0, 0, 0, 0, 0, 0, 0],  # row 2
            [0, 0, 0, 0, 0, 0, 0],  # row 3
            [0, 0, 0, 3, 0, 0, 0],  # row 4
            [0, 0, 0, 0, 0, 0, 0],  # row 5
            [0, 0, 0, 0, 0, 0, 0],  # row 6
            [0, 0, 0, 0, 0, 0, 0],  # row 7
        ],
        [  # slice 2
            [0, 0, 0, 0, 0, 0, 0],  # row 1
            [0, 0, 0, 0, 0, 0, 0],  # row 2
            [0, 0, 0, 3, 0, 0, 0],  # row 3
            [0, 0, 3, 2, 3, 0, 0],  # row 4
            [0, 0, 0, 3, 0, 0, 0],  # row 5
            [0, 0, 0, 0, 0, 0, 0],  # row 6
            [0, 0, 0, 0, 0, 0, 0],  # row 7
        ],
        [  # slice 3
            [0, 0, 0, 0, 0, 0, 0],  # row 1
            [0, 0, 0, 3, 0, 0, 0],  # row 2
            [0, 0, 3, 2, 3, 0, 0],  # row 3
            [0, 3, 2, 1, 2, 3, 0],  # row 4
            [0, 0, 3, 2, 3, 0, 0],  # row 5
            [0, 0, 0, 3, 0, 0, 0],  # row 6
            [0, 0, 0, 0, 0, 0, 0],  # row 7
        ],
        [  # slice 4
            [0, 0, 0, 3, 0, 0, 0],  # row 1
            [0, 0, 3, 2, 3, 0, 0],  # row 2
            [0, 3, 2, 1, 2, 3, 0],  # row 3
            [3, 2, 1, 1, 1, 2, 3],  # row 4
            [0, 3, 2, 1, 2, 3, 0],  # row 5
            [0, 0, 3, 2, 3, 0, 0],  # row 6
            [0, 0, 0, 3, 0, 0, 0],  # row 7
        ],
        [  # slice 5
            [0, 0, 0, 0, 0, 0, 0],  # row 1
            [0, 0, 0, 3, 0, 0, 0],  # row 2
            [0, 0, 3, 2, 3, 0, 0],  # row 3
            [0, 3, 2, 1, 2, 3, 0],  # row 4
            [0, 0, 3, 2, 3, 0, 0],  # row 5
            [0, 0, 0, 3, 0, 0, 0],  # row 6
            [0, 0, 0, 0, 0, 0, 0],  # row 7
        ],
        [  # slice 6
            [0, 0, 0, 0, 0, 0, 0],  # row 1
            [0, 0, 0, 0, 0, 0, 0],  # row 2
            [0, 0, 0, 3, 0, 0, 0],  # row 3
            [0, 0, 3, 2, 3, 0, 0],  # row 4
            [0, 0, 0, 3, 0, 0, 0],  # row 5
            [0, 0, 0, 0, 0, 0, 0],  # row 6
            [0, 0, 0, 0, 0, 0, 0],  # row 7
        ],
        [  # slice 7
            [0, 0, 0, 0, 0, 0, 0],  # row 1
            [0, 0, 0, 0, 0, 0, 0],  # row 2
            [0, 0, 0, 0, 0, 0, 0],  # row 3
            [0, 0, 0, 3, 0, 0, 0],  # row 4
            [0, 0, 0, 0, 0, 0, 0],  # row 5
            [0, 0, 0, 0, 0, 0, 0],  # row 6
            [0, 0, 0, 0, 0, 0, 0],  # row 7
        ],
    ],
    dtype=np.uint8,
)

FILE_NAME = "minimum_working_example/octahedron.npy"
Path(FILE_NAME).parent.mkdir(parents=True, exist_ok=True)
np.save(FILE_NAME, segmentation)
print(f"Saved {FILE_NAME} with shape {segmentation.shape}.")
