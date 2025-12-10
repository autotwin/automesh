use super::{
    ErrorWrapper,
    input::{invalid_input, read_segmentation, read_tessellation},
    output::{validate_output, write_finite_elements, write_metrics},
    remesh::{MeshRemeshCommands, apply_remeshing},
    smooth::{MeshSmoothCommands, apply_smoothing_method},
};
use automesh::{
    FiniteElementMethods, HEX, HexahedralFiniteElements, Octree, Remove, Scale, Size, TET, TRI,
    Tessellation, TetrahedralFiniteElements, Translate, TriangularFiniteElements,
};
use clap::Subcommand;
use conspire::math::Tensor;
use std::{path::Path, time::Instant};

#[derive(Subcommand)]
pub enum MeshSubcommand {
    /// Creates an all-hexahedral mesh from a tessellation or segmentation
    Hex(MeshArgs),
    /// Creates an all-tetrahedral mesh from a tessellation or segmentation
    Tet(MeshArgs),
    /// Creates all-triangular isosurface(s) from a tessellation or segmentation
    Tri(MeshArgs),
}

#[derive(clap::Args)]
pub struct MeshArgs {
    #[command(subcommand)]
    pub smoothing: Option<MeshSmoothCommands>,

    /// Tessellation (stl) or segmentation (npy | spn) input file
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

    /// Desired element size on the surface
    #[arg(long, short = 's', value_name = "VAL")]
    pub size: Option<f64>,

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
}
pub enum MeshBasis {
    Leaves,
    Surfaces,
    Voxels,
}

#[allow(clippy::too_many_arguments)]
pub fn mesh<const M: usize, const N: usize, T>(
    smoothing: Option<MeshSmoothCommands>,
    input: String,
    output: String,
    defeature: Option<usize>,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    remove: Option<Vec<usize>>,
    size: Option<f64>,
    xscale: f64,
    yscale: f64,
    zscale: f64,
    xtranslate: f64,
    ytranslate: f64,
    ztranslate: f64,
    metrics: Option<String>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N>
        + From<Tessellation>
        + TryFrom<(Tessellation, Size), Error = String>,
    Tessellation: From<T>,
{
    let scale = Scale::from([xscale, yscale, zscale]);
    let translate = Translate::from([xtranslate, ytranslate, ztranslate]);
    let input_extension = Path::new(&input).extension().and_then(|ext| ext.to_str());
    match input_extension {
        Some("npy") | Some("spn") => mesh_segmentation::<N>(
            smoothing, input, output, defeature, nelx, nely, nelz, remove, scale, translate,
            metrics, quiet,
        ),
        Some("stl") => mesh_tessellation::<M, N, T>(
            smoothing, input, output, size, scale, translate, metrics, quiet,
        ),
        _ => Err(invalid_input(&input, input_extension)),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mesh_segmentation<const N: usize>(
    smoothing: Option<MeshSmoothCommands>,
    input: String,
    output: String,
    defeature: Option<usize>,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    remove: Option<Vec<usize>>,
    scale: Scale,
    translate: Translate,
    metrics: Option<String>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    let mut time = Instant::now();
    let remove: Remove = Remove::from(remove);
    let mut input_type = read_segmentation(
        input,
        nelx,
        nely,
        nelz,
        remove,
        scale.clone(),
        translate.clone(),
        quiet,
        true,
    )?;
    validate_output("mesh", &output)?;
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
                print!("     \x1b[1;96mMeshing\x1b[0m voxels into hexahedra");
                mesh_print_info(MeshBasis::Voxels, &scale, &translate)
            }
            let mut finite_elements: HexahedralFiniteElements = input_type.into();
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
                            &mut finite_elements,
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
                write_metrics(&finite_elements, file, quiet)?
            }
            write_finite_elements(output, finite_elements, quiet)?;
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
                print!("     \x1b[1;96mMeshing\x1b[0m voxels into tetrahedra");
                mesh_print_info(MeshBasis::Voxels, &scale, &translate)
            }
            let mut finite_elements: TetrahedralFiniteElements = input_type.into();
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
                            &mut finite_elements,
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
                write_metrics(&finite_elements, file, quiet)?
            }
            write_finite_elements(output, finite_elements, quiet)?;
        }
        TRI => {
            if !quiet {
                time = Instant::now();
                if let Some(min_num_voxels) = defeature {
                    println!(
                        " \x1b[1;96mDefeaturing\x1b[0m clusters of {min_num_voxels} voxels or less",
                    );
                } else {
                    mesh_print_info(MeshBasis::Surfaces, &scale, &translate)
                }
            }
            let mut tree = Octree::from(input_type);
            tree.balance(true);
            if let Some(min_num_voxels) = defeature {
                tree.defeature(min_num_voxels);
                println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                time = Instant::now();
                mesh_print_info(MeshBasis::Surfaces, &scale, &translate)
            }
            let mut finite_elements = TriangularFiniteElements::from(tree);
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
                            &mut finite_elements,
                            iterations,
                            method,
                            hierarchical,
                            pass_band,
                            scale,
                            quiet,
                        )?;
                        if let Some(MeshRemeshCommands::Remesh {
                            iterations,
                            quiet: _,
                        }) = remeshing
                        {
                            apply_remeshing(&mut finite_elements, iterations, quiet, true)?;
                        }
                    }
                }
            }
            if let Some(file) = metrics {
                write_metrics(&finite_elements, file, quiet)?
            }
            write_finite_elements(output, finite_elements, quiet)?;
        }
        _ => panic!(),
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn mesh_tessellation<const M: usize, const N: usize, T>(
    smoothing: Option<MeshSmoothCommands>,
    input: String,
    output: String,
    size: Option<f64>,
    scale: Scale,
    translate: Translate,
    metrics: Option<String>,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N> + TryFrom<(Tessellation, Size), Error = String>,
    Tessellation: From<T>,
{
    let mut time = Instant::now();
    let tessellation = read_tessellation(&input, quiet, true)?;
    if !quiet {
        time = Instant::now();
        match N {
            HEX => print!("     \x1b[1;96mMeshing\x1b[0m adaptive hexahedra"),
            TET => print!("     \x1b[1;96mMeshing\x1b[0m adaptive tetrahedra"),
            TRI => print!("     \x1b[1;96mMeshing\x1b[0m adaptive triangles"),
            _ => panic!(),
        }
        mesh_print_info(MeshBasis::Voxels, &scale, &translate)
    }
    let mut finite_elements = T::try_from((tessellation, size))?;
    if !quiet {
        #[cfg(feature = "profile")]
        let other_time = Instant::now();
        let mut blocks = finite_elements.get_element_blocks().clone();
        let elements = blocks.len();
        blocks.sort();
        blocks.dedup();
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mShow number of blocks\x1b[0m {:?}",
            other_time.elapsed()
        );
        println!(
            "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} blocks, {} elements, {} nodes]\x1b[0m",
            time.elapsed(),
            blocks.len(),
            elements,
            finite_elements.get_nodal_coordinates().len()
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
                    &mut finite_elements,
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
        write_metrics(&finite_elements, file, quiet)?
    }
    write_finite_elements(output, finite_elements, quiet)
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
