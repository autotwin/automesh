use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
    metrics::write_metrics,
};
use conspire::{
    geometry::mesh::{MeshError, Tessellation},
    units::Dimensionless,
};
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

    /// Minimum scaled Jacobian at which to stop, from 0 to 1
    #[arg(default_value_t = 1.0, long, value_name = "SJ", value_parser = threshold)]
    pub threshold: f64,
}

fn threshold(value: &str) -> Result<f64, String> {
    let threshold = value
        .parse::<f64>()
        .map_err(|_| format!("{value} is not a number"))?;
    if (0.0..=1.0).contains(&threshold) {
        Ok(threshold)
    } else {
        Err(format!("{threshold} is not between 0 and 1"))
    }
}

pub fn improve(args: ImproveArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mut mesh = read_mesh(&args.input, quiet)?;
    let target = Tessellation::try_from(Path::new(&args.target))?;
    let time = Instant::now();
    crate::echo!(
        quiet,
        "   \x1b[1;96mImproving\x1b[0m the minimum scaled Jacobian"
    );
    let stop = match mesh.improve_scaled_jacobian(
        &target,
        Dimensionless::of(args.threshold),
        args.sweeps.get(),
    ) {
        Ok(()) => None,
        Err(error @ (MeshError::Budget { .. } | MeshError::Stall { .. })) => Some(error),
        Err(error) => return Err(error.to_string().into()),
    };
    match stop {
        None => crate::echo!(
            quiet,
            "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[reached the threshold {:.2}]\x1b[0m",
            time.elapsed(),
            args.threshold
        ),
        Some(error) => {
            crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
            crate::echo!(quiet, "     \x1b[1;93mStopped\x1b[0m {error}");
        }
    }
    if let Some(file) = args.metrics {
        write_metrics(&mesh, &file, quiet)?;
    }
    write_mesh(&args.output, mesh, quiet)
}
