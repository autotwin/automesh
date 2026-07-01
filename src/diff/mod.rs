use super::{
    ErrorWrapper,
    io::{read_segmentation, write_segmentation},
};

pub fn diff(
    input: Vec<String>,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let voxels_1 = read_segmentation(&input[0], nelx, nely, nelz, quiet, true)?;
    let voxels_2 = read_segmentation(&input[1], nelx, nely, nelz, quiet, false)?;
    write_segmentation(&output, &voxels_1.diff(&voxels_2), quiet)
}
