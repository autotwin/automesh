use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
};
use clap::Subcommand;
use conspire::geometry::mesh::{Mesh, Remeshing};
use std::time::Instant;

pub const REMESH_DEFAULT_ITERS: usize = 5;
pub const ADAPTIVE_DEFAULT_TOLERANCE: f64 = 0.1;
pub const ADAPTIVE_DEFAULT_GRADATION: f64 = 0.5;

#[derive(Subcommand, Debug)]
pub enum MeshRemeshCommands {
    /// Uniform target edge length over the whole mesh
    Uniform {
        /// Number of remeshing iterations
        #[arg(default_value_t = REMESH_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Target edge length [default: mean edge length]
        #[arg(long, short = 's', value_name = "SIZE")]
        size: Option<f64>,
    },

    /// Curvature-adaptive target edge length
    Adaptive {
        /// Number of remeshing iterations
        #[arg(default_value_t = REMESH_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Minimum edge length
        #[arg(long, value_name = "MIN")]
        minimum: f64,

        /// Maximum edge length
        #[arg(long, value_name = "MAX")]
        maximum: f64,

        /// Curvature tolerance
        #[arg(default_value_t = ADAPTIVE_DEFAULT_TOLERANCE, long, short = 't', value_name = "TOL")]
        tolerance: f64,

        /// Size gradation factor
        #[arg(default_value_t = ADAPTIVE_DEFAULT_GRADATION, long, short = 'g', value_name = "GRAD")]
        gradation: f64,
    },
}

pub fn remesh(
    input: String,
    output: String,
    mode: Option<MeshRemeshCommands>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let mesh = read_mesh(&input, quiet, true)?;
    let mode = mode.unwrap_or(MeshRemeshCommands::Uniform {
        iterations: REMESH_DEFAULT_ITERS,
        size: None,
    });
    let mesh = apply_remeshing(mesh, mode, quiet)?;
    write_mesh(&output, mesh, quiet)
}

pub fn apply_remeshing(
    mesh: Mesh<3>,
    mode: MeshRemeshCommands,
    quiet: bool,
) -> Result<Mesh<3>, ErrorWrapper> {
    let time = Instant::now();
    let remeshing = match mode {
        MeshRemeshCommands::Uniform { iterations, size } => {
            if !quiet {
                match size {
                    Some(length) => println!(
                        "   \x1b[1;96mRemeshing\x1b[0m with {iterations} iterations of uniform sizing \
                        (target edge length {length})"
                    ),
                    None => println!(
                        "   \x1b[1;96mRemeshing\x1b[0m with {iterations} iterations of uniform sizing"
                    ),
                }
            }
            Remeshing::Uniform {
                iterations,
                length: size,
            }
        }
        MeshRemeshCommands::Adaptive {
            iterations,
            minimum,
            maximum,
            tolerance,
            gradation,
        } => {
            if !quiet {
                println!(
                    "   \x1b[1;96mRemeshing\x1b[0m with {iterations} iterations of adaptive sizing \
                    (edge length {minimum}\u{2013}{maximum}, tolerance {tolerance}, gradation {gradation})"
                );
            }
            Remeshing::Adaptive {
                iterations,
                tolerance,
                minimum,
                maximum,
                gradation,
            }
        }
    };
    let mesh = mesh.remesh(remeshing)?;
    if !quiet {
        println!(
            "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} elements, {} nodes]\x1b[0m",
            time.elapsed(),
            mesh.number_of_elements(),
            mesh.number_of_nodes()
        );
    }
    Ok(mesh)
}
