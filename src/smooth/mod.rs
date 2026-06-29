use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
    metrics::write_metrics,
    remesh::{MeshRemeshCommands, apply_remeshing},
};
use clap::Subcommand;
use conspire::geometry::mesh::{Mesh, Smoothing, Weighting};
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

        /// Enables hierarchical smoothing
        #[arg(action, long, short = 'b')]
        hierarchical: bool,
    },
}

#[derive(clap::Args)]
pub struct SmoothArgs {
    #[command(subcommand)]
    pub remeshing: Option<MeshRemeshCommands>,

    /// Mesh input file (exo | inp | stl | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Smoothed mesh output file (exo | inp | mesh | stl | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Number of smoothing iterations
    #[arg(default_value_t = TAUBIN_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
    pub iterations: usize,

    /// Smoothing method (Laplace | Taubin) [default: Taubin]
    #[arg(long, short, value_name = "NAME")]
    pub method: Option<String>,

    /// Pass-band frequency (for Taubin only)
    #[arg(default_value_t = TAUBIN_DEFAULT_BAND, long, short = 'k', value_name = "FREQ")]
    pub pass_band: f64,

    /// Scaling parameter for all smoothing methods
    #[arg(default_value_t = TAUBIN_DEFAULT_SCALE, long, short, value_name = "SCALE")]
    pub scale: f64,

    /// Enables hierarchical smoothing
    #[arg(action, long, short = 'b')]
    pub hierarchical: bool,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

pub fn smooth(args: SmoothArgs) -> Result<(), ErrorWrapper> {
    let mut mesh = read_mesh(&args.input, args.quiet, true)?;
    apply_smoothing_method(
        &mut mesh,
        args.iterations,
        args.method,
        args.pass_band,
        args.scale,
        args.hierarchical,
        args.quiet,
    )?;
    if let Some(mode) = args.remeshing {
        mesh = apply_remeshing(mesh, mode, args.quiet)?;
    }
    if let Some(file) = args.metrics {
        write_metrics(&mesh, &file, args.quiet)?;
    }
    write_mesh(&args.output, mesh, args.quiet)
}

pub fn apply_smoothing_method(
    mesh: &mut Mesh<3>,
    iterations: usize,
    method: Option<String>,
    pass_band: f64,
    scale: f64,
    hierarchical: bool,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let time = Instant::now();
    let method = method.unwrap_or_else(|| "Taubin".to_string());
    let smoothing = match method.as_str() {
        "Laplacian" | "Laplace" | "laplacian" | "laplace" => {
            if !quiet {
                println!("   \x1b[1;96mSmoothing\x1b[0m with {iterations} iterations of Laplace");
            }
            Smoothing::Laplace {
                iterations,
                scale,
                weighting: Weighting::Uniform,
                preserve_boundary: hierarchical,
                preserve_interfaces: hierarchical,
            }
        }
        "Taubin" | "taubin" => {
            if !quiet {
                println!("   \x1b[1;96mSmoothing\x1b[0m with {iterations} iterations of Taubin");
            }
            Smoothing::Taubin {
                iterations,
                pass_band,
                scale,
                weighting: Weighting::Uniform,
                preserve_boundary: hierarchical,
                preserve_interfaces: hierarchical,
            }
        }
        _ => {
            return Err(ErrorWrapper::from(format!(
                "Invalid smoothing method {method} specified"
            )));
        }
    };
    mesh.smooth(smoothing);
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}
