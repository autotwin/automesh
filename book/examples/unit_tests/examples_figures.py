r"""This module, examples_figures.py, demonstrates creating a pixel slice in
the (x, y) plane, and then appending layers in the z axis, to create a 3D
voxel lattice, as a precursor for a hexahedral finite element mesh.

Example
-------
source ~/autotwin/automesh/.venv/bin/activate
pip install matplotlib
cd ~/autotwin/automesh/book/examples/unit_tests
python examples_figures.py

Ouputk
-----
The `output_npy` segmentation data files
The `output_png` visualization files
"""

# standard library
import datetime
from pathlib import Path
from typing import Final

# third-party libary
import matplotlib.pyplot as plt
from matplotlib.colors import LightSource
import numpy as np
from numpy.typing import NDArray

import examples_types as types
import examples_data as data

# Type aliases
Example = types.Example


def lattice_connectivity(ex: Example) -> NDArray[np.uint8]:
    """Given an Example, prints the lattice connectivity."""
    offset = 0
    nz, ny, nx = ex.segmentation.shape
    nzp, nyp, nxp = nz + 1, ny + 1, nx + 1

    # Generate the lattice nodes
    lattice_nodes = []

    lattice_node = 0
    for k in range(nzp):
        for j in range(nyp):
            for i in range(nxp):
                lattice_node += 1
                lattice_nodes.append([lattice_node, i, j, k])

    # connectivity for each voxel
    cvs = []

    offset = 0

    # print("processing indices...")
    for iz in range(nz):
        for iy in range(ny):
            for ix in range(nx):
                # print(f"(ix, iy, iz) = ({ix}, {iy}, {iz})")
                cv = offset + np.array(
                    [
                        (iz + 0) * (nxp * nyp) + (iy + 0) * nxp + ix + 1,
                        (iz + 0) * (nxp * nyp) + (iy + 0) * nxp + ix + 2,
                        (iz + 0) * (nxp * nyp) + (iy + 1) * nxp + ix + 2,
                        (iz + 0) * (nxp * nyp) + (iy + 1) * nxp + ix + 1,
                        (iz + 1) * (nxp * nyp) + (iy + 0) * nxp + ix + 1,
                        (iz + 1) * (nxp * nyp) + (iy + 0) * nxp + ix + 2,
                        (iz + 1) * (nxp * nyp) + (iy + 1) * nxp + ix + 2,
                        (iz + 1) * (nxp * nyp) + (iy + 1) * nxp + ix + 1,
                    ]
                )
                cvs.append(cv)

    cs = np.vstack(cvs)

    # voxel by voxel comparison
    # breakpoint()
    vv = ex.gold_lattice == cs
    assert np.all(vv)
    return cs


def mesh_lattice_connectivity(
    ex: Example,
    lattice: np.ndarray,
) -> tuple:
    """Given an Example (in particular, the Example's voxel data structure,
    a segmentation) and the `lattice_connectivity`, create the connectivity
    for the mesh with lattice node numbers.  A voxel with a segmentation id not
    in the Example's included ids tuple is excluded from the mesh.
    """

    # segmentation = ex.segmentation.flatten().squeeze()
    segmentation = ex.segmentation.flatten()

    # breakpoint()

    # assert that the list of included ids is equal
    included_set_unordered = set(ex.included_ids)
    included_list_ordered = sorted(included_set_unordered)
    # breakpoint()
    seg_set = set(segmentation)
    for item in included_list_ordered:
        assert item in seg_set, (
            f"Error: `included_ids` item {item} is not in the segmentation"
        )

    # Create a list of finite elements from the lattice elements.  If the
    # lattice element has a segmentation id that is not in the included_ids,
    # exlude the voxel element from the collected list to create the finite
    # element list
    blocks = ()  # empty tuple
    # breakpoint()
    for bb in included_list_ordered:
        # included_elements = []
        elements = ()  # empty tuple
        elements = elements + (bb,)  # insert the block number
        for i, element in enumerate(lattice):
            if bb == segmentation[i]:
                # breakpoint()
                elements = elements + (tuple(element.tolist()),)  # overwrite

        blocks = blocks + (elements,)  # overwrite

    # breakpoint()

    # return np.array(blocks)
    return blocks


