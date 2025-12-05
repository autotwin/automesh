use super::{
    ErrorWrapper,
    input::read_finite_elements,
    mesh::{MeshBasis, mesh_print_info},
    output::{invalid_output, write_finite_elements, write_segmentation},
};
use automesh::{FiniteElementMethods, Tessellation, Voxels};
use clap::Subcommand;
use conspire::math::Tensor;
use std::{path::Path, time::Instant};

#[derive(Subcommand)]
pub enum SegmentSubcommand {
    /// Segments an all-hexahedral mesh
    Hex(SegmentArgs),
    /// Segments an all-tetrahedral mesh
    Tet(SegmentArgs),
    /// Segments an all-triangular mesh
    Tri(SegmentArgs),
}

#[derive(clap::Args)]
pub struct SegmentArgs {
    /// Mesh input file (exo | inp)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Segmentation (npy | spn) or mesh (exo | inp) output file
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Grid length for sampling within each element
    #[arg(default_value_t = 1, long, short = 'g', value_name = "NUM")]
    pub grid: usize,

    /// Element size which is the side length
    #[arg(long, short = 's', value_name = "NUM")]
    pub size: f64,

    /// Block IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    pub remove: Option<Vec<usize>>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,
}

pub fn segment<const M1: usize, const N1: usize, T, const M2: usize, const N2: usize, U>(
    input: String,
    output: String,
    grid: usize,
    size: f64,
    remove: Option<Vec<usize>>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M1, N1> + From<Tessellation>,
    U: FiniteElementMethods<M2, N2> + From<Voxels>,
    Tessellation: From<U>,
{
    let finite_elements = read_finite_elements::<_, _, T>(&input, quiet, true)?;
    let mut time = Instant::now();
    if !quiet {
        println!("  \x1b[1;96mSegmenting\x1b[0m from finite elements")
    }
    let mut voxels = Voxels::from_finite_elements(finite_elements, grid, size);
    voxels.extend_removal(remove.into());
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
        time = Instant::now()
    }
    let extension = Path::new(&output).extension().and_then(|ext| ext.to_str());
    match extension {
        Some("exo") | Some("inp") => {
            if !quiet {
                print!("     \x1b[1;96mMeshing\x1b[0m voxels into hexahedra");
                mesh_print_info(
                    MeshBasis::Voxels,
                    voxels.get_scale(),
                    voxels.get_translate(),
                )
            }
            let finite_elements = U::from(voxels);
            if !quiet {
                let mut blocks = finite_elements.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    finite_elements.get_nodal_coordinates().len()
                );
            }
            write_finite_elements(output, finite_elements, quiet)
        }
        Some("npy") | Some("spn") => write_segmentation(output, voxels, quiet),
        _ => Err(invalid_output(&output, extension)),
    }
}
