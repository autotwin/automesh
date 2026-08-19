use super::{
    ErrorWrapper,
    io::{extension, invalid_input, read_segmentation, write_mesh},
    metrics::write_metrics,
    remesh::apply_remesh_subcommand,
    smooth::{MeshSmoothCommands, apply_smoothing_method},
};
use clap::Subcommand;
use conspire::{
    geometry::{
        Coordinate, Coordinates,
        grid::Voxels,
        mesh::{Class, Fitting, Mesh, Tessellation},
        ntree::{Balance, Balancing, CurvatureSizing, Dualization, Octree, Pairing},
        segmentation::Segmentation,
    },
    math::Tensor,
    units::Length,
};
use std::{collections::HashSet, path::Path, time::Instant};

#[derive(Subcommand)]
pub enum MeshSubcommand {
    /// Creates an all-hexahedral mesh from a segmentation or tessellation
    Hex(MeshArgs),
    /// Creates a hex-dominant mesh from a tessellation, polyhedral at the boundary
    Hexdom(MeshArgs),
    /// Creates a polyhedral mesh from a tessellation
    Poly(MeshArgs),
    /// Creates all-triangular isosurface(s) from a segmentation
    Tri(MeshArgs),
}

#[derive(clap::Args)]
pub struct MeshArgs {
    #[command(subcommand)]
    pub smoothing: Option<MeshSmoothCommands>,

    /// Segmentation (npy | spn) or tessellation (stl) input file
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

    /// Voxel IDs to remove from the mesh (npy | spn)
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

    /// Octree refinement scale for dualizing a tessellation (stl) input
    #[arg(long, default_value_t = 5.0, short = 's', value_name = "SCALE")]
    pub scale: f64,

    /// Uniform lattice of the given cell size instead of an octree (stl)
    #[arg(long, short = 'u', value_name = "SPACING")]
    pub uniform: Option<f64>,

    /// Chord-error tolerance for curvature-driven refinement [default: disabled]
    #[arg(long, short = 't', value_name = "TOL")]
    pub tolerance: Option<f64>,

    /// Uses strong balancing instead of the default weak balancing
    #[arg(action, long)]
    pub strong: bool,

    /// Snaps the buffer layer onto the surface instead of a soft fit
    #[arg(action, long)]
    pub snap: bool,

    /// Level difference allowed between neighboring octree cells (poly)
    #[arg(long, default_value_t = 1, short = 'l', value_name = "NUM")]
    pub levels: usize,

    /// Quality metrics output file (csv | npy)
    #[arg(long, value_name = "FILE")]
    pub metrics: Option<String>,
}

pub enum Element {
    Hexahedra,
    HexDominant,
    Polyhedra,
    Triangles,
}

fn read_voxels(args: &MeshArgs, quiet: bool) -> Result<Voxels<u8>, ErrorWrapper> {
    match extension(&args.input) {
        Some("npy") | Some("spn") => {
            let mut voxels =
                read_segmentation(&args.input, args.nelx, args.nely, args.nelz, quiet, true)?;
            if let Some(min) = args.defeature {
                let time = Instant::now();
                crate::echo!(
                    quiet,
                    " \x1b[1;96mDefeaturing\x1b[0m clusters of {min} voxels or less"
                );
                voxels = voxels.defeature(min);
                crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
            }
            Ok(voxels)
        }
        extension => Err(invalid_input(&args.input, extension)),
    }
}

fn finish(mut mesh: Mesh<3>, args: MeshArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    if let Some(MeshSmoothCommands::Smooth {
        remeshing,
        iterations,
        method,
        pass_band,
        scale,
        hierarchical,
    }) = args.smoothing
    {
        apply_smoothing_method(
            &mut mesh,
            iterations,
            method,
            pass_band,
            scale,
            hierarchical,
            quiet,
        )?;
        if let Some(subcommand) = remeshing {
            mesh = apply_remesh_subcommand(mesh, subcommand, quiet)?;
        }
    }
    if let Some(file) = &args.metrics {
        write_metrics(&mesh, file, quiet)?;
    }
    write_mesh(&args.output, mesh, quiet)
}