def renumber(source: tuple, old: tuple, new: tuple) -> tuple:
    """Given a source tuple, composed of a list of positive integers,
    a tuple of `old` numbers that maps into `new` numbers, return the
    source tuple with the `new` numbers."""

    # the old and the new tuples musts have the same length
    err = "Tuples `old` and `new` must have equal length."
    assert len(old) == len(new), err

    result = ()
    for item in source:
        idx = old.index(item)
        new_value = new[idx]
        result = result + (new_value,)

    return result


def mesh_element_connectivity(mesh_with_lattice_connectivity: tuple):
    """Given a mesh with lattice connectivity, return a mesh with finite
    element connectivity.
    """
    # create a list of unordered lattice node numbers
    ln = []
    for item in mesh_with_lattice_connectivity:
        # print(f"item is {item}")
        # The first item is the block number
        # block = item[0]
        # The second and onward items are the elements
        elements = item[1:]
        for element in elements:
            ln += list(element)

    ln_set = set(ln)  # sets are not necessarily ordered
    ln_ordered = tuple(sorted(ln_set))  # now these unique integers are ordered

    # and they will map into the new compressed unique interger list `mapsto`
    mapsto = tuple(range(1, len(ln_ordered) + 1))

    # now build a mesh_with_element_connectivity
    mesh = ()  # empty tuple
    # breakpoint()
    for item in mesh_with_lattice_connectivity:
        # The first item is the block number
        block_number = item[0]
        block_and_elements = ()  # empty tuple
        # insert the block number
        block_and_elements = block_and_elements + (block_number,)
        for element in item[1:]:
            new_element = renumber(source=element, old=ln_ordered, new=mapsto)
            # overwrite
            block_and_elements = block_and_elements + (new_element,)

        mesh = mesh + (block_and_elements,)  # overwrite

    return mesh


def flatten_tuple(t):
    """Uses recursion to convert nested tuples into a single-sevel tuple.

    Example:
        nested_tuple = (1, (2, 3), (4, (5, 6)), 7)
        flattened_tuple = flatten_tuple(nested_tuple)
        print(flattened_tuple)  # Output: (1, 2, 3, 4, 5, 6, 7)
    """
    flat_list = []
    for item in t:
        if isinstance(item, tuple):
            flat_list.extend(flatten_tuple(item))
        else:
            flat_list.append(item)
    # breakpoint()
    return tuple(flat_list)


def elements_without_block_ids(mesh: tuple) -> tuple:
    """Given a mesh, removes the block ids and returns only just the
    element connectivities.
    """

    aa = ()
    for item in mesh:
        bb = item[1:]
        aa = aa + bb

    return aa


