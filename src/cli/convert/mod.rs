use super::{
    super::{Remove, Scale, Translate, TriangularFiniteElements},
    ErrorWrapper,
    input::{read_finite_elements, read_segmentation},
    output::{write_finite_elements, write_segmentation},
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConvertSubcommand {
    /// Converts mesh file types (inp | stl) -> (exo | mesh | stl | vtk)
    Mesh(ConvertMeshArgs),
    /// Converts segmentation file types (npy | spn) -> (npy | spn)
    Segmentation(ConvertSegmentationArgs),
}

#[derive(clap::Args)]
pub struct ConvertMeshArgs {
    /// Mesh input file (inp | stl)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Mesh output file (exo | mesh | stl | vtk)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

#[derive(clap::Args)]
pub struct ConvertSegmentationArgs {
    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Segmentation output file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    pub nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    pub nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    pub nelz: Option<usize>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

pub fn convert_mesh(input: String, output: String, quiet: bool) -> Result<(), ErrorWrapper> {
    write_finite_elements(
        output,
        read_finite_elements::<_, TriangularFiniteElements>(&input, quiet, true)?,
        quiet,
    )
}

pub fn convert_segmentation(
    input: String,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    write_segmentation(
        output,
        read_segmentation(
            input,
            nelx,
            nely,
            nelz,
            Remove::default(),
            Scale::default(),
            Translate::default(),
            quiet,
            true,
        )?,
        quiet,
    )
}
