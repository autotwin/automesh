use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
};
use clap::Subcommand;
use conspire::geometry::mesh::{Mesh, Remeshing};
use std::time::Instant;

pub const REMESH_DEFAULT_ITERS: usize = 5;

#[derive(Subcommand, Debug)]
pub enum MeshRemeshCommands {
    /// Applies isotropic remeshing to the mesh before output
    Remesh {
        /// Number of remeshing iterations
        #[arg(default_value_t = REMESH_DEFAULT_ITERS, long, short = 'n', value_name = "NUM")]
        iterations: usize,

        /// Target edge length [default: mean edge length of the input mesh]
        #[arg(long, short = 's', value_name = "SIZE")]
        size: Option<f64>,

        /// Pass to quiet the terminal output
        #[arg(action, long, short)]
        quiet: bool,
    },
}

pub fn remesh(
    input: String,
    output: String,
    iterations: usize,
    size: Option<f64>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let mesh = read_mesh(&input, quiet, true)?;
    let mesh = apply_remeshing(mesh, iterations, size, quiet)?;
    write_mesh(&output, mesh, quiet)
}

pub fn apply_remeshing(
    mesh: Mesh<3>,
    iterations: usize,
    size: Option<f64>,
    quiet: bool,
) -> Result<Mesh<3>, ErrorWrapper> {
    let time = Instant::now();
    if !quiet {
        match size {
            Some(length) => println!(
                "   \x1b[1;96mRemeshing\x1b[0m isotropically with {iterations} iterations \
                (target edge length {length})"
            ),
            None => {
                println!("   \x1b[1;96mRemeshing\x1b[0m isotropically with {iterations} iterations")
            }
        }
    }
    let mesh = mesh.remesh(Remeshing::Isotropic {
        iterations,
        length: size,
    })?;
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
