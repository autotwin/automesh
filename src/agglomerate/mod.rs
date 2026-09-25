use super::{
    ErrorWrapper,
    io::{extension, invalid_output, read_mesh, write_mesh_threads},
    partition::PartitionOptions,
};
use clap::Args;
use std::time::Instant;

#[derive(Args)]
pub struct AgglomerateArgs {
    /// Mesh input file (exo | inp | mesh | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Agglomerated mesh output file (exo | vtu), with each part as one polyhedral element
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    #[command(flatten)]
    pub options: PartitionOptions,
}

pub fn agglomerate(args: AgglomerateArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    match extension(&args.output) {
        Some("exo" | "vtu") => {}
        other => return Err(invalid_output(&args.output, other)),
    }
    let threads = args.options.threads()?;
    let mesh = read_mesh(&args.input, quiet)?;
    let partition = args.options.split(&mesh, quiet)?;
    crate::echo!(
        quiet,
        "   \x1b[1;96mAgglomerating\x1b[0m [{} polyhedra]",
        partition.number_of_parts()
    );
    let time = Instant::now();
    let agglomerated = partition
        .agglomerated_mesh(&mesh)
        .map_err(ErrorWrapper::from)?;
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    write_mesh_threads(&args.output, agglomerated, threads, quiet)
}
