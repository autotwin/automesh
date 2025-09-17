use super::{
    ErrorWrapper,
    input::read_finite_elements,
    output::{write_finite_elements, write_metrics},
    remesh::{MeshRemeshCommands, apply_remeshing},
};
use automesh::{FiniteElementMethods, Smoothing, Tessellation};
use clap::Subcommand;
use std::time::Instant;

pub const TAUBIN_DEFAULT_ITERS: usize = 20;
pub const TAUBIN_DEFAULT_BAND: f64 = 0.1;
pub const TAUBIN_DEFAULT_SCALE: f64 = 0.6307;

#[derive(Subcommand, Debug)]
pub enum MeshSmoothCommands {
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
pub enum SmoothSubcommand {
    /// Smooths an all-hexahedral mesh
    Hex(SmoothElementArgs),
    /// Smooths an all-tetrahedral mesh
    Tet(SmoothElementArgs),
    /// Smooths an all-triangular mesh
    Tri(SmoothTriArgs),
}

#[derive(clap::Args)]
pub struct SmoothElementArgs {
    #[command(subcommand)]
    pub remeshing: Option<MeshRemeshCommands>,

    /// Pass to enable hierarchical control
    #[arg(action, long, short = 'c')]
    pub hierarchical: bool,

    /// Mesh input file (exo | inp)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Smoothed mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Number of smoothing iterations
    #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
    pub iterations: usize,

    /// Smoothing method (Laplace | Taubin) [default: Taubin]
    #[arg(long, short, value_name = "NAME")]
    pub method: Option<String>,

    /// Pass-band frequency (for Taubin only)
    #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
    pub pass_band: f64,

    /// Scaling parameter for all smoothing methods
    #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
    pub scale: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

#[derive(clap::Args)]
pub struct SmoothTriArgs {
    #[command(subcommand)]
    pub remeshing: Option<MeshRemeshCommands>,

    /// Pass to enable hierarchical control
    #[arg(action, long, short = 'c')]
    pub hierarchical: bool,

    /// Mesh input file (exo | inp | stl)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Smoothed mesh output file (exo | inp | mesh | stl | vtk)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Number of smoothing iterations
    #[arg(default_value_t = 20, long, short = 'n', value_name = "NUM")]
    pub iterations: usize,

    /// Smoothing method (Laplace | Taubin) [default: Taubin]
    #[arg(long, short, value_name = "NAME")]
    pub method: Option<String>,

    /// Pass-band frequency (for Taubin only)
    #[arg(default_value_t = 0.1, long, short = 'k', value_name = "FREQ")]
    pub pass_band: f64,

    /// Scaling parameter for all smoothing methods
    #[arg(default_value_t = 0.6307, long, short, value_name = "SCALE")]
    pub scale: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn smooth<const M: usize, const N: usize, T>(
    input: String,
    output: String,
    iterations: usize,
    method: Option<String>,
    hierarchical: bool,
    pass_band: f64,
    scale: f64,
    remeshing: Option<MeshRemeshCommands>,
    metrics: Option<String>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N> + From<Tessellation>,
    Tessellation: From<T>,
{
    let mut finite_elements: T = read_finite_elements(&input, quiet, true)?;
    apply_smoothing_method(
        &mut finite_elements,
        iterations,
        method,
        hierarchical,
        pass_band,
        scale,
        quiet,
    )?;
    if let Some(MeshRemeshCommands::Remesh {
        iterations,
        quiet: _,
    }) = remeshing
    {
        apply_remeshing(&mut finite_elements, iterations, quiet, true)?;
    }
    if let Some(file) = metrics {
        write_metrics(&finite_elements, file, quiet)?
    }
    write_finite_elements(output, finite_elements, quiet)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_smoothing_method<const M: usize, const N: usize, T>(
    output_type: &mut T,
    iterations: usize,
    method: Option<String>,
    hierarchical: bool,
    pass_band: f64,
    scale: f64,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N>,
{
    let time = Instant::now();
    let smoothing_method = method.unwrap_or("Taubin".to_string());
    if matches!(
        smoothing_method.as_str(),
        "Energetic"
            | "energetic"
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
                "Energetic" | "energetic" => {
                    println!("with energetic smoothing")
                }
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
            "Energetic" | "energetic" => {
                output_type.smooth(&Smoothing::Energetic)?;
            }
            "Laplacian" | "Laplace" | "laplacian" | "laplace" => {
                output_type.smooth(&Smoothing::Laplacian(iterations, scale))?;
            }
            "Taubin" | "taubin" => {
                output_type.smooth(&Smoothing::Taubin(iterations, pass_band, scale))?;
            }
            _ => panic!(),
        }
        if !quiet {
            println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
        }
        Ok(())
    } else {
        Err(format!(
            "Invalid smoothing method {smoothing_method} specified",
        ))?
    }
}
