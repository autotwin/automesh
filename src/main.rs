use clap::{Parser, Subcommand};
use std::{
    env::consts::{ARCH, OS},
    time::Instant,
};

mod convert;
mod defeature;
mod diff;
mod error;
mod extract;
mod io;
mod log;
mod mesh;
mod metrics;
mod remesh;
mod segment;
mod smooth;

use convert::{ConvertSubcommand, convert_mesh, convert_segmentation};
use defeature::defeature;
use diff::diff;
use error::ErrorWrapper;
use extract::extract;
use mesh::{Element, MeshSubcommand};
use metrics::{MetricsArgs, metrics};
use remesh::{MeshRemeshCommands, remesh};
use segment::{SegmentArgs, segment};
use smooth::{SmoothArgs, smooth};

macro_rules! about {
    () => {
        format!(
            "

     @@@@@@@@@@@@@@@@
      @@@@  @@@@@@@@@@
     @@@@  @@@@@@@@@@@    \x1b[1;4m{}: Automatic mesh generation\x1b[0m
    @@@@  @@@@@@@@@@@@
      @@    @@    @@      {}
      @@    @@    @@      {}
    @@@@@@@@@@@@  @@@     {}
    @@@@@@@@@@@  @@@@     {}
    @@@@@@@@@@ @@@@@ @
     @@@@@@@@@@@@@@@@",
            env!("CARGO_PKG_NAME"),
            format!("v{} {} {}", env!("CARGO_PKG_VERSION"), OS, ARCH),
            format!(
                "build {} {}",
                option_env!("GIT_COMMIT_HASH").unwrap_or(""),
                env!("BUILD_TIME"),
            ),
            env!("CARGO_PKG_AUTHORS").split(':').next().unwrap_or(""),
            env!("CARGO_PKG_AUTHORS").split(':').nth(1).unwrap_or(""),
        )
    };
}

#[derive(Parser)]
#[command(about = about!(), arg_required_else_help = true, version)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Mirror terminal output to a log file
    #[arg(global = true, long, value_name = "FILE")]
    log: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, global = true, long, short)]
    quiet: bool,
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

        /// Defeatured segmentation output file (npy | spn | vti)
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
    },

    /// Show the difference between two segmentations
    Diff {
        /// Segmentation input files (npy | spn)
        #[arg(long, num_args = 2, short, value_delimiter = ' ', value_name = "FILE")]
        input: Vec<String>,

        /// Segmentation difference output file (npy | spn | vti)
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
    },

    /// Extracts a specified range of voxels from a segmentation
    Extract {
        /// Segmentation input file (npy | spn)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Extracted segmentation output file (npy | spn | vti)
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
    },

    /// Creates a finite element mesh from a segmentation
    Mesh {
        #[command(subcommand)]
        subcommand: MeshSubcommand,
    },

    /// Quality metrics for an existing finite element mesh
    Metrics(MetricsArgs),

    /// Applies isotropic remeshing to an existing mesh [default mode: uniform]
    Remesh {
        /// Mesh input file (exo | inp | stl | vtu)
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Mesh output file (exo | inp | mesh | stl | vtu)
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Sizing mode [default: uniform]
        #[command(subcommand)]
        mode: Option<MeshRemeshCommands>,
    },

    /// Creates a segmentation or voxelized mesh from an existing mesh
    Segment(SegmentArgs),

    /// Applies smoothing to an existing mesh
    Smooth(SmoothArgs),
}

fn main() -> Result<(), ErrorWrapper> {
    let time = Instant::now();
    let args = Args::parse();
    if let Some(path) = &args.log {
        log::set_logfile(path)?;
        log::write_log(&about!());
        log::write_log("");
    }
    let quiet = args.quiet;
    let result = match args.command {
        Some(Commands::Convert { subcommand }) => match subcommand {
            ConvertSubcommand::Mesh(args) => convert_mesh(args, quiet),
            ConvertSubcommand::Segmentation(args) => convert_segmentation(
                args.input,
                args.output,
                args.nelx,
                args.nely,
                args.nelz,
                quiet,
            ),
        },
        Some(Commands::Defeature {
            input,
            output,
            min,
            nelx,
            nely,
            nelz,
        }) => defeature(input, output, min, nelx, nely, nelz, quiet),
        Some(Commands::Diff {
            input,
            output,
            nelx,
            nely,
            nelz,
        }) => diff(input, output, nelx, nely, nelz, quiet),
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
        }) => extract(
            input, output, nelx, nely, nelz, xmin, xmax, ymin, ymax, zmin, zmax, quiet,
        ),
        Some(Commands::Mesh { subcommand }) => match subcommand {
            MeshSubcommand::Hex(args) => mesh::mesh(Element::Hexahedra, args, quiet),
            MeshSubcommand::Tri(args) => mesh::mesh(Element::Triangles, args, quiet),
        },
        Some(Commands::Metrics(args)) => metrics(args, quiet),
        Some(Commands::Remesh {
            input,
            output,
            mode,
        }) => remesh(input, output, mode, quiet),
        Some(Commands::Segment(args)) => segment(args, quiet),
        Some(Commands::Smooth(args)) => smooth(args, quiet),
        None => return Ok(()),
    };
    crate::echo!(quiet, "       \x1b[1;98mTotal\x1b[0m {:?}", time.elapsed());
    result
}
