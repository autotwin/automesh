use automesh::{
    Extraction, FiniteElementMethods, HEX, HexahedralFiniteElements, Octree, Remove, Scale,
    Smoothing, TET, TRI, Tessellation, TetrahedralFiniteElements, Translate,
    TriangularFiniteElements,
};
use clap::{Parser, Subcommand};
use conspire::math::TensorVec;
use std::time::Instant;

mod cli;
use cli::{
    ErrorWrapper,
    convert::{ConvertSubcommand, convert_mesh, convert_segmentation},
    input::{read_finite_elements, read_segmentation},
    output::{write_finite_elements, write_segmentation},
    remesh::{MeshRemeshCommands, REMESH_DEFAULT_ITERS, remesh},
};

const TAUBIN_DEFAULT_ITERS: usize = 20;
const TAUBIN_DEFAULT_BAND: f64 = 0.1;
const TAUBIN_DEFAULT_SCALE: f64 = 0.6307;

macro_rules! about {
    () => {
        format!(
            "

     @@@@@@@@@@@@@@@@
      @@@@  @@@@@@@@@@
     @@@@  @@@@@@@@@@@
    @@@@  @@@@@@@@@@@@    \x1b[1;4m{}: Automatic mesh generation\x1b[0m
      @@    @@    @@      {}
      @@    @@    @@      {}
    @@@@@@@@@@@@  @@@
    @@@@@@@@@@@  @@@@
    @@@@@@@@@@ @@@@@ @
     @@@@@@@@@@@@@@@@",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_AUTHORS").split(":").collect::<Vec<&str>>()[0],
            env!("CARGO_PKG_AUTHORS").split(":").collect::<Vec<&str>>()[1]
        )
    };
}

#[derive(Parser)]
#[command(about = about!(), arg_required_else_help = true, version)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Converts between mesh or segmentation file types
    Convert {
        #[command(subcommand)]
        subcommand: ConvertSubcommand,
    },

    /// Defeatures and creates a new segmentation
    Defeature {
        /// Segmentation input file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Defeatured segmentation output file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Defeature clusters with less than MIN voxels
        #[arg(long, short, value_name = "MIN")]
        min: usize,

        /// Number of voxels in the x-direction (spn)
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction (spn)
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction (spn)
        #[arg(long, short = 'z', value_name = "NEL")]
        nelz: Option<usize>,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },

    /// Show the difference between two segmentations
    Diff {
        /// Segmentation input files (npy | spn)
        #[arg(long, num_args = 2, short, value_delimiter = ' ', value_name = "FILE")]
        input: Vec<String>,

        /// Segmentation difference output file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Number of voxels in the x-direction (spn)
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction (spn)
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction (spn)
        #[arg(long, short = 'z', value_name = "NEL")]
        nelz: Option<usize>,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },

    /// Extracts a specified range of voxels from a segmentation
    Extract {
        /// Segmentation input file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Extracted segmentation output file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Number of voxels in the x-direction (spn)
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction (spn)
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction (spn)
        #[arg(long, short = 'z', value_name = "NEL")]
        nelz: Option<usize>,

        /// Minimum voxel in the x-direction
        #[arg(long, value_name = "MIN")]
        xmin: usize,

        /// Maximum voxel in the x-direction
        #[arg(long, value_name = "MAX")]
        xmax: usize,

        /// Minimum voxel in the y-direction
        #[arg(long, value_name = "MIN")]
        ymin: usize,

        /// Maximum voxel in the y-direction
        #[arg(long, value_name = "MAX")]
        ymax: usize,

        /// Minimum voxel in the z-direction
        #[arg(long, value_name = "MIN")]
        zmin: usize,

        /// Maximum voxel in the z-direction
        #[arg(long, value_name = "MAX")]
        zmax: usize,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },

    /// Creates a finite element mesh from a segmentation
    Mesh {
        #[command(subcommand)]
        subcommand: MeshSubcommand,
    },

    /// Quality metrics for an existing finite element mesh
    Metrics {
        /// Mesh input file (inp | stl)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Quality metrics output file (csv | npy)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },

    /// Creates a balanced octree from a segmentation
    #[command(hide = true)]
    Octree {
        /// Segmentation input file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Octree output file (exo | inp | mesh | vtk)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Number of voxels in the x-direction (spn)
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction (spn)
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction (spn)
        #[arg(long, short = 'z', value_name = "NEL")]
        nelz: Option<usize>,

        /// Voxel IDs to remove from the mesh
        #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
        remove: Option<Vec<usize>>,

        /// Scaling (> 0.0) in the x-direction, applied before translation
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        xscale: f64,

        /// Scaling (> 0.0) in the y-direction, applied before translation
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        yscale: f64,

        /// Scaling (> 0.0) in the z-direction, applied before translation
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        zscale: f64,

        /// Translation in the x-direction
        #[arg(
            long,
            default_value_t = 0.0,
            allow_negative_numbers = true,
            value_name = "VAL"
        )]
        xtranslate: f64,

        /// Translation in the y-direction
        #[arg(
            long,
            default_value_t = 0.0,
            allow_negative_numbers = true,
            value_name = "VAL"
        )]
        ytranslate: f64,

        /// Translation in the z-direction
        #[arg(
            long,
            default_value_t = 0.0,
            allow_negative_numbers = true,
            value_name = "VAL"
        )]
        ztranslate: f64,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,

        /// Pass to apply pairing
        #[arg(action, long, short)]
        pair: bool,

        /// Pass to apply strong balancing
        #[arg(action, long, short)]
        strong: bool,
    },

    /// Applies isotropic remeshing to an existing mesh
    Remesh {
        /// Mesh input file (inp | stl)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Mesh output file (exo | mesh | stl | vtk)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Number of remeshing iterations
        #[arg(default_value_t = REMESH_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },

    /// Applies smoothing to an existing mesh
    Smooth {
        #[command(subcommand)]
        subcommand: SmoothSubcommand,
    },
}

