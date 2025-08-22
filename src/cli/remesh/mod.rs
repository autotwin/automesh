use clap::Subcommand;

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
