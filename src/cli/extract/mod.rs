use super::{
    ErrorWrapper,
    io::{read_segmentation, write_segmentation},
};

#[allow(clippy::too_many_arguments)]
pub fn extract(
    input: String,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    xmin: usize,
    xmax: usize,
    ymin: usize,
    ymax: usize,
    zmin: usize,
    zmax: usize,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let voxels = read_segmentation(&input, nelx, nely, nelz, quiet, true)?;
    let extracted = voxels.extract([xmin..xmax + 1, ymin..ymax + 1, zmin..zmax + 1]);
    write_segmentation(&output, &extracted, quiet)
}
