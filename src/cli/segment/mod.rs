use super::{ErrorWrapper, input::read_finite_elements, output::{invalid_output, write_finite_elements, write_segmentation}};
use automesh::{
    FiniteElementMethods, Tessellation, Voxels,
};
use clap::Subcommand;
use std::{path::Path, time::Instant};

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

    /// Segmentation (npy | spn) or mesh (exo | inp) output file
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Number of subdivision levels
    #[arg(long, short = 'n', value_name = "NUM")]
    pub levels: usize,

    /// Block IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    pub remove: Option<Vec<usize>>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

pub fn segment<const N: usize, T, const M: usize, U>(
    input: String,
    output: String,
    levels: usize,
    remove: Option<Vec<usize>>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N> + From<Tessellation>,
    U: FiniteElementMethods<M> + From<Voxels>,
    Tessellation: From<U>,
{
    let finite_elements = read_finite_elements::<_, T>(&input, quiet, true)?;
    let time = Instant::now();
    if !quiet {
        println!("  \x1b[1;96mSegmenting\x1b[0m from finite elements")
    }
    let mut voxels = Voxels::from_finite_elements(finite_elements, levels);
    voxels.extend_removal(remove.into());
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    let extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match extension {
        Some("exo") | Some("inp") => write_finite_elements(output, U::from(voxels), quiet),
        Some("npy") | Some("spn") => write_segmentation(output, voxels, quiet),
        _ => Err(invalid_output(&output, extension)),
    }
}
