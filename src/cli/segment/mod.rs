use super::{ErrorWrapper, input::read_finite_elements, output::write_segmentation};
use automesh::{
    FiniteElementMethods, Tessellation, Voxels,
};
use clap::Subcommand;
use std::time::Instant;

#[derive(Subcommand)]
pub enum SegmentSubcommand {
    /// Segments an all-hexahedral mesh
    Hex(SegmentArgs),
    /// Segments an all-hexahedral mesh
    Tet(SegmentArgs),
    /// Segments an all-hexahedral mesh
    Tri(SegmentArgs),
}

#[derive(clap::Args)]
pub struct SegmentArgs {
    /// Mesh input file (exo | inp)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Segmentation output file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Number of subdivision levels
    #[arg(long, short = 'n', value_name = "NUM")]
    pub levels: usize,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

pub fn segment<const N: usize, T>(
    input: String,
    output: String,
    levels: usize,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N> + From<Tessellation>,
    Tessellation: From<T>,
{
    let finite_elements = read_finite_elements::<_, T>(&input, quiet, true)?;
    let time = Instant::now();
    if !quiet {
        println!("  \x1b[1;96mSegmenting\x1b[0m from finite elements")
    }
    let segmentation = Voxels::from_finite_elements(finite_elements, levels);
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    write_segmentation(output, segmentation, quiet)
}
