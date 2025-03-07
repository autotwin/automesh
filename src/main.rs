use automesh::{
    FiniteElementMethods, FiniteElementSpecifics, HexahedralFiniteElements, IntoFiniteElements,
    Nel, Octree, Scale, Smoothing, Tessellation, Translate, Tree, TriangularFiniteElements, Voxels,
};
use clap::{Parser, Subcommand};
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
    @@@@@@@@@@@  @@@@     \x1b[1;4mNotes:\x1b[0m
    @@@@@@@@@@ @@@@@ @    - Input/output file types are inferred
     @@@@@@@@@@@@@@@@     - Scaling is applied before translation",
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
        /// Mesh (inp) or segmentation (npy | spn) input file
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Mesh (exo | mesh | stl | vtk) or segmentation (npy | spn) output
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

    /// Creates a finite element mesh from a segmentation
    Mesh {
        #[command(subcommand)]
        meshing: Option<MeshingCommands>,

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

        /// Scaling (> 0.0) in the x-direction
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        xscale: f64,

        /// Scaling (> 0.0) in the y-direction
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        yscale: f64,

        /// Scaling (> 0.0) in the z-direction
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

        /// Name of the quality metrics file
        #[arg(long, value_name = "FILE")]
        metrics: Option<String>,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,

        /// Pass to mesh using dualization
        #[arg(action, hide = true, long)]
        dual: bool,

        /// Pass to mesh internal surfaces
        #[arg(action, long)]
        surface: bool,
    },

    /// Quality metrics for an existing finite element mesh
    Metrics {
        /// Mesh (inp | stl) input file
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

        /// Octree output file (exo | inp | mesh | stl | vtk)
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

        /// Scaling (> 0.0) in the x-direction
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        xscale: f64,

        /// Scaling (> 0.0) in the y-direction
        #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
        yscale: f64,

        /// Scaling (> 0.0) in the z-direction
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
        /// Pass to enable hierarchical control
        #[arg(action, long, short = 'c')]
        hierarchical: bool,

        /// Mesh (inp | stl) input file
        #[arg(long, short, value_name = "FILE")]
        input: String,

        /// Smoothed mesh (exo | inp | mesh | stl | vtk) output file
        #[arg(long, short, value_name = "FILE")]
        output: String,

        /// Number of smoothing iterations
        #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Name of the smoothing method [default: Taubin]
        #[arg(long, short, value_name = "NAME")]
        method: Option<String>,

        /// Pass-band frequency for Taubin smoothing
        #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
        pass_band: f64,

        /// Scaling parameter for smoothing
        #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
        scale: f64,

        /// Name of the quality metrics file (csv | npy)
        #[arg(long, value_name = "FILE")]
        metrics: Option<String>,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },
}

#[derive(Subcommand)]
enum MeshingCommands {
    /// Applies smoothing to the mesh before output
    Smooth {
        /// Pass to enable hierarchical control
        #[arg(action, long, short = 'c')]
        hierarchical: bool,

        /// Number of smoothing iterations
        #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Name of the smoothing method [default: Taubin]
        #[arg(long, short, value_name = "NAME")]
        method: Option<String>,

        /// Pass-band frequency for Taubin smoothing
        #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
        pass_band: f64,

        /// Scaling parameter for smoothing
        #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
        scale: f64,
    },
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
enum InputTypes {
    Abaqus(HexahedralFiniteElements),
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
        Some(Commands::Convert {
            input,
            output,
            nelx,
            nely,
            nelz,
            quiet,
        }) => {
            is_quiet = quiet;
            convert(input, output, nelx, nely, nelz, quiet)
        }
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
        Some(Commands::Mesh {
            meshing,
            input,
            output,
            defeature,
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
            metrics,
            quiet,
            dual,
            surface,
        }) => {
            is_quiet = quiet;
            mesh(
                meshing, input, output, defeature, nelx, nely, nelz, remove, xscale, yscale,
                zscale, xtranslate, ytranslate, ztranslate, metrics, quiet, dual, surface,
            )
        }
        Some(Commands::Metrics {
            input,
            output,
            quiet,
        }) => {
            is_quiet = quiet;
            metrics(input, output, quiet)
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
        Some(Commands::Smooth {
            input,
            output,
            iterations,
            method,
            hierarchical,
            pass_band,
            scale,
            metrics,
            quiet,
        }) => {
            is_quiet = quiet;
            smooth(
                input,
                output,
                iterations,
                method,
                hierarchical,
                pass_band,
                scale,
                metrics,
                quiet,
            )
        }
        None => return Ok(()),
    };
    if !is_quiet {
        println!("       \x1b[1;98mTotal\x1b[0m {:?}", time.elapsed());
    }
    result
}

fn convert(
    input: String,
    output: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input(&input, nelx, nely, nelz, quiet)? {
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
        InputTypes::Npy(voxels) | InputTypes::Spn(voxels) => match output_extension {
            Some("spn") => write_output(
                output,
                OutputTypes::<8, HexahedralFiniteElements>::Spn(voxels),
                quiet,
            ),
            Some("npy") => write_output(
                output,
                OutputTypes::<8, HexahedralFiniteElements>::Npy(voxels),
                quiet,
            ),
            _ => invalid_output(&output, output_extension),
        },
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
    match read_input(&input, nelx, nely, nelz, quiet)? {
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
                    OutputTypes::<8, HexahedralFiniteElements>::Npy(voxels),
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
                    OutputTypes::<8, HexahedralFiniteElements>::Spn(voxels),
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
fn mesh(
    meshing: Option<MeshingCommands>,
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
    surface: bool,
) -> Result<(), ErrorWrapper> {
    let mut time = Instant::now();
    let remove = remove.map(|removed_blocks| {
        removed_blocks
            .into_iter()
            .map(|entry| entry as u8)
            .collect()
    });
    let scale = Scale::from([xscale, yscale, zscale]);
    let translate = Translate::from([xtranslate, ytranslate, ztranslate]);
    let mut input_type = match read_input(&input, nelx, nely, nelz, quiet)? {
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
    if !quiet {
        let entirely_default = scale == Default::default() && translate == Default::default();
        if !entirely_default {
            print!("\x1b[u \x1b[A\x1b[2m[");
        }
        if xscale != 1.0 {
            print!("xscale: {}, ", scale.x());
        }
        if yscale != 1.0 {
            print!("yscale: {}, ", scale.y());
        }
        if zscale != 1.0 {
            print!("zscale: {}, ", scale.z());
        }
        if xtranslate != 0.0 {
            print!("xtranslate: {}, ", xtranslate);
        }
        if ytranslate != 0.0 {
            print!("ytranslate: {}, ", ytranslate);
        }
        if ztranslate != 0.0 {
            print!("ztranslate: {}, ", ztranslate);
        }
        if !entirely_default {
            println!("\x1b[2D]\x1b[0m");
        }
    }
    if surface {
        if !quiet {
            time = Instant::now();
            if let Some(min_num_voxels) = defeature {
                println!(
                    " \x1b[1;96mDefeaturing\x1b[0m clusters of {} voxels or less",
                    min_num_voxels
                );
            } else {
                println!("     \x1b[1;96mMeshing\x1b[0m internal surfaces")
            }
        }
        let (nel_padded, mut tree) = Octree::from_voxels(input_type);
        tree.balance(true);
        if let Some(min_num_voxels) = defeature {
            tree.defeature(min_num_voxels);
            println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
            time = Instant::now();
            println!("     \x1b[1;96mMeshing\x1b[0m internal surfaces")
        }
        let mut output_type: TriangularFiniteElements =
            tree.into_finite_elements(nel_padded, remove, scale, translate)?;
        if !quiet {
            println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
        }
        if let Some(options) = meshing {
            match options {
                MeshingCommands::Smooth {
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
    } else {
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
            println!("     \x1b[1;96mMeshing\x1b[0m {}", output);
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
            println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
        }
        if let Some(options) = meshing {
            match options {
                MeshingCommands::Smooth {
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
    Ok(())
}

fn metrics(input: String, output: String, quiet: bool) -> Result<(), ErrorWrapper> {
    let output_type = match read_input(&input, None, None, None, quiet)? {
        InputTypes::Abaqus(finite_elements) => finite_elements,
        InputTypes::Npy(_) | InputTypes::Spn(_) => {
            Err(format!("No metrics for segmentation file {}", input))?
        }
        InputTypes::Stl(_) => todo!(),
    };
    metrics_inner(&output_type, output, quiet)
}

fn metrics_inner(
    fem: &HexahedralFiniteElements,
    output: String,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
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
    let input_type = match read_input(&input, nelx, nely, nelz, quiet)? {
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
        println!("     \x1b[1;96mMeshing\x1b[0m {}", output);
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
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
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
fn smooth(
    input: String,
    output: String,
    iterations: usize,
    method: Option<String>,
    hierarchical: bool,
    pass_band: f64,
    scale: f64,
    metrics: Option<String>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let output_extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match read_input(&input, None, None, None, quiet)? {
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
                    OutputTypes::<3, TriangularFiniteElements>::Stl(
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
        "Gauss"
            | "gauss"
            | "Gaussian"
            | "gaussian"
            | "Laplacian"
            | "Laplace"
            | "laplacian"
            | "laplace"
            | "Taubin"
            | "taubin"
    ) {
        if !quiet {
            print!("   \x1b[1;96mSmoothing\x1b[0m ");
            match smoothing_method.as_str() {
                "Gauss" | "gauss" | "Gaussian" | "gaussian" | "Laplacian" | "Laplace"
                | "laplacian" | "laplace" => {
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
            "Gauss" | "gauss" | "Gaussian" | "gaussian" | "Laplacian" | "Laplace" | "laplacian"
            | "laplace" => {
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

fn read_input(
    input: &str,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
) -> Result<InputTypes, ErrorWrapper> {
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
        Some("inp") => {
            if !quiet {
                println!();
            }
            InputTypes::Abaqus(HexahedralFiniteElements::from_inp(input)?)
        }
        Some("npy") => {
            if !quiet {
                println!();
            }
            InputTypes::Npy(Voxels::from_npy(input)?)
        }
        Some("spn") => {
            let nel = Nel::from_input([nelx, nely, nelz])?;
            if !quiet {
                println!(
                    " \x1b[2m[nelx: {}, nely: {}, nelz: {}]\x1b[0m",
                    nel.x(),
                    nel.y(),
                    nel.z(),
                );
            }
            InputTypes::Spn(Voxels::from_spn(input, nel)?)
        }
        Some("stl") => {
            if !quiet {
                println!();
            }
            InputTypes::Stl(Tessellation::from_stl(input)?)
        }
        _ => {
            if !quiet {
                println!();
            }
            Err(format!(
                "Invalid extension .{} from input file {}",
                input_extension.unwrap_or("UNDEFINED"),
                input
            ))?
        }
    };
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}\x1b[s", time.elapsed());
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
