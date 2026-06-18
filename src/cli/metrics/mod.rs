use super::{ErrorWrapper, io::read_mesh};
use clap::Subcommand;
use conspire::{
    geometry::mesh::{Mesh, Verdict},
    io::{Npy, Write},
};
use std::{
    fs::File,
    io::{BufWriter, Write as WriteIO},
    time::Instant,
};

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
    /// Mesh input file (exo | inp | stl | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Quality metrics output file (csv | npy)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

pub fn metrics(args: MetricsArgs) -> Result<(), ErrorWrapper> {
    let mesh = read_mesh(&args.input, args.quiet, true)?;
    write_metrics(&mesh, &args.output, args.quiet)
}

fn flatten(metric: Vec<Vec<f64>>) -> Vec<f64> {
    metric.into_iter().flatten().collect()
}

pub fn write_metrics(mesh: &Mesh<3>, file: &str, quiet: bool) -> Result<(), ErrorWrapper> {
    if !quiet {
        println!("     \x1b[1;96mMetrics\x1b[0m {file}");
    }
    let time = Instant::now();
    let maximum_edge_ratios = flatten(mesh.maximum_edge_ratios());
    let minimum_scaled_jacobians = flatten(mesh.minimum_scaled_jacobians());
    let maximum_skews = flatten(mesh.maximum_skews());
    let volumes = flatten(mesh.volumes());
    let extension = super::io::extension(file);
    match extension {
        Some("csv") => {
            let mut writer = BufWriter::new(File::create(file)?);
            writer.write_all(
                b"maximum edge ratio,minimum scaled jacobian,maximum skew,element volume\n",
            )?;
            for (((ratio, jacobian), skew), volume) in maximum_edge_ratios
                .iter()
                .zip(minimum_scaled_jacobians.iter())
                .zip(maximum_skews.iter())
                .zip(volumes.iter())
            {
                writer.write_all(
                    format!("{ratio:>10.6e},{jacobian:>10.6e},{skew:>10.6e},{volume:>10.6e}\n")
                        .as_bytes(),
                )?;
            }
            writer.flush()?;
        }
        Some("npy") => {
            let rows = maximum_edge_ratios.len();
            let mut data = Vec::with_capacity(rows * 4);
            for (((ratio, jacobian), skew), volume) in maximum_edge_ratios
                .iter()
                .zip(minimum_scaled_jacobians.iter())
                .zip(maximum_skews.iter())
                .zip(volumes.iter())
            {
                data.push(*ratio);
                data.push(*jacobian);
                data.push(*skew);
                data.push(*volume);
            }
            Npy {
                data,
                shape: vec![rows, 4],
                fortran_order: false,
            }
            .write(file)?;
        }
        _ => {
            return Err(ErrorWrapper::from(format!(
                "Unsupported metrics extension .{} (use csv or npy)",
                extension.unwrap_or("UNDEFINED")
            )));
        }
    }
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}