#[derive(Subcommand)]
enum MeshSubcommand {
    /// Creates an all-hexahedral mesh from a segmentation
    Hex(MeshHexArgs),
    /// Creates an all-tetrahedral mesh from a segmentation
    Tet(MeshTetArgs),
    /// Creates all-triangular isosurface(s) from a segmentation
    Tri(MeshTriArgs),
}

#[derive(clap::Args)]
struct MeshHexArgs {
    #[command(subcommand)]
    smoothing: Option<MeshSmoothCommands>,

    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Defeature clusters with less than NUM voxels
    #[arg(long, short, value_name = "NUM")]
    defeature: Option<usize>,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    nelz: Option<usize>,

    /// Voxel IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    remove: Option<Vec<usize>>,

    /// Scaling (> 0.0) in the x-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    xscale: f64,

    /// Scaling (> 0.0) in the y-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    yscale: f64,

    /// Scaling (> 0.0) in the z-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    zscale: f64,

    /// Translation in the x-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    xtranslate: f64,

    /// Translation in the y-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    ytranslate: f64,

    /// Translation in the z-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    ztranslate: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,

    /// Pass to mesh adaptively
    #[arg(action, hide = true, long)]
    adapt: bool,
}

#[derive(clap::Args)]
struct MeshTetArgs {
    #[command(subcommand)]
    smoothing: Option<MeshSmoothCommands>,

    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Defeature clusters with less than NUM voxels
    #[arg(long, short, value_name = "NUM")]
    defeature: Option<usize>,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    nelz: Option<usize>,

    /// Voxel IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    remove: Option<Vec<usize>>,

    /// Scaling (> 0.0) in the x-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    xscale: f64,

    /// Scaling (> 0.0) in the y-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    yscale: f64,

    /// Scaling (> 0.0) in the z-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    zscale: f64,

    /// Translation in the x-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    xtranslate: f64,

    /// Translation in the y-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    ytranslate: f64,

    /// Translation in the z-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    ztranslate: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,

