use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
    metrics::write_metrics,
};
use conspire::geometry::mesh::{Stop, Tessellation};
use std::{num::NonZeroUsize, path::Path, time::Instant};

const SWEEPS: NonZeroUsize = NonZeroUsize::new(20).unwrap();

#[derive(clap::Args)]
pub struct ImproveArgs {
    /// Mesh input file (exo | inp | mesh | vtu)
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

    /// Maximum number of minimizer sweeps at each quality threshold
    #[arg(default_value_t = SWEEPS, long, value_name = "NUM")]
    pub sweeps: NonZeroUsize,
}

pub fn improve(args: ImproveArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mut mesh = read_mesh(&args.input, quiet)?;
    let target = Tessellation::try_from(Path::new(&args.target))?;
    let time = Instant::now();
    crate::echo!(
        quiet,
        "   \x1b[1;96mImproving\x1b[0m the minimum scaled Jacobian"
    );
    let stop = match mesh.improve_scaled_jacobian(&target, args.sweeps.get())? {
        Stop::Budget { theta, sweeps } => format!(
            "stopped at Θ = {:.2}: sweep budget ({sweeps} {}) used up",
            theta.value(),
            plural(sweeps)
        ),
        Stop::Stall { theta, sweeps } => format!(
            "stopped at Θ = {:.2}: stalled after {sweeps} {}, energy decrease below 1e-6 relative",
            theta.value(),
            plural(sweeps)
        ),
    };
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{stop}]\x1b[0m",
        time.elapsed()
    );
    if let Some(file) = args.metrics {
        write_metrics(&mesh, &file, quiet)?;
    }
    write_mesh(&args.output, mesh, quiet)
}

fn plural(sweeps: usize) -> &'static str {
    if sweeps == 1 { "sweep" } else { "sweeps" }
}
