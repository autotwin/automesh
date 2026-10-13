use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
    metrics::write_metrics,
};
use conspire::geometry::mesh::Tessellation;
use std::{path::Path, time::Instant};

#[derive(clap::Args)]
pub struct ImproveArgs {
    /// Hexahedral mesh input file (exo | inp | mesh | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Improved mesh output file (exo | inp | mesh | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Target surface the boundary must keep fitting (stl)
    #[arg(long, short, value_name = "FILE")]
    pub target: String,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,
}

pub fn improve(args: ImproveArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mut mesh = read_mesh(&args.input, quiet)?;
    let target = Tessellation::try_from(Path::new(&args.target))?;
    let time = Instant::now();
    crate::echo!(
        quiet,
        "   \x1b[1;96mImproving\x1b[0m the scaled Jacobian of the hexahedra"
    );
    mesh.improve_scaled_jacobian(&target)?;
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    if let Some(file) = args.metrics {
        write_metrics(&mesh, &file, quiet)?;
    }
    write_mesh(&args.output, mesh, quiet)
}