    /// Pass to mesh adaptively
    #[arg(action, hide = true, long)]
    adapt: bool,
}

#[derive(clap::Args)]
struct MeshTriArgs {
    #[command(subcommand)]
    smoothing: Option<MeshSmoothCommands>,

    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Mesh output file (exo | inp | mesh | stl | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Defeature clusters with less than NUM voxels
    #[arg(long, short, value_name = "NUM")]
    defeature: Option<usize>,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    nelz: Option<usize>,

    /// Voxel IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    remove: Option<Vec<usize>>,

    /// Scaling (> 0.0) in the x-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    xscale: f64,

    /// Scaling (> 0.0) in the y-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    yscale: f64,

    /// Scaling (> 0.0) in the z-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    zscale: f64,

    /// Translation in the x-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    xtranslate: f64,

    /// Translation in the y-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    ytranslate: f64,

    /// Translation in the z-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    ztranslate: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,

    /// Pass to mesh adaptively
    #[arg(action, hide = true, long)]
    adapt: bool,
}

#[derive(Subcommand, Debug)]
enum MeshSmoothCommands {
    /// Applies smoothing to the mesh before output
    Smooth {
        #[command(subcommand)]
        remeshing: Option<MeshRemeshCommands>,

        /// Pass to enable hierarchical control
        #[arg(action, long, short = 'c')]
        hierarchical: bool,

        /// Number of smoothing iterations
        #[arg(default_value_t = TAUBIN_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Smoothing method (Laplace | Taubin) [default: Taubin]
        #[arg(long, short, value_name = "NAME")]
        method: Option<String>,

        /// Pass-band frequency (for Taubin only)
        #[arg(default_value_t = TAUBIN_DEFAULT_BAND, long, short = 'k', value_name = "FREQ")]
        pass_band: f64,

        /// Scaling parameter for all smoothing methods
        #[arg(default_value_t = TAUBIN_DEFAULT_SCALE, long, short, value_name = "SCALE")]
        scale: f64,
    },
}

#[derive(Subcommand)]
enum SmoothSubcommand {
    /// Smooths an all-hexahedral mesh
    Hex(SmoothHexArgs),
    /// Smooths an all-tetrahedral mesh
    Tet(SmoothTetArgs),
    /// Smooths an all-triangular mesh
    Tri(SmoothTriArgs),
}

#[derive(clap::Args)]
struct SmoothHexArgs {
    /// Pass to enable hierarchical control
    #[arg(action, long, short = 'c')]
    hierarchical: bool,

    /// Mesh input file (inp)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Smoothed mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Number of smoothing iterations
    #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
    iterations: usize,

    /// Smoothing method (Laplace | Taubin) [default: Taubin]
    #[arg(long, short, value_name = "NAME")]
    method: Option<String>,

    /// Pass-band frequency (for Taubin only)
    #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
    pass_band: f64,

    /// Scaling parameter for all smoothing methods
    #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
    scale: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,
}

#[derive(clap::Args)]
struct SmoothTetArgs {
    /// Pass to enable hierarchical control
    #[arg(action, long, short = 'c')]
    hierarchical: bool,

    /// Mesh input file (inp)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Smoothed mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Number of smoothing iterations
    #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
    iterations: usize,

    /// Smoothing method (Laplace | Taubin) [default: Taubin]
    #[arg(long, short, value_name = "NAME")]
    method: Option<String>,

    /// Pass-band frequency (for Taubin only)
    #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
    pass_band: f64,

    /// Scaling parameter for all smoothing methods
    #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
    scale: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,
}

#[derive(clap::Args)]
struct SmoothTriArgs {
    #[command(subcommand)]
    remeshing: Option<MeshRemeshCommands>,

    /// Pass to enable hierarchical control
    #[arg(action, long, short = 'c')]
    hierarchical: bool,

    /// Mesh input file (stl); #TODO: the (inp) file type is a work in progress
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Smoothed mesh output file (exo | inp | mesh | stl | vtk)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Number of smoothing iterations
    #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
    iterations: usize,

