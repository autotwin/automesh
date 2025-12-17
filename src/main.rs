use automesh::{
    FiniteElementMethods, HexahedralFiniteElements, Octree, Remove, Scale,
    TetrahedralFiniteElements, Translate, TriangularFiniteElements,
};
use clap::{Parser, Subcommand};
use conspire::math::Tensor;
use std::time::Instant;

mod cli;
use cli::{
    ErrorWrapper,
    convert::{ConvertSubcommand, convert_mesh, convert_segmentation},
    defeature::defeature,
    diff::diff,
    extract::extract,
    input::{read_finite_elements, read_segmentation},
    mesh::{MeshSubcommand, mesh, mesh_print_info},
    metrics::MetricsSubcommand,
    output::{write_finite_elements, write_metrics},
    remesh::{REMESH_DEFAULT_ITERS, remesh},
    segment::{SegmentSubcommand, segment},
    smooth::{SmoothSubcommand, smooth},
};
use std::env::consts::{ARCH, OS};

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

    /// Creates a finite element mesh from a tessellation or segmentation
    Mesh {
        #[command(subcommand)]
        subcommand: MeshSubcommand,
    },

    /// Quality metrics for an existing finite element mesh
    Metrics {
        #[command(subcommand)]
        subcommand: MetricsSubcommand,
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
        /// Mesh input file (exo | inp | stl)
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

    /// Creates a segmentation or voxelized mesh from an existing mesh
    Segment {
        #[command(subcommand)]
        subcommand: SegmentSubcommand,
    },

    /// Applies smoothing to an existing mesh
    Smooth {
        #[command(subcommand)]
        subcommand: SmoothSubcommand,
    },
}

fn main() -> Result<(), ErrorWrapper> {
    let time = Instant::now();
    let is_quiet;
    let args = Args::parse();
    let result = match args.command {
        Some(Commands::Convert { subcommand }) => match subcommand {
            ConvertSubcommand::Mesh(args) => {
                is_quiet = args.subcommand.is_quiet();
                convert_mesh(args.subcommand)
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
                mesh::<_, _, HexahedralFiniteElements>(
                    args.smoothing,
                    args.input,
                    args.output,
                    args.defeature,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.remove,
                    args.size,
                    args.xscale,
                    args.yscale,
                    args.zscale,
                    args.xtranslate,
                    args.ytranslate,
                    args.ztranslate,
                    args.metrics,
                    args.quiet,
                )
            }
            MeshSubcommand::Tet(args) => {
                is_quiet = args.quiet;
                mesh::<_, _, TetrahedralFiniteElements>(
                    args.smoothing,
                    args.input,
                    args.output,
                    args.defeature,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.remove,
                    args.size,
                    args.xscale,
                    args.yscale,
                    args.zscale,
                    args.xtranslate,
                    args.ytranslate,
                    args.ztranslate,
                    args.metrics,
                    args.quiet,
                )
            }
            MeshSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                mesh::<_, _, TriangularFiniteElements>(
                    args.smoothing,
                    args.input,
                    args.output,
                    args.defeature,
                    args.nelx,
                    args.nely,
                    args.nelz,
                    args.remove,
                    args.size,
                    args.xscale,
                    args.yscale,
                    args.zscale,
                    args.xtranslate,
                    args.ytranslate,
                    args.ztranslate,
                    args.metrics,
                    args.quiet,
                )
            }
        },
        Some(Commands::Metrics { subcommand }) => match subcommand {
            MetricsSubcommand::Hex(args) => {
                is_quiet = args.quiet;
                write_metrics(
                    &read_finite_elements::<_, _, HexahedralFiniteElements>(
                        &args.input,
                        args.quiet,
                        true,
                    )?,
                    args.output,
                    args.quiet,
                )
            }
            MetricsSubcommand::Tet(args) => {
                is_quiet = args.quiet;
                write_metrics(
                    &read_finite_elements::<_, _, TetrahedralFiniteElements>(
                        &args.input,
                        args.quiet,
                        true,
                    )?,
                    args.output,
                    args.quiet,
                )
            }
            MetricsSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                write_metrics(
                    &read_finite_elements::<_, _, TriangularFiniteElements>(
                        &args.input,
                        args.quiet,
                        true,
                    )?,
                    args.output,
                    args.quiet,
                )
            }
        },
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
        Some(Commands::Segment { subcommand }) => match subcommand {
            SegmentSubcommand::Hex(args) => {
                is_quiet = args.quiet;
                segment::<_, _, HexahedralFiniteElements, _, _, HexahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.grid,
                    args.size,
                    args.remove,
                    args.quiet,
                )
            }
            SegmentSubcommand::Tet(args) => {
                is_quiet = args.quiet;
                segment::<_, _, TetrahedralFiniteElements, _, _, HexahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.grid,
                    args.size,
                    args.remove,
                    args.quiet,
                )
            }
            SegmentSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                segment::<_, _, TriangularFiniteElements, _, _, HexahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.grid,
                    args.size,
                    args.remove,
                    args.quiet,
                )
            }
        },
        Some(Commands::Smooth { subcommand }) => match subcommand {
            SmoothSubcommand::Hex(args) => {
                is_quiet = args.quiet;
                smooth::<_, _, HexahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.iterations,
                    args.method,
                    args.hierarchical,
                    args.pass_band,
                    args.scale,
                    args.remeshing,
                    args.metrics,
                    args.quiet,
                )
            }
            SmoothSubcommand::Tet(args) => {
                is_quiet = args.quiet;
                smooth::<_, _, TetrahedralFiniteElements>(
                    args.input,
                    args.output,
                    args.iterations,
                    args.method,
                    args.hierarchical,
                    args.pass_band,
                    args.scale,
                    args.remeshing,
                    args.metrics,
                    args.quiet,
                )
            }
            SmoothSubcommand::Tri(args) => {
                is_quiet = args.quiet;
                smooth::<_, _, TriangularFiniteElements>(
                    args.input,
                    args.output,
                    args.iterations,
                    args.method,
                    args.hierarchical,
                    args.pass_band,
                    args.scale,
                    args.remeshing,
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

// temporary beta feature to aid in debugging
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
        mesh_print_info(
            cli::mesh::MeshBasis::Leaves,
            &scale_temporary,
            &translate_temporary,
        )
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
