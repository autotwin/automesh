use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh},
    metrics::write_metrics,
};
use conspire::{
    geometry::mesh::{Fitting, Freedom, MeshError, Tessellation},
    units::Dimensionless,
};
use std::{path::Path, time::Instant};

pub const DEFAULT_SWEEPS: usize = 20;

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

    /// Minimum scaled Jacobian to reach, stopping once it is met
    /// [default: as high as the sweeps allow]
    #[arg(long, value_name = "MSJ")]
    pub threshold: Option<f64>,

    /// Number of sweeps allowed at each level of the minimum scaled Jacobian
    #[arg(default_value_t = DEFAULT_SWEEPS, long, value_name = "NUM")]
    pub sweeps: usize,

    /// Snaps the boundary onto the target first, which a boundary that is
    /// not already on the target (a soft fit) requires
    #[arg(action, long)]
    pub snap: bool,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,
}

pub fn improve(args: ImproveArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mut mesh = read_mesh(&args.input, quiet)?;
    let target = Tessellation::try_from(Path::new(&args.target))?;
    if args.snap {
        let time = Instant::now();
        crate::echo!(
            quiet,
            "    \x1b[1;96mSnapping\x1b[0m the boundary onto the target"
        );
        mesh.inflate(&target, Freedom::Shell, Fitting::Snap)?;
        crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    let time = Instant::now();
    crate::echo!(
        quiet,
        "   \x1b[1;96mImproving\x1b[0m the minimum scaled Jacobian"
    );
    let outcome = mesh.improve_scaled_jacobian(
        &target,
        Dimensionless::of(args.threshold.unwrap_or(1.0)),
        args.sweeps,
    );
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    let unmet = match outcome {
        Ok(()) => None,
        Err(error @ (MeshError::Budget { .. } | MeshError::Stall { .. })) => {
            args.threshold.map(|_| error)
        }
        Err(error) => return Err(ErrorWrapper::from(error.to_string())),
    };
    if let Some(file) = args.metrics {
        write_metrics(&mesh, &file, quiet)?;
    }
    write_mesh(&args.output, mesh, quiet)?;
    unmet.map_or(Ok(()), |error| Err(ErrorWrapper::from(error.to_string())))
}