pub fn mesh(element: Element, args: MeshArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    match (&element, extension(&args.input)) {
        (Element::Hexahedra, Some("stl")) => return hexahedralize(args, quiet),
        (element @ (Element::HexDominant | Element::Polyhedra), Some("stl")) => {
            return cut(args, element, quiet);
        }
        (Element::HexDominant | Element::Polyhedra, extension) => {
            return Err(invalid_input(&args.input, extension));
        }
        _ => {}
    }
    if args.uniform.is_some() {
        return Err(ErrorWrapper::from(
            "Uniform lattice meshing applies to tessellation (stl) inputs only",
        ));
    }
    let voxels = read_voxels(&args, quiet)?;
    let time = Instant::now();
    let mesh = match element {
        Element::Hexahedra => {
            crate::echo!(quiet, "     \x1b[1;96mMeshing\x1b[0m voxels into hexahedra");
            let remove: Option<Vec<u8>> = args
                .remove
                .as_ref()
                .map(|ids| ids.iter().map(|&id| id as u8).collect());
            let scale = Coordinate::from([args.xscale, args.yscale, args.zscale]);
            let translate = Coordinate::from([args.xtranslate, args.ytranslate, args.ztranslate]);
            let segmentation = Segmentation::new(voxels, scale, translate);
            Mesh::from_segmentation(segmentation, remove.as_deref())
        }
        Element::HexDominant | Element::Polyhedra => {
            unreachable!("cutting requires a tessellation input")
        }
        Element::Triangles => {
            crate::echo!(quiet, "     \x1b[1;96mMeshing\x1b[0m voxels into triangles");
            let voxels = remove_materials(voxels, args.remove.as_deref());
            let mesh = Mesh::from(Tessellation::from(voxels));
            scaled(
                mesh,
                [args.xscale, args.yscale, args.zscale],
                [args.xtranslate, args.ytranslate, args.ztranslate],
            )
        }
    };
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} elements, {} nodes]\x1b[0m",
        time.elapsed(),
        mesh.number_of_elements(),
        mesh.number_of_nodes()
    );
    finish(mesh, args, quiet)
}

/// Meshes a tessellation (stl) input into an all-hexahedral mesh.
///
/// A background mesh of the enclosed volume is built first — the dual of an
/// octree fitted to the surface, or a uniform lattice under `--uniform` — and
/// trimmed to the surface. A buffer layer is then fitted onto the surface.
/// Buffering is timed on its own because it dominates the total by far, while
/// the steps building the background are lumped together as one.
fn hexahedralize(args: MeshArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    crate::echo!(quiet, "     \x1b[1;96mReading\x1b[0m {}", args.input);
    let mut time = Instant::now();
    let tessellation = Tessellation::try_from(Path::new(&args.input))?;
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    let fitting = if args.snap {
        Fitting::Snap
    } else {
        Fitting::Soft
    };

    crate::echo!(
        quiet,
        "     \x1b[1;96mMeshing\x1b[0m hexahedra {}",
        if args.uniform.is_some() {
            "uniformly"
        } else {
            "adaptively"
        }
    );
    time = Instant::now();
    let mut mesh = if let Some(spacing) = args.uniform {
        tessellation.lattice_background(Length::meters(spacing))?.0
    } else {
        let balancing = if args.strong {
            Balancing::Strong(1)
        } else {
            Balancing::Weak(1)
        };
        let mut octree = Octree::<u16, usize>::from_features(
            &tessellation,
            args.scale,
            CurvatureSizing {
                tolerance: args.tolerance.map(Length::meters),
                ..Default::default()
            },
            0,
        );
        octree.equilibrate(balancing, Pairing::Regular)?;
        octree.dualize()
    };
    tessellation.trim(&mut mesh)?;
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} elements, {} nodes]\x1b[0m",
        time.elapsed(),
        mesh.number_of_elements(),
        mesh.number_of_nodes()
    );

    crate::echo!(
        quiet,
        "   \x1b[1;96mBuffering\x1b[0m hexahedra onto geometry"
    );
    time = Instant::now();
    let mesh = mesh.buffer(&tessellation, fitting)?;
    let mesh = scaled(
        mesh,
        [args.xscale, args.yscale, args.zscale],
        [args.xtranslate, args.ytranslate, args.ztranslate],
    );
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} elements, {} nodes]\x1b[0m",
        time.elapsed(),
        mesh.number_of_elements(),
        mesh.number_of_nodes()
    );
    finish(mesh, args, quiet)
}