    /// Smoothing method (Laplace | Taubin) [default: Taubin]
    #[arg(long, short, value_name = "NAME")]
    method: Option<String>,

    /// Pass-band frequency (for Taubin only)
    #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
    pass_band: f64,

    /// Scaling parameter for all smoothing methods
    #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
    scale: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,
}

fn main() -> Result<(), ErrorWrapper> {
    let time = Instant::now();
    let is_quiet;
    let args = Args::parse();
    let result = match args.command {
        Some(Commands::Convert { subcommand }) => match subcommand {
            ConvertSubcommand::Mesh(args) => {
                is_quiet = args.quiet;
                convert_mesh(args.input, args.output, args.quiet)
            }
            ConvertSubcommand::Segmentation(args) => {
                is_quiet = args.quiet;
                convert_segmentation(
                    args.input,
                    args.output,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.quiet,
                )
            }
        },
        Some(Commands::Defeature {
            input,
            output,
            min,
            nelx,
            nely,
            nelz,
            quiet,
        }) => {
            is_quiet = quiet;
            defeature(input, output, min, nelx, nely, nelz, quiet)
        }
        Some(Commands::Diff {
            input,
            output,
            nelx,
            nely,
            nelz,
            quiet,
        }) => {
            is_quiet = quiet;
            diff(input, output, nelx, nely, nelz, quiet)
        }
        Some(Commands::Extract {
            input,
            output,
            nelx,
            nely,
            nelz,
            xmin,
            xmax,
            ymin,
            ymax,
            zmin,
            zmax,
            quiet,
        }) => {
            is_quiet = quiet;
            extract(
                input, output, nelx, nely, nelz, xmin, xmax, ymin, ymax, zmin, zmax, quiet,
            )
        }
        Some(Commands::Mesh { subcommand }) => match subcommand {
            MeshSubcommand::Hex(args) => {
                is_quiet = args.quiet;
                mesh::<HEX>(
                    args.smoothing,
                    args.input,
                    args.output,
                    args.defeature,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.remove,
                    args.xscale,
                    args.yscale,
                    args.zscale,
                    args.xtranslate,
                    args.ytranslate,
                    args.ztranslate,
                    args.metrics,
                    args.quiet,
                    args.adapt,
                )
            }
            MeshSubcommand::Tet(args) => {
                is_quiet = args.quiet;
                mesh::<TET>(
                    args.smoothing,
                    args.input,
                    args.output,
                    args.defeature,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.remove,
                    args.xscale,
                    args.yscale,
                    args.zscale,
                    args.xtranslate,
                    args.ytranslate,
                    args.ztranslate,
                    args.metrics,
                    args.quiet,
                    args.adapt,
                )
            }
            MeshSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                mesh::<TRI>(
                    args.smoothing,
                    args.input,
                    args.output,
                    args.defeature,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.remove,
                    args.xscale,
                    args.yscale,
                    args.zscale,
                    args.xtranslate,
                    args.ytranslate,
                    args.ztranslate,
                    args.metrics,
                    args.quiet,
                    args.adapt,
                )
            }
        },
        Some(Commands::Metrics {
            input,
            output,
            quiet,
        }) => {
            is_quiet = quiet;
            write_metrics(
                &read_finite_elements::<_, TriangularFiniteElements>(&input, quiet, true)?,
                output,
                quiet,
            )
        }
        Some(Commands::Octree {
            input,
            output,
            nelx,
            nely,
            nelz,
            remove,
            xscale,
            yscale,
            zscale,
            xtranslate,
            ytranslate,
            ztranslate,
            quiet,
            pair,
            strong,
        }) => {
            is_quiet = quiet;
            octree(
                input, output, nelx, nely, nelz, remove, xscale, yscale, zscale, xtranslate,
                ytranslate, ztranslate, quiet, pair, strong,
            )
        }
        Some(Commands::Remesh {
            input,
            output,
            iterations,
            quiet,
        }) => {
            is_quiet = quiet;
            remesh(input, output, iterations, quiet)
        }
        Some(Commands::Smooth { subcommand }) => match subcommand {
            SmoothSubcommand::Hex(args) => {
                is_quiet = args.quiet;
                smooth::<_, HexahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.iterations,
                    args.method,
                    args.hierarchical,
                    args.pass_band,
                    args.scale,
                    args.metrics,
                    args.quiet,
                )
            }
            SmoothSubcommand::Tet(args) => {
                is_quiet = args.quiet;
                smooth::<_, TetrahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.iterations,
                    args.method,
                    args.hierarchical,
                    args.pass_band,
                    args.scale,
                    args.metrics,
                    args.quiet,
                )
            }
            SmoothSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                smooth::<_, TriangularFiniteElements>(
                    args.input,
                    args.output,
                    args.iterations,
                    args.method,
                    args.hierarchical,
                    args.pass_band,
                    args.scale,
                    args.metrics,
                    args.quiet,
                )
            }
        },
        None => return Ok(()),
    };
    if !is_quiet {
        println!("       \x1b[1;98mTotal\x1b[0m {:?}", time.elapsed());
    }
    result
}

