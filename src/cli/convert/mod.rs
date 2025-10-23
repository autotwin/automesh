use super::{
    super::{
        HexahedralFiniteElements, Remove, Scale, TetrahedralFiniteElements, Translate,
        TriangularFiniteElements,
    },
    ErrorWrapper,
    input::{read_finite_elements, read_segmentation},
    output::{write_finite_elements, write_segmentation},
};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConvertSubcommand {
    /// Converts mesh file types (exo | inp | stl) -> (exo | mesh | stl | vtk)
    Mesh(ConvertMeshArgs),
    /// Converts segmentation file types (npy | spn) -> (npy | spn)
    Segmentation(ConvertSegmentationArgs),
}

#[derive(clap::Args)]
pub struct ConvertMeshArgs {
    #[command(subcommand)]
    pub subcommand: ConvertMeshSubcommand,
}

#[derive(Subcommand)]
pub enum ConvertMeshSubcommand {
    /// Converts an all-hexahedral mesh
    Hex(ConvertMeshSubcommandArgs),
    /// Converts an all-tetrahedral mesh
    Tet(ConvertMeshSubcommandArgs),
    /// Converts an all-triangular mesh
    Tri(ConvertMeshSubcommandArgs),
}

impl ConvertMeshSubcommand {
    pub fn is_quiet(&self) -> bool {
        match self {
            ConvertMeshSubcommand::Hex(args) => args.quiet,
            ConvertMeshSubcommand::Tet(args) => args.quiet,
            ConvertMeshSubcommand::Tri(args) => args.quiet,
        }
    }
}

#[derive(clap::Args)]
pub struct ConvertMeshSubcommandArgs {
    /// Mesh input file (exo | inp | stl)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Mesh output file (exo | mesh | stl | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,
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

pub fn convert_mesh(subcommand: ConvertMeshSubcommand) -> Result<(), ErrorWrapper> {
    match subcommand {
        ConvertMeshSubcommand::Hex(args) => write_finite_elements(
            args.output,
            read_finite_elements::<_, _, HexahedralFiniteElements>(&args.input, args.quiet, true)?,
            args.quiet,
        ),
        ConvertMeshSubcommand::Tet(args) => write_finite_elements(
            args.output,
            read_finite_elements::<_, _, TetrahedralFiniteElements>(&args.input, args.quiet, true)?,
            args.quiet,
        ),
        ConvertMeshSubcommand::Tri(args) => write_finite_elements(
            args.output,
            read_finite_elements::<_, _, TriangularFiniteElements>(&args.input, args.quiet, true)?,
            args.quiet,
        ),
    }
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
