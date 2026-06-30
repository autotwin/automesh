use super::{
    ErrorWrapper,
    io::{read_segmentation, write_segmentation},
};
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub fn defeature(
    input: String,
    output: String,
    min: usize,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let voxels = read_segmentation(&input, nelx, nely, nelz, quiet, true)?;
    let time = Instant::now();
    crate::echo!(
        quiet,
        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min} voxels or less"
    );
    let voxels = voxels.defeature(min);
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    write_segmentation(&output, &voxels, quiet)
}
