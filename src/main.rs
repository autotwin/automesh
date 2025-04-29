use automesh::{
    Blocks, Extraction, FiniteElementMethods, FiniteElementSpecifics, HexahedralFiniteElements,
    IntoFiniteElements, Nel, Octree, Scale, Smoothing, Tessellation, Translate, Tree,
    TriangularFiniteElements, Voxels, HEX, TRI,
};
use clap::{Parser, Subcommand};
use conspire::math::TensorVec;
use ndarray_npy::{ReadNpyError, WriteNpyError};
use netcdf::Error as ErrorNetCDF;
use std::{io::Error as ErrorIO, path::Path, time::Instant};
use vtkio::Error as ErrorVtk;

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

        /// Number of voxels in the x-direction
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction
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

        /// Extractd segmentation output file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Number of voxels in the x-direction
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction
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

        /// Number of voxels in the x-direction
        #[arg(long, short = 'x', value_name = "NEL")]
        nelx: Option<usize>,

        /// Number of voxels in the y-direction
        #[arg(long, short = 'y', value_name = "NEL")]
        nely: Option<usize>,

        /// Number of voxels in the z-direction
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

    /// Applies smoothing to an existing mesh
    Smooth {
        #[command(subcommand)]
        subcommand: SmoothSubcommand,
    },
}

#[derive(Subcommand)]
enum ConvertSubcommand {
    /// Converts mesh file types (inp | stl) -> (exo | mesh | stl | vtk)
    Mesh(ConvertMeshArgs),
    /// Converts segmentation file types (npy | spn) -> (npy | spn)
    Segmentation(ConvertSegmentationArgs),
}

#[derive(clap::Args)]
struct ConvertMeshArgs {
    /// Mesh input file (inp | stl)
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
struct ConvertSegmentationArgs {
    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    input: String,

    /// Segmentation output file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    output: String,

    /// Number of voxels in the x-direction
    #[arg(long, short = 'x', value_name = "NEL")]
    nelx: Option<usize>,

    /// Number of voxels in the y-direction
    #[arg(long, short = 'y', value_name = "NEL")]
    nely: Option<usize>,

    /// Number of voxels in the z-direction
    #[arg(long, short = 'z', value_name = "NEL")]
    nelz: Option<usize>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    quiet: bool,
}

#[derive(Subcommand)]
enum MeshSubcommand {
    /// Creates an all-hexahedral mesh from a segmentation
    Hex(MeshHexArgs),
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

    /// Number of voxels in the x-direction
    #[arg(long, short = 'x', value_name = "NEL")]
    nelx: Option<usize>,

    /// Number of voxels in the y-direction
    #[arg(long, short = 'y', value_name = "NEL")]
    nely: Option<usize>,

    /// Number of voxels in the z-direction
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

    /// Pass to mesh using dualization
    #[arg(action, hide = true, long)]
    dual: bool,
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

    /// Number of voxels in the x-direction
    #[arg(long, short = 'x', value_name = "NEL")]
    nelx: Option<usize>,

    /// Number of voxels in the y-direction
    #[arg(long, short = 'y', value_name = "NEL")]
    nely: Option<usize>,

    /// Number of voxels in the z-direction
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

