use super::{
    ErrorWrapper,
    input::read_finite_elements,
    output::write_finite_elements,
    smooth::{TAUBIN_DEFAULT_BAND, TAUBIN_DEFAULT_ITERS, TAUBIN_DEFAULT_SCALE},
};
use automesh::{FiniteElementMethods, Smoothing, TriangularFiniteElements};
use clap::Subcommand;
use conspire::math::Tensor;
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
        read_finite_elements::<_, _, TriangularFiniteElements>(&input, quiet, true)?;
    apply_remeshing(&mut finite_elements, iterations, quiet, false)?;
    write_finite_elements(output, finite_elements, quiet)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_remeshing<const M: usize, const N: usize, T>(
    finite_elements: &mut T,
    iterations: usize,
    quiet: bool,
    smoothed: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N>,
{
    let time = Instant::now();
    if !quiet {
        println!("   \x1b[1;96mRemeshing\x1b[0m isotropically with {iterations} iterations")
    }
    if !smoothed {
        finite_elements.node_element_connectivity()?;
        finite_elements.node_node_connectivity()?;
    }
    finite_elements.remesh(
        iterations,
        &Smoothing::Taubin(
            TAUBIN_DEFAULT_ITERS,
            TAUBIN_DEFAULT_BAND,
            TAUBIN_DEFAULT_SCALE,
        ),
        None,
    );
    if !quiet {
        let mut blocks = finite_elements.get_element_blocks().clone();
        let elements = blocks.len();
        blocks.sort();
        blocks.dedup();
        println!(
            "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
            time.elapsed(),
            blocks.len(),
            elements,
            finite_elements.get_nodal_coordinates().len()
        );
    }
    Ok(())
}
