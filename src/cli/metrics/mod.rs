use clap::Subcommand;

#[derive(Subcommand)]
pub enum MetricsSubcommand {
    /// Quality metrics for an all-hexahedral finite element mesh
    Hex(MetricsArgs),
    /// Quality metrics for an all-tetrahedral finite element mesh
    Tet(MetricsArgs),
    /// Quality metrics for an all-triangular finite element mesh
    Tri(MetricsArgs),
}

#[derive(clap::Args)]
pub struct MetricsArgs {
    /// Mesh input file (exo | inp | stl)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Quality metrics output file (csv | npy)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}