/// Cuts an octree fitted to a tessellation (stl) input to the surface.
///
/// [`Element::Polyhedra`] cuts the octree itself, while [`Element::HexDominant`]
/// cuts its dual, leaving hexahedra everywhere but at the boundary.
fn cut(args: MeshArgs, element: &Element, quiet: bool) -> Result<(), ErrorWrapper> {
    crate::echo!(quiet, "     \x1b[1;96mReading\x1b[0m {}", args.input);
    let mut time = Instant::now();
    let tessellation = Tessellation::try_from(Path::new(&args.input))?;
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    let polyhedral = matches!(element, Element::Polyhedra);
    if polyhedral && args.uniform.is_some() {
        return Err(ErrorWrapper::from(
            "Uniform lattice meshing applies to mesh hex and mesh hexdom only",
        ));
    }
    if !polyhedral && args.levels != 1 {
        return Err(ErrorWrapper::from(
            "Dualization requires 2:1 balancing, so levels applies to mesh poly only",
        ));
    }

    crate::echo!(
        quiet,
        "     \x1b[1;96mMeshing\x1b[0m {} {}",
        if polyhedral { "polyhedra" } else { "hexahedra" },
        if args.uniform.is_some() {
            "uniformly"
        } else {
            "adaptively"
        }
    );
    time = Instant::now();
    let (background, classes) = if let Some(spacing) = args.uniform {
        tessellation.lattice_background(Length::meters(spacing))
    } else {
        let balancing = if args.strong {
            Balancing::Strong(args.levels)
        } else {
            Balancing::Weak(args.levels)
        };
        if polyhedral {
            tessellation.octree_background(balancing, args.scale)
        } else {
            tessellation.dual_background(balancing, args.scale)
        }
    }?;
    let (elements, nodes) = retained(&background, &classes);
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{elements} elements, {nodes} nodes]\x1b[0m",
        time.elapsed()
    );

    crate::echo!(
        quiet,
        "   \x1b[1;96mBuffering\x1b[0m polyhedra onto geometry"
    );
    time = Instant::now();
    let mesh = if polyhedral {
        tessellation.cut_polyhedral(background, &classes)
    } else {
        tessellation.cut(background, &classes)
    }?;
    let mesh = scaled(
        mesh,
        [args.xscale, args.yscale, args.zscale],
        [args.xtranslate, args.ytranslate, args.ztranslate],
    );
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{} elements, {} nodes]\x1b[0m",
        time.elapsed(),
        mesh.number_of_elements(),
        mesh.number_of_nodes()
    );
    finish(mesh, args, quiet)
}

/// Counts the background cells the cut will keep, and the nodes they use.
///
/// A background mesh spans the whole octree or lattice, so most of its cells
/// can lie outside the tessellation and be discarded by the cut. Counting only
/// what survives keeps this step comparable to the one that follows it, and to
/// the hexahedral path, where trimming drops those cells before they are
/// counted at all.
fn retained(background: &Mesh<3>, classes: &[Class]) -> (usize, usize) {
    let mut nodes = HashSet::new();
    let elements = background
        .connectivities()
        .iter()
        .flatten()
        .zip(classes)
        .filter(|(_, class)| !matches!(class, Class::Outside))
        .inspect(|(element, _)| nodes.extend(element.iter().copied()))
        .count();
    (elements, nodes.len())
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
                coordinate[0] * scale[0] + Length::meters(translate[0]),
                coordinate[1] * scale[1] + Length::meters(translate[1]),
                coordinate[2] * scale[2] + Length::meters(translate[2]),
            ])
        })
        .collect();
    Mesh::from((connectivities.into_members(), coordinates))
}