fn defeature(
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

fn diff(
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

#[allow(clippy::too_many_arguments)]
fn extract(
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

enum MeshBasis {
    Leaves,
    Surfaces,
    Voxels,
}

#[allow(clippy::too_many_arguments)]
fn mesh<const N: usize>(
    smoothing: Option<MeshSmoothCommands>,
    input: String,
    output: String,
    defeature: Option<usize>,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    remove: Option<Vec<usize>>,
    xscale: f64,
    yscale: f64,
    zscale: f64,
    xtranslate: f64,
    ytranslate: f64,
    ztranslate: f64,
    metrics: Option<String>,
    quiet: bool,
    adapt: bool,
) -> Result<(), ErrorWrapper> {
    let mut time = Instant::now();
    let scale_temporary = Scale::from([xscale, yscale, zscale]);
    let translate_temporary = Translate::from([xtranslate, ytranslate, ztranslate]);
    let remove = Remove::from(remove);
    let scale = Scale::from([xscale, yscale, zscale]);
    let translate = Translate::from([xtranslate, ytranslate, ztranslate]);
    let mut input_type = read_segmentation(
        input, nelx, nely, nelz, remove, scale, translate, quiet, true,
    )?;
    match N {
        HEX => {
            if let Some(min_num_voxels) = defeature {
                if !quiet {
                    time = Instant::now();
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less",
                    );
                }
                input_type = input_type.defeature(min_num_voxels);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
            }
            if !quiet {
                time = Instant::now();
            }
            let mut output_type: HexahedralFiniteElements = if adapt {
                if !quiet {
                    print!("     \x1b[1;96mMeshing\x1b[0m adaptive hexahedra");
                    mesh_print_info(MeshBasis::Voxels, &scale_temporary, &translate_temporary)
                }
                let mut tree = Octree::from(input_type);
                tree.balance_and_pair(true);
                tree.into()
            } else {
                if !quiet {
                    print!("     \x1b[1;96mMeshing\x1b[0m voxels into hexahedra");
                    mesh_print_info(MeshBasis::Voxels, &scale_temporary, &translate_temporary)
                }
                input_type.into()
            };
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    output_type.get_nodal_coordinates().len()
                );
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
                        remeshing: _,
                        iterations,
                        method,
                        hierarchical,
                        pass_band,
                        scale,
                    } => {
                        apply_smoothing_method(
                            &mut output_type,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&output_type, file, quiet)?
            }
            write_finite_elements(output, output_type, quiet)?;
        }
        TET => {
            if let Some(min_num_voxels) = defeature {
                if !quiet {
                    time = Instant::now();
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less"
                    );
                }
                input_type = input_type.defeature(min_num_voxels);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
            }
            if !quiet {
                time = Instant::now();
            }
            let mut output_type: TetrahedralFiniteElements = if adapt {
                Err("Adaptive tetrahedra not yet implemented".to_string())?
            } else {
                if !quiet {
                    print!("     \x1b[1;96mMeshing\x1b[0m voxels into tetrahedra");
                    mesh_print_info(MeshBasis::Voxels, &scale_temporary, &translate_temporary)
                }
                input_type.into()
            };
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    output_type.get_nodal_coordinates().len()
                );
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
                        remeshing: _,
                        iterations,
                        method,
                        hierarchical,
                        pass_band,
                        scale,
                    } => {
                        apply_smoothing_method(
                            &mut output_type,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&output_type, file, quiet)?
            }
            write_finite_elements(output, output_type, quiet)?;
        }
        TRI => {
            if !quiet {
                time = Instant::now();
                if let Some(min_num_voxels) = defeature {
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less",
                    );
                } else {
                    mesh_print_info(MeshBasis::Surfaces, &scale_temporary, &translate_temporary)
                }
            }
            let mut tree = Octree::from(input_type);
            tree.balance(true);
            if let Some(min_num_voxels) = defeature {
                tree.defeature(min_num_voxels);
                println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                time = Instant::now();
                mesh_print_info(MeshBasis::Surfaces, &scale_temporary, &translate_temporary)
            }
            let mut output_type = TriangularFiniteElements::from(tree);
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    output_type.get_nodal_coordinates().len()
                );
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
                        remeshing,
                        iterations,
                        method,
                        hierarchical,
                        pass_band,
                        scale,
                    } => {
                        apply_smoothing_method(
                            &mut output_type,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                        if let Some(MeshRemeshCommands::Remesh { iterations, quiet }) = remeshing {
                            let time = Instant::now();
                            if !quiet {
                                println!(
                                    "   \x1b[1;96mRemeshing\x1b[0m isotropically with {iterations} iterations"
                                )
                            }
                            output_type.remesh(
                                iterations,
                                &Smoothing::Taubin(
                                    TAUBIN_DEFAULT_ITERS,
                                    TAUBIN_DEFAULT_BAND,
                                    TAUBIN_DEFAULT_SCALE,
                                ),
                            );
                            if !quiet {
                                println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed())
                            }
                        }
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&output_type, file, quiet)?
            }
            write_finite_elements(output, output_type, quiet)?;
        }
        _ => panic!(),
    }
    Ok(())
}

