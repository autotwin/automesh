use super::{
    ErrorWrapper,
    io::{extension, invalid_output, read_mesh, write_mesh_threads},
};
use clap::Args;
use conspire::{
    geometry::mesh::Merging,
    units::Time,
    vem::{
        agglomerate::{Agglomeration, Candidates, Reference},
        block::element::DEFAULT_STABILIZATION,
    },
};
use std::{thread::available_parallelism, time::Instant};

#[derive(Args)]
pub struct AgglomerateArgs {
    /// Mesh input file (exo | inp | mesh | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Agglomerated mesh output file (exo | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Time scale an element is expected to have [default: the median over the elements]
    #[arg(long, short, value_name = "TIME")]
    pub reference: Option<f64>,

    /// Factor below the reference time scale at which an element is joined to a neighbor
    #[arg(long, short, value_name = "FACTOR")]
    pub step_reduction: f64,

    /// Factor by which a join must improve on the worse of the two time scales
    #[arg(long, short = 'm', value_name = "FACTOR")]
    pub minimum_improvement: f64,

    /// Smallest volume of a part of a joined element, relative to the mean of its parts
    #[arg(default_value_t = 0.01, long, value_name = "FRACTION")]
    pub minimum_volume: f64,

    /// Maximum number of passes over the elements
    #[arg(default_value_t = 5, long, short, value_name = "NUM")]
    pub passes: usize,

    /// Poisson ratio of the material used to compute time scales
    #[arg(default_value_t = 0.3, long, value_name = "RATIO")]
    pub poisson: f64,

    /// Weight of the part of the stiffness taken from the tetrahedra of the element
    #[arg(default_value_t = DEFAULT_STABILIZATION, long, value_name = "WEIGHT")]
    pub stabilization: f64,

    /// Number of threads used to write the output
    #[arg(long, short = 'j', value_name = "NUM")]
    pub threads: Option<usize>,
}

impl AgglomerateArgs {
    fn threads(&self) -> Result<usize, ErrorWrapper> {
        match self.threads {
            Some(0) => Err(ErrorWrapper::from("Threads must be positive")),
            Some(threads) => Ok(threads),
            None => Ok(available_parallelism().map_or(1, |threads| threads.get())),
        }
    }
    fn validate(&self) -> Result<(), ErrorWrapper> {
        match extension(&self.output) {
            Some("exo") | Some("vtu") => {}
            other => return Err(invalid_output(&self.output, other)),
        }
        if self.reference.is_some_and(|reference| reference <= 0.0) {
            return Err(ErrorWrapper::from("Reference must be positive"));
        }
        if self.step_reduction <= 1.0 {
            return Err(ErrorWrapper::from(
                "Step reduction must be greater than one",
            ));
        }
        if self.minimum_improvement <= 1.0 {
            return Err(ErrorWrapper::from(
                "Minimum improvement must be greater than one",
            ));
        }
        if self.minimum_volume < 0.0 {
            return Err(ErrorWrapper::from("Minimum volume must not be negative"));
        }
        if self.passes == 0 {
            return Err(ErrorWrapper::from("Passes must be positive"));
        }
        if self.poisson <= -1.0 || self.poisson >= 0.5 {
            return Err(ErrorWrapper::from(
                "Poisson ratio must be between -1 and 0.5",
            ));
        }
        if !(0.0..=1.0).contains(&self.stabilization) {
            return Err(ErrorWrapper::from("Stabilization must be between 0 and 1"));
        }
        Ok(())
    }
}

pub fn agglomerate(args: AgglomerateArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    args.validate()?;
    let threads = args.threads()?;
    let mesh = read_mesh(&args.input, quiet)?;
    crate::echo!(
        quiet,
        "   \x1b[1;96mAgglomerating\x1b[0m [{} elements]",
        mesh.number_of_elements()
    );
    let time = Instant::now();
    let candidates = Candidates::from_mesh(&mesh, args.poisson, args.stabilization)
        .map_err(ErrorWrapper::from)?;
    let agglomerated = candidates
        .agglomerate(&Agglomeration {
            reference: match args.reference {
                Some(reference) => Reference::Value(Time::seconds(reference)),
                None => Reference::Median,
            },
            step_reduction: args.step_reduction,
            minimum_volume: args.minimum_volume,
            merging: Merging {
                minimum_improvement: args.minimum_improvement,
                passes: args.passes,
            },
        })
        .map_err(ErrorWrapper::from)?;
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} [{} polyhedra, {} unresolved]",
        time.elapsed(),
        agglomerated.time_scales().len(),
        agglomerated.merged.unresolved.len()
    );
    let mesh = agglomerated
        .merged
        .mesh(&mesh)
        .map_err(ErrorWrapper::from)?;
    write_mesh_threads(&args.output, mesh, threads, quiet)
}
