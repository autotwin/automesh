use super::{ErrorWrapper, input::read_segmentation, output::write_segmentation};
use automesh::{Remove, Scale, Translate};

pub fn diff(
    input: Vec<String>,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let voxels_1 = read_segmentation(
        input[0].clone(),
        nelx,
        nely,
        nelz,
        Remove::default(),
        Scale::default(),
        Translate::default(),
        quiet,
        true,
    )?;
    let voxels_2 = read_segmentation(
        input[1].clone(),
        nelx,
        nely,
        nelz,
        Remove::default(),
        Scale::default(),
        Translate::default(),
        quiet,
        false,
    )?;
    write_segmentation(output, voxels_1.diff(&voxels_2), quiet)
}
