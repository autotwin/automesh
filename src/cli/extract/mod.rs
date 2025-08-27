use super::{ErrorWrapper, input::read_segmentation, output::write_segmentation};
use automesh::{Extraction, Remove, Scale, Translate};

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
    let mut voxels = read_segmentation(
        input,
        nelx,
        nely,
        nelz,
        Remove::default(),
        Scale::default(),
        Translate::default(),
        quiet,
        true,
    )?;
    voxels.extract(Extraction::from_input([
        xmin, xmax, ymin, ymax, zmin, zmax,
    ])?);
    write_segmentation(output, voxels, quiet)
}
