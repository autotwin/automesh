use super::{
    ErrorWrapper,
    io::{read_mesh, read_segmentation, write_mesh, write_segmentation},
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConvertSubcommand {
    /// Converts mesh file types (exo | inp | stl | vtu) -> (exo | inp | mesh | stl | vtu)
    Mesh(ConvertMeshArgs),
    /// Converts segmentation file types (npy | spn) -> (npy | spn | vti)
    Segmentation(ConvertSegmentationArgs),
}

#[derive(clap::Args)]
pub struct ConvertMeshArgs {
    /// Mesh input file (exo | inp | stl | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Mesh output file (exo | inp | mesh | stl | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,
}

#[derive(clap::Args)]
pub struct ConvertSegmentationArgs {
    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Segmentation output file (npy | spn | vti)
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
}

pub fn convert_mesh(args: ConvertMeshArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mesh = read_mesh(&args.input, quiet, true)?;
    write_mesh(&args.output, mesh, quiet)
}

pub fn convert_segmentation(
    input: String,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let voxels = read_segmentation(&input, nelx, nely, nelz, quiet, true)?;
    write_segmentation(&output, &voxels, quiet)
}
