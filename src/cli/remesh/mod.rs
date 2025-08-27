use super::{
    ErrorWrapper,
    input::read_finite_elements,
    output::write_finite_elements,
    smooth::{TAUBIN_DEFAULT_BAND, TAUBIN_DEFAULT_ITERS, TAUBIN_DEFAULT_SCALE},
};
use automesh::{FiniteElementMethods, Smoothing, TriangularFiniteElements};
use clap::Subcommand;
use std::time::Instant;

pub const REMESH_DEFAULT_ITERS: usize = 5;

#[derive(Subcommand, Debug)]
pub enum MeshRemeshCommands {
    /// Applies isotropic remeshing to the mesh before output
    Remesh {
        /// Number of remeshing iterations
        #[arg(default_value_t = REMESH_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },
}

pub fn remesh(
    input: String,
    output: String,
    iterations: usize,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let mut finite_elements =
        read_finite_elements::<_, TriangularFiniteElements>(&input, quiet, true)?;
    let time = Instant::now();
    if !quiet {
        println!("   \x1b[1;96mRemeshing\x1b[0m isotropically with {iterations} iterations")
    }
    finite_elements.node_element_connectivity()?;
    finite_elements.node_node_connectivity()?;
    finite_elements.remesh(
        iterations,
        &Smoothing::Taubin(
            TAUBIN_DEFAULT_ITERS,
            TAUBIN_DEFAULT_BAND,
            TAUBIN_DEFAULT_SCALE,
        ),
    );
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    write_finite_elements(output, finite_elements, quiet)
}
