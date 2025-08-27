use super::{
    ErrorWrapper,
    input::read_segmentation,
    output::{write_finite_elements, write_metrics},
    remesh::MeshRemeshCommands,
    smooth::{
        MeshSmoothCommands, TAUBIN_DEFAULT_BAND, TAUBIN_DEFAULT_ITERS, TAUBIN_DEFAULT_SCALE,
        apply_smoothing_method,
    },
};
use automesh::{
    FiniteElementMethods, HEX, HexahedralFiniteElements, Octree, Remove, Scale, Smoothing, TET,
    TRI, TetrahedralFiniteElements, Translate, TriangularFiniteElements,
};
use clap::Subcommand;
use conspire::math::TensorVec;
use std::time::Instant;

#[derive(Subcommand)]
pub enum MeshSubcommand {
    /// Creates an all-hexahedral mesh from a segmentation
    Hex(MeshHexArgs),
    /// Creates an all-tetrahedral mesh from a segmentation
    Tet(MeshTetArgs),
    /// Creates all-triangular isosurface(s) from a segmentation
    Tri(MeshTriArgs),
}

#[derive(clap::Args)]
pub struct MeshHexArgs {
    #[command(subcommand)]
    pub smoothing: Option<MeshSmoothCommands>,

    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Defeature clusters with less than NUM voxels
    #[arg(long, short, value_name = "NUM")]
    pub defeature: Option<usize>,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    pub nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    pub nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    pub nelz: Option<usize>,

    /// Voxel IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    pub remove: Option<Vec<usize>>,

    /// Scaling (> 0.0) in the x-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub xscale: f64,

    /// Scaling (> 0.0) in the y-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub yscale: f64,

    /// Scaling (> 0.0) in the z-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub zscale: f64,

    /// Translation in the x-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub xtranslate: f64,

    /// Translation in the y-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub ytranslate: f64,

    /// Translation in the z-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub ztranslate: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,

    /// Pass to mesh adaptively
    #[arg(action, hide = true, long)]
    pub adapt: bool,
}

#[derive(clap::Args)]
pub struct MeshTetArgs {
    #[command(subcommand)]
    pub smoothing: Option<MeshSmoothCommands>,

    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Mesh output file (exo | inp | mesh | vtk)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Defeature clusters with less than NUM voxels
    #[arg(long, short, value_name = "NUM")]
    pub defeature: Option<usize>,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    pub nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    pub nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    pub nelz: Option<usize>,

    /// Voxel IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    pub remove: Option<Vec<usize>>,

    /// Scaling (> 0.0) in the x-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub xscale: f64,

    /// Scaling (> 0.0) in the y-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub yscale: f64,

    /// Scaling (> 0.0) in the z-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub zscale: f64,

    /// Translation in the x-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub xtranslate: f64,

    /// Translation in the y-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub ytranslate: f64,

    /// Translation in the z-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub ztranslate: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,

    /// Pass to mesh adaptively
    #[arg(action, hide = true, long)]
    pub adapt: bool,
}

#[derive(clap::Args)]
pub struct MeshTriArgs {
    #[command(subcommand)]
    pub smoothing: Option<MeshSmoothCommands>,

    /// Segmentation input file (npy | spn)
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Mesh output file (exo | inp | mesh | stl | vtk)
    #[arg(long, short, value_name = "FILE")]
    pub output: String,

    /// Defeature clusters with less than NUM voxels
    #[arg(long, short, value_name = "NUM")]
    pub defeature: Option<usize>,

    /// Number of voxels in the x-direction (spn)
    #[arg(long, short = 'x', value_name = "NEL")]
    pub nelx: Option<usize>,

    /// Number of voxels in the y-direction (spn)
    #[arg(long, short = 'y', value_name = "NEL")]
    pub nely: Option<usize>,

    /// Number of voxels in the z-direction (spn)
    #[arg(long, short = 'z', value_name = "NEL")]
    pub nelz: Option<usize>,

    /// Voxel IDs to remove from the mesh
    #[arg(long, num_args = 1.., short, value_delimiter = ' ', value_name = "ID")]
    pub remove: Option<Vec<usize>>,

    /// Scaling (> 0.0) in the x-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub xscale: f64,

    /// Scaling (> 0.0) in the y-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub yscale: f64,

    /// Scaling (> 0.0) in the z-direction, applied before translation
    #[arg(default_value_t = 1.0, long, value_name = "SCALE")]
    pub zscale: f64,

    /// Translation in the x-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub xtranslate: f64,

    /// Translation in the y-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub ytranslate: f64,

    /// Translation in the z-direction
    #[arg(
        long,
        default_value_t = 0.0,
        allow_negative_numbers = true,
        value_name = "VAL"
    )]
    pub ztranslate: f64,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,

    /// Pass to quiet the terminal output
    #[arg(action, long, short)]
    pub quiet: bool,

    /// Pass to mesh adaptively
    #[arg(action, hide = true, long)]
    pub adapt: bool,
}

pub enum MeshBasis {
    Leaves,
    Surfaces,
    Voxels,
}

