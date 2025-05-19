import matplotlib.pyplot as plt
# import numpy as np

from pathlib import Path

# Generate some data
filenames = [
    "~/scratch/ixi/IXI012-HH-1211-T1_large_s2.csv",
    "~/scratch/ixi/IXI012-HH-1211-T1_large_s3.csv",
    "~/scratch/ixi/IXI012-HH-1211-T1_large_s4.csv",
    # "res_2_iter_05.csv",
    # "res_3_iter_05.csv",
    # "res_4_iter_05.csv",
]
for filename in filenames:

    with open(Path(filename).expanduser(), "r") as file:
        data = file.read()

    aa = data.strip().split("\n")
    bb = [float(x) for x in aa]
    print(f"Number of elements: {len(bb)}")
    # n_neg = [x <= 0.0 for x in bb]

    # breakpoint()
    # Create the histogram
    plt.hist(bb, bins=20, color="blue", alpha=0.7, log=True)

    # Add labels and title
    plt.xlabel("Minimum Scaled Jacobian (MSJ)")
    plt.ylabel("Frequency")
    plt.title(f"{filename}")
    # plt.xticks(np.arange(-0.1, 1.0, 0.1))
    xt = [-0.25, 0.0, 0.25, 0.5, 0.75, 1.00]
    plt.xticks(xt)
    plt.xlim([xt[0], xt[-1]])
    plt.ylim([1, 2.0e6])

    # x_ticks = list(range(nxp))
    # y_ticks = list(range(nyp))
    # z_ticks = list(range(nzp))

    # ax.set_xlim(float(x_ticks[0]), float(x_ticks[-1]))

    # Show the plot
    # plt.show()

    # Save the plot
    fn = Path(filename).stem + "_msj" + ".png"
    plt.savefig(fn)
    print(f"Saved file: {fn}")

    # Clear the current figure
    plt.clf()
