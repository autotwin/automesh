use super::ErrorWrapper;

#[derive(clap::Args)]
pub struct HausdorffArgs {
    /// Mesh input file (exo | inp | mesh | vtu)
    #[arg(long, short = 'a', value_name = "FILE")]
    pub mesh: String,

    /// Tessellation input file (stl)
    #[arg(long, short = 'b', value_name = "FILE")]
    pub tessellation: String,

    /// Output prefix; any extension is dropped
    #[arg(long, short, value_name = "PREFIX")]
    pub output: String,
}

pub fn hausdorff(_args: HausdorffArgs, _quiet: bool) -> Result<(), ErrorWrapper> {
    Err("automesh hausdorff is not implemented yet (see #761)".into())
}