fn mesh_print_info(basis: MeshBasis, scale: &Scale, translate: &Translate) {
    match basis {
        MeshBasis::Leaves => {
            print!("     \x1b[1;96mMeshing\x1b[0m leaves into hexahedra")
        }
        MeshBasis::Surfaces => {
            print!("     \x1b[1;96mMeshing\x1b[0m internal surfaces")
        }
        MeshBasis::Voxels => {}
    }
    if scale != &Default::default() || translate != &Default::default() {
        print!(" \x1b[2m[");
        if scale.x() != &1.0 {
            print!("xscale: {}, ", scale.x())
        }
        if scale.y() != &1.0 {
            print!("yscale: {}, ", scale.y())
        }
        if scale.z() != &1.0 {
            print!("zscale: {}, ", scale.z())
        }
        if translate.x() != &0.0 {
            print!("xtranslate: {}, ", translate.x())
        }
        if translate.y() != &0.0 {
            print!("ytranslate: {}, ", translate.y())
        }
        if translate.z() != &0.0 {
            print!("ztranslate: {}, ", translate.z())
        }
        println!("\x1b[2D]\x1b[0m")
    } else {
        println!()
    }
}

#[allow(clippy::too_many_arguments)]
fn octree(
    input: String,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    remove: Option<Vec<usize>>,
    xscale: f64,
    yscale: f64,
    zscale: f64,
    xtranslate: f64,
    ytranslate: f64,
    ztranslate: f64,
    quiet: bool,
    pair: bool,
    strong: bool,
) -> Result<(), ErrorWrapper> {
    let remove_temporary = remove
        .clone()
        .map(|removal| removal.iter().map(|&entry| entry as u8).collect());
    let scale_temporary = Scale::from([xscale, yscale, zscale]);
    let translate_temporary = Translate::from([xtranslate, ytranslate, ztranslate]);
    let scale = [xscale, yscale, zscale].into();
    let translate = [xtranslate, ytranslate, ztranslate].into();
    let remove = Remove::from(remove);
    let input_type = read_segmentation(
        input, nelx, nely, nelz, remove, scale, translate, quiet, true,
    )?;
    let time = Instant::now();
    if !quiet {
        mesh_print_info(MeshBasis::Leaves, &scale_temporary, &translate_temporary)
    }
    let mut tree = Octree::from(input_type);
    tree.balance(strong);
    if pair {
        tree.balance_and_pair(true);
    } else {
        tree.balance(strong);
    }
    tree.prune();
    let output_type =
        tree.octree_into_finite_elements(remove_temporary, scale_temporary, translate_temporary)?;
    if !quiet {
        let mut blocks = output_type.get_element_blocks().clone();
        let elements = blocks.len();
        blocks.sort();
        blocks.dedup();
        println!(
            "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
            time.elapsed(),
            blocks.len(),
            elements,
            output_type.get_nodal_coordinates().len()
        );
    }
    write_finite_elements(output, output_type, quiet)
}

