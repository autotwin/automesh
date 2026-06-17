use super::{ErrorWrapper, input::read_voxels, output::write_voxels};

pub fn diff(
    input: Vec<String>,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let voxels_1 = read_voxels(&input[0], nelx, nely, nelz, quiet, true)?;
    let voxels_2 = read_voxels(&input[1], nelx, nely, nelz, quiet, false)?;
    write_voxels(&output, &voxels_1.diff(&voxels_2), quiet)
}
