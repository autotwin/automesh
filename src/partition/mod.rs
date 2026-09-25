use super::{
    ErrorWrapper,
    io::{read_mesh, write_mesh_threads},
};
use clap::{Args, ValueEnum};
use conspire::geometry::mesh::{Mesh, Partition};
use std::time::Instant;

/// Parsed by clap, so a misspelled method fails before any work starts.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "UPPER")]
pub enum PartitionMethod {
    /// Recursive coordinate bisection
    Rcb,
    /// Recursive inertial bisection
    Rib,
    /// Regular box divisions of the bounding box
    Box,
}

#[derive(Args)]
pub struct PartitionOptions {
    /// Partitioning method
    #[arg(
        default_value_t = PartitionMethod::Rcb,
        ignore_case = true,
        long,
        short,
        value_enum,
        value_name = "NAME"
    )]
    pub method: PartitionMethod,

    /// Number of parts (RCB | RIB)
    #[arg(long, short = 'n', value_name = "NUM")]
    pub parts: Option<usize>,

    /// Number of divisions in each direction (BOX)
    #[arg(long, num_args = 3, short, value_delimiter = ' ', value_names = ["NX", "NY", "NZ"])]
    pub divisions: Option<Vec<usize>>,

    /// Number of threads used to write the output
    #[arg(long, short = 'j', value_name = "NUM")]
    pub threads: Option<usize>,
}

impl PartitionOptions {
    pub fn threads(&self) -> Result<usize, ErrorWrapper> {
        match self.threads {
            Some(0) => Err(ErrorWrapper::from("Threads must be positive")),
            Some(threads) => Ok(threads),
            None => Ok(std::thread::available_parallelism().map_or(1, |threads| threads.get())),
        }
    }

    /// Partitions the mesh, reporting the method and number of parts.
    pub fn split(&self, mesh: &Mesh<3>, quiet: bool) -> Result<Partition, ErrorWrapper> {
        let time = Instant::now();
        let partition = match self.method {
            PartitionMethod::Box => {
                if self.parts.is_some() {
                    return Err(ErrorWrapper::from(
                        "Parts (-n) applies to RCB and RIB, use divisions (-d) for BOX",
                    ));
                }
                let divisions = self
                    .divisions
                    .as_ref()
                    .ok_or_else(|| ErrorWrapper::from("BOX requires divisions (-d NX NY NZ)"))?;
                if divisions.contains(&0) {
                    return Err(ErrorWrapper::from("Divisions must be positive"));
                }
                mesh.partition_box([divisions[0], divisions[1], divisions[2]])
            }
            method => {
                if self.divisions.is_some() {
                    return Err(ErrorWrapper::from(
                        "Divisions (-d) applies to BOX, use parts (-n) for RCB and RIB",
                    ));
                }
                let parts = self
                    .parts
                    .ok_or_else(|| ErrorWrapper::from("RCB and RIB require parts (-n NUM)"))?;
                if parts == 0 || parts > mesh.number_of_elements() {
                    return Err(ErrorWrapper::from(format!(
                        "Parts must be between 1 and the number of elements ({})",
                        mesh.number_of_elements()
                    )));
                }
                match method {
                    PartitionMethod::Rib => mesh.partition_rib(parts),
                    _ => mesh.partition_rcb(parts),
                }
            }
        };
        crate::echo!(
            quiet,
            "   \x1b[1;96mSplitting\x1b[0m using {} [{} parts]",
            format!("{:?}", self.method).to_uppercase(),
            partition.number_of_parts()
        );
        crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
        Ok(partition)
    }
}

#[derive(Args)]
pub struct PartitionArgs {
    /// Mesh input file (exo | inp | mesh | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Partitioned mesh output file (exo | inp | mesh | vtu), with each part split into its own element block(s)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    #[command(flatten)]
    pub options: PartitionOptions,
}

pub fn partition(args: PartitionArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let threads = args.options.threads()?;
    let mesh = read_mesh(&args.input, quiet)?;
    let partition = args.options.split(&mesh, quiet)?;
    write_mesh_threads(&args.output, partition.blocked_mesh(&mesh), threads, quiet)
}