#[allow(clippy::too_many_arguments)]
pub fn mesh<const N: usize>(
    smoothing: Option<MeshSmoothCommands>,
    input: String,
    output: String,
    defeature: Option<usize>,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    remove: Option<Vec<usize>>,
    xscale: f64,
    yscale: f64,
    zscale: f64,
    xtranslate: f64,
    ytranslate: f64,
    ztranslate: f64,
    metrics: Option<String>,
    quiet: bool,
    adapt: bool,
) -> Result<(), ErrorWrapper> {
    let mut time = Instant::now();
    let scale_temporary = Scale::from([xscale, yscale, zscale]);
    let translate_temporary = Translate::from([xtranslate, ytranslate, ztranslate]);
    let remove = Remove::from(remove);
    let scale = Scale::from([xscale, yscale, zscale]);
    let translate = Translate::from([xtranslate, ytranslate, ztranslate]);
    let mut input_type = read_segmentation(
        input, nelx, nely, nelz, remove, scale, translate, quiet, true,
    )?;
    match N {
        HEX => {
            if let Some(min_num_voxels) = defeature {
                if !quiet {
                    time = Instant::now();
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less",
                    );
                }
                input_type = input_type.defeature(min_num_voxels);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
            }
            if !quiet {
                time = Instant::now();
            }
            let mut output_type: HexahedralFiniteElements = if adapt {
                if !quiet {
                    print!("     \x1b[1;96mMeshing\x1b[0m adaptive hexahedra");
                    mesh_print_info(MeshBasis::Voxels, &scale_temporary, &translate_temporary)
                }
                let mut tree = Octree::from(input_type);
                tree.balance_and_pair(true);
                tree.into()
            } else {
                if !quiet {
                    print!("     \x1b[1;96mMeshing\x1b[0m voxels into hexahedra");
                    mesh_print_info(MeshBasis::Voxels, &scale_temporary, &translate_temporary)
                }
                input_type.into()
            };
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    output_type.get_nodal_coordinates().len()
                );
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
                        remeshing: _,
                        iterations,
                        method,
                        hierarchical,
                        pass_band,
                        scale,
                    } => {
                        apply_smoothing_method(
                            &mut output_type,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&output_type, file, quiet)?
            }
            write_finite_elements(output, output_type, quiet)?;
        }
        TET => {
            if let Some(min_num_voxels) = defeature {
                if !quiet {
                    time = Instant::now();
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less"
                    );
                }
                input_type = input_type.defeature(min_num_voxels);
                if !quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
            }
            if !quiet {
                time = Instant::now();
            }
            let mut output_type: TetrahedralFiniteElements = if adapt {
                Err("Adaptive tetrahedra not yet implemented".to_string())?
            } else {
                if !quiet {
                    print!("     \x1b[1;96mMeshing\x1b[0m voxels into tetrahedra");
                    mesh_print_info(MeshBasis::Voxels, &scale_temporary, &translate_temporary)
                }
                input_type.into()
            };
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    output_type.get_nodal_coordinates().len()
                );
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
                        remeshing: _,
                        iterations,
                        method,
                        hierarchical,
                        pass_band,
                        scale,
                    } => {
                        apply_smoothing_method(
                            &mut output_type,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&output_type, file, quiet)?
            }
            write_finite_elements(output, output_type, quiet)?;
        }
        TRI => {
            if !quiet {
                time = Instant::now();
                if let Some(min_num_voxels) = defeature {
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less",
                    );
                } else {
                    mesh_print_info(MeshBasis::Surfaces, &scale_temporary, &translate_temporary)
                }
            }
            let mut tree = Octree::from(input_type);
            tree.balance(true);
            if let Some(min_num_voxels) = defeature {
                tree.defeature(min_num_voxels);
                println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                time = Instant::now();
                mesh_print_info(MeshBasis::Surfaces, &scale_temporary, &translate_temporary)
            }
            let mut output_type = TriangularFiniteElements::from(tree);
            if !quiet {
                let mut blocks = output_type.get_element_blocks().clone();
                let elements = blocks.len();
                blocks.sort();
                blocks.dedup();
                println!(
                    "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
                    time.elapsed(),
                    blocks.len(),
                    elements,
                    output_type.get_nodal_coordinates().len()
                );
            }
            if let Some(options) = smoothing {
                match options {
                    MeshSmoothCommands::Smooth {
                        remeshing,
                        iterations,
                        method,
                        hierarchical,
                        pass_band,
                        scale,
                    } => {
                        apply_smoothing_method(
                            &mut output_type,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                        if let Some(MeshRemeshCommands::Remesh { iterations, quiet }) = remeshing {
                            let time = Instant::now();
                            if !quiet {
                                println!(
                                    "   \x1b[1;96mRemeshing\x1b[0m isotropically with {iterations} iterations"
                                )
                            }
                            output_type.remesh(
                                iterations,
                                &Smoothing::Taubin(
                                    TAUBIN_DEFAULT_ITERS,
                                    TAUBIN_DEFAULT_BAND,
                                    TAUBIN_DEFAULT_SCALE,
                                ),
                            );
                            if !quiet {
                                println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed())
                            }
                        }
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&output_type, file, quiet)?
            }
            write_finite_elements(output, output_type, quiet)?;
        }
        _ => panic!(),
    }
    Ok(())
}

pub fn mesh_print_info(basis: MeshBasis, scale: &Scale, translate: &Translate) {
    match basis {
        MeshBasis::Leaves => {
            print!("     \x1b[1;96mMeshing\x1b[0m leaves into hexahedra")
        }
        MeshBasis::Surfaces => {
            print!("     \x1b[1;96mMeshing\x1b[0m internal surfaces")
        }
        MeshBasis::Voxels => {}
    }
    if scale != &Default::default() || translate != &Default::default() {
        print!(" \x1b[2m[");
        if scale.x() != &1.0 {
            print!("xscale: {}, ", scale.x())
        }
        if scale.y() != &1.0 {
            print!("yscale: {}, ", scale.y())
        }
        if scale.z() != &1.0 {
            print!("zscale: {}, ", scale.z())
        }
        if translate.x() != &0.0 {
            print!("xtranslate: {}, ", translate.x())
        }
        if translate.y() != &0.0 {
            print!("ytranslate: {}, ", translate.y())
        }
        if translate.z() != &0.0 {
            print!("ztranslate: {}, ", translate.z())
        }
        println!("\x1b[2D]\x1b[0m")
    } else {
        println!()
    }
}
