use super::{
    ErrorWrapper,
    io::{extension, invalid_input, read_segmentation, write_mesh},
    metrics::write_metrics,
    remesh::{MeshRemeshCommands, apply_remeshing},
    smooth::{MeshSmoothCommands, apply_smoothing_method},
};
use clap::Subcommand;
use conspire::{
    geometry::{
        Coordinate, Coordinates,
        grid::Voxels,
        mesh::{Mesh, Tessellation},
        segmentation::Segmentation,
    },
    math::Tensor,
};
use std::time::Instant;

#[derive(Subcommand)]
pub enum MeshSubcommand {
    /// Creates an all-hexahedral mesh from a segmentation
    Hex(MeshArgs),
    /// Creates all-triangular isosurface(s) from a segmentation
    Tri(MeshArgs),
}

#[derive(clap::Args)]
pub struct MeshArgs {
    #[command(subcommand)]
    pub smoothing: Option<MeshSmoothCommands>,

    /// Segmentation (npy | spn) input file
    #[arg(long, short, value_name = "FILE")]
    pub input: String,

    /// Mesh output file (exo | inp | mesh | stl | vtu)
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
}

pub enum Element {
    Hexahedra,
    Triangles,
}

fn read_voxels(args: &MeshArgs) -> Result<Voxels<u8>, ErrorWrapper> {
    match extension(&args.input) {
        Some("npy") | Some("spn") => {
            let mut voxels = read_segmentation(
                &args.input,
                args.nelx,
                args.nely,
                args.nelz,
                args.quiet,
                true,
            )?;
            if let Some(min) = args.defeature {
                let time = Instant::now();
                if !args.quiet {
                    println!(" \x1b[1;96mDefeaturing\x1b[0m clusters of {min} voxels or less");
                }
                voxels = voxels.defeature(min);
                if !args.quiet {
                    println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
                }
            }
            Ok(voxels)
        }
        extension => Err(invalid_input(&args.input, extension)),
    }
}

fn finish(mut mesh: Mesh<3>, args: MeshArgs) -> Result<(), ErrorWrapper> {
    if let Some(MeshSmoothCommands::Smooth {
        remeshing,
        iterations,
        method,
        pass_band,
        scale,
    }) = args.smoothing
    {
        apply_smoothing_method(&mut mesh, iterations, method, pass_band, scale, args.quiet)?;
        if let Some(MeshRemeshCommands::Remesh { iterations, .. }) = remeshing {
            mesh = apply_remeshing(mesh, iterations, args.quiet)?;
        }
    }
    if let Some(file) = &args.metrics {
        write_metrics(&mesh, file, args.quiet)?;
    }
    write_mesh(&args.output, mesh, args.quiet)
}

pub fn mesh(element: Element, args: MeshArgs) -> Result<(), ErrorWrapper> {
    let voxels = read_voxels(&args)?;
    let time = Instant::now();
    let mesh = match element {
        Element::Hexahedra => {
            if !args.quiet {
                println!("     \x1b[1;96mMeshing\x1b[0m voxels into hexahedra");
            }
            let remove: Option<Vec<u8>> = args
                .remove
                .as_ref()
                .map(|ids| ids.iter().map(|&id| id as u8).collect());
            let scale = Coordinate::from([args.xscale, args.yscale, args.zscale]);
            let translate = Coordinate::from([args.xtranslate, args.ytranslate, args.ztranslate]);
            let segmentation = Segmentation::new(voxels, scale, translate);
            Mesh::from_segmentation(segmentation, remove.as_deref())
        }
        Element::Triangles => {
            if !args.quiet {
                println!("     \x1b[1;96mMeshing\x1b[0m voxels into triangles");
            }
            let voxels = remove_materials(voxels, args.remove.as_deref());
            let mesh = Mesh::from(Tessellation::from(voxels));
            scaled(
                mesh,
                [args.xscale, args.yscale, args.zscale],
                [args.xtranslate, args.ytranslate, args.ztranslate],
            )
        }
    };
    if !args.quiet {
        println!(
            "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} elements, {} nodes]\x1b[0m",
            time.elapsed(),
            mesh.number_of_elements(),
            mesh.number_of_nodes()
        );
    }
    finish(mesh, args)
}

/// Zeroes out (treats as void) any voxels whose material is in `remove`.
fn remove_materials(voxels: Voxels<u8>, remove: Option<&[usize]>) -> Voxels<u8> {
    match remove {
        Some(remove) if !remove.is_empty() => {
            let nel = *voxels.nel();
            let data = voxels
                .data()
                .iter()
                .map(|&block| {
                    if remove.contains(&(block as usize)) {
                        0
                    } else {
                        block
                    }
                })
                .collect();
            Voxels::new(data, nel)
        }
        _ => voxels,
    }
}

/// Applies per-axis scaling (before translation) to the mesh coordinates.
fn scaled(mesh: Mesh<3>, scale: [f64; 3], translate: [f64; 3]) -> Mesh<3> {
    if scale == [1.0, 1.0, 1.0] && translate == [0.0, 0.0, 0.0] {
        return mesh;
    }
    let (connectivities, coordinates) = mesh.into();
    let coordinates: Coordinates<3> = coordinates
        .iter()
        .map(|coordinate| {
            Coordinate::from([
                coordinate[0] * scale[0] + translate[0],
                coordinate[1] * scale[1] + translate[1],
                coordinate[2] * scale[2] + translate[2],
            ])
        })
        .collect();
    Mesh::from((connectivities.into_members(), coordinates))
}
