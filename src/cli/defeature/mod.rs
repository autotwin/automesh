use super::{ErrorWrapper, input::read_segmentation, output::write_segmentation};
use automesh::{Remove, Scale, Translate};
use std::time::Instant;

pub fn defeature(
    input: String,
    output: String,
    min: usize,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
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
    let time = Instant::now();
    if !quiet {
        println!(" \x1b[1;96mDefeaturing\x1b[0m clusters of {min} voxels or less",);
    }
    voxels = voxels.defeature(min);
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    write_segmentation(output, voxels, quiet)
}
