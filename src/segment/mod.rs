use super::{
    ErrorWrapper,
    io::{extension, invalid_output, read_mesh, write_mesh, write_segmentation},
};
use conspire::geometry::{grid::Voxels, mesh::Mesh};
use std::time::Instant;

#[derive(clap::Args)]
pub struct SegmentArgs {
    /// Mesh input file (exo | inp | stl | vtu)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Segmentation (npy | spn | vti) or mesh (exo | inp | mesh | vtu) output file
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Grid length for sampling within each element (currently unused)
    #[arg(default_value_t = 1, long, short = 'g', value_name = "NUM")]
    pub grid: usize,

    /// Element size which is the side length
    #[arg(long, short = 's', value_name = "NUM")]
    pub size: f64,

    /// Block IDs to remove from the segmentation
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    pub remove: Option<Vec<usize>>,
}

pub fn segment(args: SegmentArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mesh = read_mesh(&args.input, quiet, true)?;
    let time = Instant::now();
    crate::echo!(quiet, "  \x1b[1;96mSegmenting\x1b[0m from finite elements");
    let voxels = Voxels::<usize>::from_finite_elements(&mesh, args.size);
    let nel = *voxels.nel();
    let remove = args.remove.unwrap_or_default();
    let data: Vec<u8> = voxels
        .data()
        .iter()
        .map(|&block| {
            if remove.contains(&block) {
                0
            } else {
                block as u8
            }
        })
        .collect();
    let voxels = Voxels::<u8>::new(data, nel);
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    match extension(&args.output) {
        Some("npy") | Some("spn") | Some("vti") => write_segmentation(&args.output, &voxels, quiet),
        Some("exo") | Some("inp") | Some("mesh") | Some("vtu") => {
            let mesh = Mesh::from_voxels(voxels, Some(&[0u8]));
            write_mesh(&args.output, mesh, quiet)
        }
        extension => Err(invalid_output(&args.output, extension)),
    }
}
