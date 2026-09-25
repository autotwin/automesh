use super::{
    ErrorWrapper,
    io::{extension, invalid_output, read_mesh, write_exodus},
};
use clap::{Args, ValueEnum};
use conspire::geometry::mesh::{Mesh, Partition};
use std::{sync::Mutex, thread, time::Instant};

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
            (0..partition.number_of_parts())
                .filter(|&part| !partition.part_elements(part).is_empty())
                .count()
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

    /// Partitioned mesh output file (exo), written as one file per part named FILE.PARTS.RANK
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    #[command(flatten)]
    pub options: PartitionOptions,
}

pub fn partition(args: PartitionArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    match extension(&args.output) {
        Some("exo") => {}
        other => return Err(invalid_output(&args.output, other)),
    }
    let threads = args.options.threads()?;
    let mesh = read_mesh(&args.input, quiet)?;
    let partition = args.options.split(&mesh, quiet)?;
    let parts = (0..partition.number_of_parts())
        .filter(|&part| !partition.part_elements(part).is_empty())
        .collect::<Vec<_>>();
    let width = parts.len().to_string().len();
    let jobs = Mutex::new(
        parts
            .iter()
            .enumerate()
            .map(|(rank, &part)| {
                (
                    format!("{}.{}.{rank:0width$}", args.output, parts.len()),
                    partition.part(&mesh, part).0,
                )
            })
            .collect::<Vec<_>>()
            .into_iter(),
    );
    crate::echo!(
        quiet,
        "     \x1b[1;96mWriting\x1b[0m {}.{}.* [{} files]",
        args.output,
        parts.len(),
        parts.len()
    );
    let time = Instant::now();
    let errors = Mutex::new(Vec::new());
    thread::scope(|scope| {
        (0..threads.min(parts.len())).for_each(|_| {
            scope.spawn(|| {
                loop {
                    let job = jobs.lock().unwrap().next();
                    let Some((file, part)) = job else { break };
                    if let Err(error) = write_exodus(&file, part, 1) {
                        errors.lock().unwrap().push(error)
                    }
                }
            });
        })
    });
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    match errors.into_inner().unwrap().into_iter().next() {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