    /// Pass to mesh using dualization
    #[arg(action, hide = true, long)]
    dual: bool,
}

#[derive(Subcommand, Debug)]
enum MeshSmoothCommands {
    /// Applies smoothing to the mesh before output
    Smooth {
        /// Pass to enable hierarchical control
        #[arg(action, long, short = 'c')]
        hierarchical: bool,

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
    },
}

#[derive(Subcommand)]
enum SmoothSubcommand {
    /// Smooths an all-hexahedral mesh
    Hex(SmoothHexArgs),
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
struct SmoothTriArgs {
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

struct ErrorWrapper {
    message: String,
}

impl std::fmt::Debug for ErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[1;91m{}.\x1b[0m", self.message)
    }
}

impl From<ErrorIO> for ErrorWrapper {
    fn from(error: ErrorIO) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<ErrorNetCDF> for ErrorWrapper {
    fn from(error: ErrorNetCDF) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<ErrorVtk> for ErrorWrapper {
    fn from(error: ErrorVtk) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<ReadNpyError> for ErrorWrapper {
    fn from(error: ReadNpyError) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

impl From<String> for ErrorWrapper {
    fn from(message: String) -> ErrorWrapper {
        ErrorWrapper { message }
    }
}

impl From<&str> for ErrorWrapper {
    fn from(message: &str) -> ErrorWrapper {
        ErrorWrapper {
            message: message.to_string(),
        }
    }
}

impl From<WriteNpyError> for ErrorWrapper {
    fn from(error: WriteNpyError) -> ErrorWrapper {
        ErrorWrapper {
            message: error.to_string(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum InputTypes<const N: usize, T>
where
    T: FiniteElementMethods<N>,
{
    Abaqus(T),
    Npy(Voxels),
    Spn(Voxels),
    Stl(Tessellation),
}

enum OutputTypes<const N: usize, T>
where
    T: FiniteElementMethods<N>,
{
    Abaqus(T),
    Exodus(T),
    Mesh(T),
    Npy(Voxels),
    Spn(Voxels),
    Stl(Tessellation),
    Vtk(T),
}

fn invalid_input(file: &str, extension: Option<&str>) -> Result<(), ErrorWrapper> {
    Ok(Err(format!(
        "Invalid extension .{} from input file {}",
        extension.unwrap_or("UNDEFINED"),
        file
    ))?)
}

fn invalid_output(file: &str, extension: Option<&str>) -> Result<(), ErrorWrapper> {
    Ok(Err(format!(
        "Invalid extension .{} from output file {}",
        extension.unwrap_or("UNDEFINED"),
        file
    ))?)
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
                mesh::<HEX, HexahedralFiniteElements>(
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
                    args.dual,
                )
            }
            MeshSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                mesh::<TRI, TriangularFiniteElements>(
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
                    args.dual,
                )
            }
        },
        Some(Commands::Metrics {
            input,
            output,
            quiet,
        }) => {
            is_quiet = quiet;
            metrics::<HEX, HexahedralFiniteElements>(input, output, quiet)
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
        Some(Commands::Smooth { subcommand }) => match subcommand {
            SmoothSubcommand::Hex(args) => {
                is_quiet = args.quiet;
                smooth::<HEX, HexahedralFiniteElements>(
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
                smooth::<TRI, TriangularFiniteElements>(
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

fn convert_mesh(input: String, output: String, quiet: bool) -> Result<(), ErrorWrapper> {
    let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input::<HEX, HexahedralFiniteElements>(&input, None, None, None, quiet)? {
        InputTypes::Abaqus(finite_elements) => match output_extension {
            Some("exo") => write_output(output, OutputTypes::Exodus(finite_elements), quiet),
            Some("inp") => write_output(output, OutputTypes::Abaqus(finite_elements), quiet),
            Some("mesh") => write_output(output, OutputTypes::Mesh(finite_elements), quiet),
            Some("stl") => write_output(
                output,
                OutputTypes::<3, TriangularFiniteElements>::Stl(finite_elements.into_tesselation()),
                quiet,
            ),
            Some("vtk") => write_output(output, OutputTypes::Vtk(finite_elements), quiet),
            _ => invalid_output(&output, output_extension),
        },
        InputTypes::Npy(_voxels) | InputTypes::Spn(_voxels) => {
            invalid_input(&input, input_extension)
        }
        InputTypes::Stl(tessellation) => {
            let finite_elements = tessellation.into_finite_elements();
            match output_extension {
                Some("exo") => write_output(output, OutputTypes::Exodus(finite_elements), quiet),
                Some("inp") => write_output(output, OutputTypes::Abaqus(finite_elements), quiet),
                Some("mesh") => write_output(output, OutputTypes::Mesh(finite_elements), quiet),
                Some("stl") => write_output(
                    output,
                    OutputTypes::<3, TriangularFiniteElements>::Stl(
                        finite_elements.into_tesselation(),
                    ),
                    quiet,
                ),
                Some("vtk") => write_output(output, OutputTypes::Vtk(finite_elements), quiet),
                _ => invalid_output(&output, output_extension),
            }
        }
    }
}

fn convert_segmentation(
    input: String,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input::<HEX, HexahedralFiniteElements>(&input, nelx, nely, nelz, quiet)? {
        InputTypes::Abaqus(_finite_elements) => invalid_input(&input, input_extension),
        InputTypes::Npy(voxels) | InputTypes::Spn(voxels) => match output_extension {
            Some("spn") => write_output(
                output,
                OutputTypes::<HEX, HexahedralFiniteElements>::Spn(voxels),
                quiet,
            ),
            Some("npy") => write_output(
                output,
                OutputTypes::<HEX, HexahedralFiniteElements>::Npy(voxels),
                quiet,
            ),
            _ => invalid_output(&output, output_extension),
        },
        InputTypes::Stl(_voxels) => invalid_input(&input, input_extension),
    }
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
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input::<HEX, HexahedralFiniteElements>(&input, nelx, nely, nelz, quiet)? {
        InputTypes::Npy(mut voxels) | InputTypes::Spn(mut voxels) => match output_extension {
            Some("npy") => {
                let time = Instant::now();
                if !quiet {
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {} voxels or less",
                        min
                    );
                }
                voxels = voxels.defeature(min);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
                write_output(
                    output,
                    OutputTypes::<HEX, HexahedralFiniteElements>::Npy(voxels),
                    quiet,
                )
            }
            Some("spn") => {
                let time = Instant::now();
                if !quiet {
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {} voxels or less",
                        min
                    );
                }
                voxels = voxels.defeature(min);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
                write_output(
                    output,
                    OutputTypes::<HEX, HexahedralFiniteElements>::Spn(voxels),
                    quiet,
                )
            }
            _ => invalid_output(&output, output_extension),
        },
        _ => {
            let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
            Err(format!(
                "Invalid extension .{} from input file {}",
                input_extension.unwrap_or("UNDEFINED"),
                input
            ))?
        }
    }
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
    let extraction = Extraction::from_input([xmin, xmax, ymin, ymax, zmin, zmax])?;
    let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input::<HEX, HexahedralFiniteElements>(&input, nelx, nely, nelz, quiet)? {
        InputTypes::Abaqus(_finite_elements) => invalid_input(&input, input_extension),
        InputTypes::Npy(mut voxels) | InputTypes::Spn(mut voxels) => match output_extension {
            Some("spn") => {
                voxels.extract(extraction);
                write_output(
                    output,
                    OutputTypes::<HEX, HexahedralFiniteElements>::Spn(voxels),
                    quiet,
                )
            }
            Some("npy") => {
                voxels.extract(extraction);
                write_output(
                    output,
                    OutputTypes::<HEX, HexahedralFiniteElements>::Npy(voxels),
                    quiet,
                )
            }
            _ => invalid_output(&output, output_extension),
        },
        InputTypes::Stl(_voxels) => invalid_input(&input, input_extension),
    }
}

enum MeshBasis {
    Leaves,
    Surfaces,
    Voxels,
}

#[allow(clippy::too_many_arguments)]
fn mesh<const N: usize, T>(
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
    dual: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let mut time = Instant::now();
    let remove = remove.map(|removed_blocks| {
        removed_blocks
            .into_iter()
            .map(|entry| entry as u8)
            .collect()
    });
    let scale = Scale::from([xscale, yscale, zscale]);
    let translate = Translate::from([xtranslate, ytranslate, ztranslate]);
    let mut input_type = match read_input::<N, T>(&input, nelx, nely, nelz, quiet)? {
        InputTypes::Npy(voxels) => voxels,
        InputTypes::Spn(voxels) => voxels,
        _ => {
            let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
            Err(format!(
                "Invalid extension .{} from input file {}",
                input_extension.unwrap_or("UNDEFINED"),
                input
            ))?
        }
    };
    match N {
        HEX => {
            if let Some(min_num_voxels) = defeature {
                if !quiet {
                    time = Instant::now();
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {} voxels or less",
                        min_num_voxels
                    );
                }
                input_type = input_type.defeature(min_num_voxels);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
            }
            if !quiet {
                time = Instant::now();
                mesh_print_info(MeshBasis::Voxels, &scale, &translate)
            }
            let mut output_type = if dual {
                let (nel_padded, mut tree) = Octree::from_voxels(input_type);
                tree.balance(true);
                tree.pair();
                tree.into_finite_elements(nel_padded, remove, scale, translate)?
            } else {
                input_type.into_finite_elements(remove, scale, translate)?
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
                metrics_inner(&output_type, file, quiet)?
            }
            let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
            match output_extension {
                Some("exo") => write_output(output, OutputTypes::Exodus(output_type), quiet)?,
                Some("inp") => write_output(output, OutputTypes::Abaqus(output_type), quiet)?,
                Some("mesh") => write_output(output, OutputTypes::Mesh(output_type), quiet)?,
                Some("vtk") => write_output(output, OutputTypes::Vtk(output_type), quiet)?,
                _ => invalid_output(&output, output_extension)?,
            }
        }
        TRI => {
            if !quiet {
                time = Instant::now();
                if let Some(min_num_voxels) = defeature {
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {} voxels or less",
                        min_num_voxels
                    );
                } else {
                    mesh_print_info(MeshBasis::Surfaces, &scale, &translate)
                }
            }
            let (nel_padded, mut tree) = Octree::from_voxels(input_type);
            tree.balance(true);
            if let Some(min_num_voxels) = defeature {
                tree.defeature(min_num_voxels);
                println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                time = Instant::now();
                mesh_print_info(MeshBasis::Surfaces, &scale, &translate)
            }
            let mut output_type: TriangularFiniteElements =
                tree.into_finite_elements(nel_padded, remove, scale, translate)?;
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!("        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m", time.elapsed(), blocks.len(), elements, output_type.get_nodal_coordinates().len());
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
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
            let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
            match output_extension {
                Some("exo") => write_output(output, OutputTypes::Exodus(output_type), quiet)?,
                Some("inp") => write_output(output, OutputTypes::Abaqus(output_type), quiet)?,
                Some("mesh") => write_output(output, OutputTypes::Mesh(output_type), quiet)?,
                Some("stl") => write_output(
                    output,
                    OutputTypes::<3, TriangularFiniteElements>::Stl(output_type.into_tesselation()),
                    quiet,
                )?,
                Some("vtk") => write_output(output, OutputTypes::Vtk(output_type), quiet)?,
                _ => invalid_output(&output, output_extension)?,
            }
        }
        _ => panic!(),
    }
    Ok(())
}

fn mesh_print_info(basis: MeshBasis, scale: &Scale, translate: &Translate) {
    match basis {
        MeshBasis::Leaves => {
            print!("     \x1b[1;96mMeshing\x1b[0m leaves into hexes")
        }
        MeshBasis::Surfaces => {
            print!("     \x1b[1;96mMeshing\x1b[0m internal surfaces")
        }
        MeshBasis::Voxels => {
            print!("     \x1b[1;96mMeshing\x1b[0m voxels into hexes");
        }
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

fn metrics<const N: usize, T>(
    input: String,
    output: String,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let output_type = match read_input::<N, T>(&input, None, None, None, quiet)? {
        InputTypes::Abaqus(finite_elements) => finite_elements,
        InputTypes::Npy(_) | InputTypes::Spn(_) => {
            Err(format!("No metrics for segmentation file {}", input))?
        }
        InputTypes::Stl(_) => todo!(),
    };
    metrics_inner(&output_type, output, quiet)
}

fn metrics_inner<const N: usize, T>(
    fem: &T,
    output: String,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let time = Instant::now();
    if !quiet {
        println!("     \x1b[1;96mMetrics\x1b[0m {}", output);
    }
    fem.write_metrics(&output)?;
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
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
    let remove = remove.map(|removed_blocks| {
        removed_blocks
            .into_iter()
            .map(|entry| entry as u8)
            .collect()
    });
    let scale = [xscale, yscale, zscale].into();
    let translate = [xtranslate, ytranslate, ztranslate].into();
    let input_type =
        match read_input::<HEX, HexahedralFiniteElements>(&input, nelx, nely, nelz, quiet)? {
            InputTypes::Npy(voxels) => voxels,
            InputTypes::Spn(voxels) => voxels,
            _ => {
                let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
                Err(format!(
                    "Invalid extension .{} from input file {}",
                    input_extension.unwrap_or("UNDEFINED"),
                    input
                ))?
            }
        };
    let time = Instant::now();
    if !quiet {
        mesh_print_info(MeshBasis::Leaves, &scale, &translate)
    }
    let (_, mut tree) = Octree::from_voxels(input_type);
    tree.balance(strong);
    if pair {
        tree.pair();
    }
    tree.prune();
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    let output_type = tree.octree_into_finite_elements(remove, scale, translate)?;
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
    match output_extension {
        Some("exo") => write_output(output, OutputTypes::Exodus(output_type), quiet)?,
        Some("inp") => write_output(output, OutputTypes::Abaqus(output_type), quiet)?,
        Some("mesh") => write_output(output, OutputTypes::Mesh(output_type), quiet)?,
        Some("vtk") => write_output(output, OutputTypes::Vtk(output_type), quiet)?,
        _ => invalid_output(&output, output_extension)?,
    }
    Ok(())
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
    T: FiniteElementMethods<N>,
{
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input::<N, T>(&input, None, None, None, quiet)? {
        InputTypes::Abaqus(mut finite_elements) => {
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
                metrics_inner(&finite_elements, file, quiet)?
            }
            match output_extension {
                Some("exo") => write_output(output, OutputTypes::Exodus(finite_elements), quiet),
                Some("inp") => write_output(output, OutputTypes::Abaqus(finite_elements), quiet),
                Some("mesh") => write_output(output, OutputTypes::Mesh(finite_elements), quiet),
                Some("stl") => write_output(
                    output,
                    OutputTypes::<TRI, TriangularFiniteElements>::Stl(
                        finite_elements.into_tesselation(),
                    ),
                    quiet,
                ),
                Some("vtk") => write_output(output, OutputTypes::Vtk(finite_elements), quiet),
                _ => invalid_output(&output, output_extension),
            }
        }
        InputTypes::Stl(tesselation) => {
            let mut finite_elements = tesselation.into_finite_elements();
            apply_smoothing_method(
                &mut finite_elements,
                iterations,
                method,
                hierarchical,
                pass_band,
                scale,
                quiet,
            )?;
            match output_extension {
                Some("exo") => write_output(output, OutputTypes::Exodus(finite_elements), quiet),
                Some("inp") => write_output(output, OutputTypes::Abaqus(finite_elements), quiet),
                Some("mesh") => write_output(output, OutputTypes::Mesh(finite_elements), quiet),
                Some("stl") => write_output(
                    output,
                    OutputTypes::<3, TriangularFiniteElements>::Stl(
                        finite_elements.into_tesselation(),
                    ),
                    quiet,
                ),
                Some("vtk") => write_output(output, OutputTypes::Vtk(finite_elements), quiet),
                _ => invalid_output(&output, output_extension),
            }
        }
        InputTypes::Npy(_) | InputTypes::Spn(_) => {
            Err(format!("No smoothing for segmentation file {}", input))?
        }
    }
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
                    println!("with {} iterations of Laplace", iterations)
                }
                "Taubin" | "taubin" => {
                    println!("with {} iterations of Taubin", iterations)
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
                output_type.smooth(Smoothing::Laplacian(iterations, scale))?;
            }
            "Taubin" | "taubin" => {
                output_type.smooth(Smoothing::Taubin(iterations, pass_band, scale))?;
            }
            _ => panic!(),
        }
        if !quiet {
            println!("        \x1b[1;92mDone\x1b[0m {:?}", time_smooth.elapsed());
        }
        Ok(())
    } else {
        Err(format!(
            "Invalid smoothing method {} specified",
            smoothing_method
        ))?
    }
}

fn read_input<const N: usize, T>(
    input: &str,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<InputTypes<N, T>, ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let time = Instant::now();
    if !quiet {
        println!(
            "\x1b[1m    {} {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        print!("     \x1b[1;96mReading\x1b[0m {}", input);
    }
    let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
    let result = match input_extension {
        Some("inp") => InputTypes::Abaqus(T::from_inp(input)?),
        Some("npy") => InputTypes::Npy(Voxels::from_npy(input)?),
        Some("spn") => {
            let nel = Nel::from_input([nelx, nely, nelz])?;
            if !quiet {
                print!(
                    " \x1b[2m[nelx: {}, nely: {}, nelz: {}]",
                    nel.x(),
                    nel.y(),
                    nel.z(),
                );
            }
            InputTypes::Spn(Voxels::from_spn(input, nel)?)
        }
        Some("stl") => InputTypes::Stl(Tessellation::from_stl(input)?),
        _ => Err(format!(
            "Invalid extension .{} from input file {}",
            input_extension.unwrap_or("UNDEFINED"),
            input
        ))?,
    };
    if !quiet {
        print!(
            "\x1b[0m\n        \x1b[1;92mDone\x1b[0m {:?}",
            time.elapsed()
        );
        match &result {
            InputTypes::Npy(voxels) | InputTypes::Spn(voxels) => {
                let mut materials: Blocks = voxels.get_data().iter().copied().collect();
                let voxels = materials.len();
                materials.sort();
                materials.dedup();
                println!(
                    " \x1b[2m[{} materials, {} voxels]\x1b[0m",
                    materials.len(),
                    voxels
                );
            }
            _ => {
                println!();
            }
        }
    }
    Ok(result)
}

fn write_output<const N: usize, T>(
    output: String,
    output_type: OutputTypes<N, T>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<N>,
{
    let time = Instant::now();
    if !quiet {
        println!("     \x1b[1;96mWriting\x1b[0m {}", output);
    }
    match output_type {
        OutputTypes::Abaqus(fem) => fem.write_inp(&output)?,
        OutputTypes::Exodus(fem) => fem.write_exo(&output)?,
        OutputTypes::Mesh(fem) => fem.write_mesh(&output)?,
        OutputTypes::Npy(voxels) => voxels.write_npy(&output)?,
        OutputTypes::Spn(voxels) => voxels.write_spn(&output)?,
        OutputTypes::Stl(tessellation) => tessellation.write_stl(&output)?,
        OutputTypes::Vtk(fem) => fem.write_vtk(&output)?,
    }
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}