def main():
    """The main program."""

    # Create an instance of a specific example
    # user input begin
    examples = [
        data.Single(),
        # data.DoubleX(),
        # data.DoubleY(),
        # data.TripleX(),
        # data.QuadrupleX(),
        # data.Quadruple2VoidsX(),
        # data.Quadruple2Blocks(),
        # data.Quadruple2BlocksVoid(),
        # data.Cube(),
        # data.CubeMulti(),
        # data.CubeWithInclusion(),
        data.Bracket(),
        # data.LetterF(),
        # data.LetterF3D(),
        # data.Sparse(),
    ]

    # output_dir: Final[str] = "~/scratch"
    output_dir: Final[Path] = Path(__file__).parent
    DPI: Final[int] = 300  # resolution, dots per inch

    for ex in examples:
        # computation
        output_npy: Path = Path(output_dir).expanduser().joinpath(ex.file_stem + ".npy")

        # visualization
        SHOW: Final[bool] = True  # Post-processing visuals, show on screen
        SAVE: Final[bool] = True  # Save the .png file
        output_png_short = ex.file_stem + ".png"
        output_png: Path = Path(output_dir).expanduser().joinpath(output_png_short)
        # el, az, roll = 25, -115, 0
        # el, az, roll = 28, -115, 0
        el, az, roll = 63, -110, 0  # used for most visuals
        # el, az, roll = 11, -111, 0  # used for CubeWithInclusion
        # el, az, roll = 60, -121, 0
        # el, az, roll = 42, -120, 0
        #
        # colors
        # cmap = cm.get_cmap("viridis")  # viridis colormap
        # cmap = plt.get_cmap(name="viridis")
        cmap = plt.get_cmap(name="tab10")
        # number of discrete colors
        num_colors = len(ex.included_ids)
        colors = cmap(np.linspace(0, 1, num_colors))
        # breakpoint()
        # azimuth (deg):
        #   0 is east  (from +y-axis looking back toward origin)
        #  90 is north (from +x-axis looking back toward origin)
        # 180 is west  (from -y-axis looking back toward origin)
        # 270 is south (from -x-axis looking back toward origin)
        # elevation (deg): 0 is horizontal, 90 is vertical (+z-axis up)
        lightsource = LightSource(azdeg=325, altdeg=45)  # azimuth, elevation
        nodes_shown: bool = True
        # nodes_shown: bool = False
        voxel_alpha: float = 0.1
        # voxel_alpha: float = 0.7

        # io: if the output directory does not already exist, create it
        output_path = Path(output_dir).expanduser()
        if not output_path.exists():
            print(f"Could not find existing output directory: {output_path}")
            Path.mkdir(output_path)
            print(f"Created: {output_path}")
            assert output_path.exists()

        nelz, nely, nelx = ex.segmentation.shape
        lc = lattice_connectivity(ex=ex)

        # breakpoint()
        mesh_w_lattice_conn = mesh_lattice_connectivity(ex=ex, lattice=lc)
        err = "Calculated lattice connectivity error."
        assert mesh_w_lattice_conn == ex.gold_mesh_lattice_connectivity, err

        mesh_w_element_conn = mesh_element_connectivity(mesh_w_lattice_conn)
        err = "Calcualted element connectivity error."  # overwrite
        assert mesh_w_element_conn == ex.gold_mesh_element_connectivity, err

        # save the numpy data as a .npy file
        np.save(output_npy, ex.segmentation)
        print(f"Saved: {output_npy}")

        # to load the array back from the .npy file,
        # use the numpy.load function:
        loaded_array = np.load(output_npy)

        # verify the loaded array
        # print(f"segmentation loaded from saved file: {loaded_array}")

        assert np.all(loaded_array == ex.segmentation)

        # now that the .npy file has been created and verified,
        # move it to the repo at ~/autotwin/automesh/tests/input

        if not SHOW:
            return

        # visualization

        # Define the dimensions of the lattice
        nxp, nyp, nzp = (nelx + 1, nely + 1, nelz + 1)

        # Create a figure and a 3D axis
        # fig = plt.figure()
        fig = plt.figure(figsize=(10, 5))  # Adjust the figure size
        # fig = plt.figure(figsize=(8, 4))  # Adjust the figure size
        # ax = fig.add_subplot(111, projection="3d")
        # figure with 1 row, 2 columns
        ax = fig.add_subplot(1, 2, 1, projection="3d")  # r1, c2, 1st subplot
        ax2 = fig.add_subplot(1, 2, 2, projection="3d")  # r1, c2, 2nd subplot

        # For 3D plotting of voxels in matplotlib, we must swap the 'x' and the
        # 'z' axes.  The original axes in the segmentation are (z, y, x) and
        # are numbered (0, 1, 2).  We want new exists as (x, y, z) and thus
        # with numbering (2, 1, 0).
        vox = np.transpose(ex.segmentation, (2, 1, 0))
        # add voxels for each of the included materials
        for i, block_id in enumerate(ex.included_ids):
            # breakpoint()
            solid = vox == block_id
            # ax.voxels(solid, facecolors=voxel_color, alpha=voxel_alpha)
            # ax.voxels(solid, facecolors=colors[i], alpha=voxel_alpha)
            ax.voxels(
                solid,
                facecolors=colors[i],
                edgecolor=colors[i],
                alpha=voxel_alpha,
                lightsource=lightsource,
            )
            # plot the same voxels on the 2nd axis
            ax2.voxels(
                solid,
                facecolors=colors[i],
                edgecolor=colors[i],
                alpha=voxel_alpha,
                lightsource=lightsource,
            )

        # breakpoint()

        # Generate the lattice points
        x = []
        y = []
        z = []
        labels = []

        # Generate the element points
        xel = []
        yel = []
        zel = []
        # generate a set from the element connectivity
        # breakpoint()
        # ec_set = set(flatten_tuple(mesh_w_lattice_conn))  # bug!
        # bug fix:
        ec_set = set(flatten_tuple(elements_without_block_ids(mesh_w_lattice_conn)))

        # breakpoint()

        lattice_ijk = 0
        # gnn = global node number
        gnn = 0
        gnn_labels = []

        for k in range(nzp):
            for j in range(nyp):
                for i in range(nxp):
                    x.append(i)
                    y.append(j)
                    z.append(k)
                    if lattice_ijk + 1 in ec_set:
                        gnn += 1
                        xel.append(i)
                        yel.append(j)
                        zel.append(k)
                        gnn_labels.append(f" {gnn}")
                    lattice_ijk += 1
                    labels.append(f" {lattice_ijk}: ({i},{j},{k})")

        if nodes_shown:
            # Plot the lattice coordinates
            ax.scatter(
                x,
                y,
                z,
                s=20,
                facecolors="red",
                edgecolors="none",
            )

            # Label the lattice coordinates
            for n, label in enumerate(labels):
                ax.text(x[n], y[n], z[n], label, color="darkgray", fontsize=8)

            # Plot the nodes included in the finite element connectivity
            ax2.scatter(
                xel,
                yel,
                zel,
                s=30,
                facecolors="blue",
                edgecolors="blue",
            )

            # Label the global node numbers
            for n, label in enumerate(gnn_labels):
                ax2.text(xel[n], yel[n], zel[n], label, color="darkblue", fontsize=8)

        # Set labels for the axes
        ax.set_xlabel("x")
        ax.set_ylabel("y")
        ax.set_zlabel("z")
        # repeat for the 2nd axis
        ax2.set_xlabel("x")
        ax2.set_ylabel("y")
        ax2.set_zlabel("z")

        x_ticks = list(range(nxp))
        y_ticks = list(range(nyp))
        z_ticks = list(range(nzp))

        ax.set_xticks(x_ticks)
        ax.set_yticks(y_ticks)
        ax.set_zticks(z_ticks)
        # repeat for the 2nd axis
        ax2.set_xticks(x_ticks)
        ax2.set_yticks(y_ticks)
        ax2.set_zticks(z_ticks)

        ax.set_xlim(float(x_ticks[0]), float(x_ticks[-1]))
        ax.set_ylim(float(y_ticks[0]), float(y_ticks[-1]))
        ax.set_zlim(float(z_ticks[0]), float(z_ticks[-1]))
        # repeat for the 2nd axis
        ax2.set_xlim(float(x_ticks[0]), float(x_ticks[-1]))
        ax2.set_ylim(float(y_ticks[0]), float(y_ticks[-1]))
        ax2.set_zlim(float(z_ticks[0]), float(z_ticks[-1]))

        # Set the camera view
        ax.set_aspect("equal")
        ax.view_init(elev=el, azim=az, roll=roll)
        # repeat for the 2nd axis
        ax2.set_aspect("equal")
        ax2.view_init(elev=el, azim=az, roll=roll)

        # Adjust the distance of the camera.  The default value is 10.
        # Increasing/decreasing this value will zoom in/out, respectively.
        # ax.dist = 5  # Change the distance of the camera
        # Doesn't seem to work, and the title is clipping the uppermost node
        # and lattice numbers, so suppress the titles for now.

        # Set the title
        # ax.set_title(ex.figure_title)

        # Add a footnote
        # Get the current date and time in UTC
        now_utc = datetime.datetime.now(datetime.UTC)
        # Format the date and time as a string
        timestamp_utc = now_utc.strftime("%Y-%m-%d %H:%M:%S UTC")
        fn = f"Figure: {output_png_short} "
        fn += f"created with {__file__}\non {timestamp_utc}."
        fig.text(0.5, 0.01, fn, ha="center", fontsize=8)

        # Show the plot
        if SHOW:
            plt.show()

        if SAVE:
            # plt.show()
            fig.savefig(output_png, dpi=DPI)
            print(f"Saved: {output_png}")


if __name__ == "__main__":
    main()