#[allow(clippy::too_many_arguments)]
fn smooth<const N: usize, T>(
    input: String,
    output: String,
    iterations: usize,
    method: Option<String>,
    hierarchical: bool,
    pass_band: f64,
    scale: f64,
    metrics: Option<String>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N> + From<Tessellation>,
    Tessellation: From<T>,
{
    let mut finite_elements = read_finite_elements(&input, quiet, true)?;
    apply_smoothing_method(
        &mut finite_elements,
        iterations,
        method,
        hierarchical,
        pass_band,
        scale,
        quiet,
    )?;
    if let Some(file) = metrics {
        write_metrics(&finite_elements, file, quiet)?
    }
    write_finite_elements(output, finite_elements, quiet)
}

#[allow(clippy::too_many_arguments)]
fn apply_smoothing_method<const N: usize, T>(
    output_type: &mut T,
    iterations: usize,
    method: Option<String>,
    hierarchical: bool,
    pass_band: f64,
    scale: f64,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let time_smooth = Instant::now();
    let smoothing_method = method.unwrap_or("Taubin".to_string());
    if matches!(
        smoothing_method.as_str(),
        "Laplacian" | "Laplace" | "laplacian" | "laplace" | "Taubin" | "taubin"
    ) {
        if !quiet {
            print!("   \x1b[1;96mSmoothing\x1b[0m ");
            match smoothing_method.as_str() {
                "Laplacian" | "Laplace" | "laplacian" | "laplace" => {
                    println!("with {iterations} iterations of Laplace")
                }
                "Taubin" | "taubin" => {
                    println!("with {iterations} iterations of Taubin")
                }
                _ => panic!(),
            }
        }
        output_type.node_element_connectivity()?;
        output_type.node_node_connectivity()?;
        if hierarchical {
            output_type.nodal_hierarchy()?;
        }
        output_type.nodal_influencers();
        match smoothing_method.as_str() {
            "Laplacian" | "Laplace" | "laplacian" | "laplace" => {
                output_type.smooth(&Smoothing::Laplacian(iterations, scale))?;
            }
            "Taubin" | "taubin" => {
                output_type.smooth(&Smoothing::Taubin(iterations, pass_band, scale))?;
            }
            _ => panic!(),
        }
        if !quiet {
            println!("        \x1b[1;92mDone\x1b[0m {:?}", time_smooth.elapsed());
        }
        Ok(())
    } else {
        Err(format!(
            "Invalid smoothing method {smoothing_method} specified",
        ))?
    }
}

fn write_metrics<const N: usize, T>(
    fem: &T,
    output: String,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let time = Instant::now();
    if !quiet {
        println!("     \x1b[1;96mMetrics\x1b[0m {output}");
    }
    fem.write_metrics(&output)?;
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}
