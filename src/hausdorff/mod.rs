mod surface;

#[cfg(test)]
mod test;

use super::{
    ErrorWrapper,
    io::{extension, invalid_input, read_mesh},
};
use conspire::{
    geometry::mesh::{Connectivity, Mesh, Tessellation, Verdict},
    math::FxHashMap,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use surface::Surface;

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

pub fn hausdorff(args: HausdorffArgs, quiet: bool) -> Result<(), ErrorWrapper> {
    let mesh = read_volume(&args.mesh, quiet)?;
    let tessellation = read_tessellation(&args.tessellation, quiet)?;
    let boundary = Boundary::from(&mesh);
    boundary.surface.check("mesh boundary")?;
    tessellation.check("tessellation")?;
    if boundary.tangled > 0 {
        crate::echo!(
            quiet,
            "     \x1b[1;93mWarning\x1b[0m {} boundary elements have a negative minimum scaled Jacobian; signs near them are suspect",
            boundary.tangled
        );
    }
    let prefix = Path::new(&args.output).with_extension("");
    let a_samples = boundary
        .nodes
        .iter()
        .map(|&node| boundary.surface.points()[node])
        .collect::<Vec<_>>();
    let a_distances = tessellation.signed_distances(&a_samples);
    let b_samples = tessellation.points();
    let b_distances = boundary.surface.signed_distances(b_samples);
    let a_to_b = Summary::from(a_distances.as_slice());
    let b_to_a = Summary::from(b_distances.as_slice());
    let hausdorff = a_to_b.maximum.max(b_to_a.maximum);
    a_to_b.echo("A to B", quiet);
    b_to_a.echo("B to A", quiet);
    crate::echo!(quiet, "   \x1b[1;96mHausdorff\x1b[0m H_h = {hausdorff:e}");
    let labels = mesh.node_labels();
    write_rows(
        &suffixed(&prefix, "_A_to_B.csv"),
        &a_to_b.comments("A to B", hausdorff),
        "node,reference node,x,y,z,signed distance",
        boundary
            .nodes
            .iter()
            .zip(a_samples.iter().zip(a_distances.iter()))
            .enumerate()
            .map(|(row, (&node, ([x, y, z], s)))| {
                let reference = labels.map_or(node + 1, |labels| labels[node]);
                format!("{},{reference},{x},{y},{z},{s}", row + 1)
            }),
        quiet,
    )?;
    write_rows(
        &suffixed(&prefix, "_B_to_A.csv"),
        &b_to_a.comments("B to A", hausdorff),
        "vertex,x,y,z,signed distance",
        b_samples
            .iter()
            .zip(b_distances.iter())
            .enumerate()
            .map(|(row, ([x, y, z], s))| format!("{},{x},{y},{z},{s}", row + 1)),
        quiet,
    )
}

fn read_volume(file: &str, quiet: bool) -> Result<Mesh<3>, ErrorWrapper> {
    let extension = extension(file);
    if !matches!(extension, Some("exo" | "inp" | "mesh" | "vtu")) {
        return Err(invalid_input(file, extension));
    }
    let mesh = read_mesh(file, quiet)?;
    let hexahedral = mesh
        .iter()
        .all(|block| matches!(block, Connectivity::Hexahedral(_)));
    let tetrahedral = mesh
        .iter()
        .all(|block| matches!(block, Connectivity::Tetrahedral(_)));
    if hexahedral || tetrahedral {
        Ok(mesh)
    } else {
        Err(format!("Mesh {file} must hold only hexahedra or only tetrahedra").into())
    }
}

fn read_tessellation(file: &str, quiet: bool) -> Result<Surface, ErrorWrapper> {
    let extension = extension(file);
    if extension != Some("stl") {
        return Err(invalid_input(file, extension));
    }
    crate::echo!(quiet, "     \x1b[1;96mReading\x1b[0m {file}");
    let time = Instant::now();
    let tessellation = Tessellation::try_from(Path::new(file))?;
    let triangles = tessellation
        .mesh()
        .connectivities()
        .iter()
        .flatten()
        .map(|triangle| [triangle[0], triangle[1], triangle[2]])
        .collect();
    let surface = Surface::new(triangles, tessellation.mesh().coordinates().clone());
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    Ok(surface)
}

struct Boundary {
    surface: Surface,
    nodes: Vec<usize>,
    tangled: usize,
}

impl From<&Mesh<3>> for Boundary {
    fn from(mesh: &Mesh<3>) -> Self {
        let mut faces = FxHashMap::<Vec<usize>, (Vec<usize>, usize, usize)>::default();
        let mut element = 0;
        mesh.iter().for_each(|block| {
            block.iter().for_each(|nodes| {
                block.element_faces(nodes).into_iter().for_each(|oriented| {
                    let mut key = oriented.clone();
                    key.sort_unstable();
                    faces
                        .entry(key)
                        .and_modify(|(_, _, count)| *count += 1)
                        .or_insert((oriented, element, 1));
                });
                element += 1;
            })
        });
        let mut exterior = faces
            .into_values()
            .filter_map(|(face, element, count)| (count == 1).then_some((face, element)))
            .collect::<Vec<_>>();
        exterior.sort_unstable();
        let coordinates = mesh.coordinates().clone();
        let point = |node: usize| [0, 1, 2].map(|i| coordinates[node][i].value());
        let triangles = exterior
            .iter()
            .flat_map(|(face, _)| match *face.as_slice() {
                [n0, n1, n2] => vec![[n0, n1, n2]],
                [n0, n1, n2, n3] => quadrilateral_split([n0, n1, n2, n3], point).to_vec(),
                _ => panic!("exterior faces are triangles or quadrilaterals"),
            })
            .collect();
        let mut nodes = exterior
            .iter()
            .flat_map(|(face, _)| face.iter().copied())
            .collect::<Vec<_>>();
        nodes.sort_unstable();
        nodes.dedup();
        let mut elements = exterior
            .iter()
            .map(|&(_, element)| element)
            .collect::<Vec<_>>();
        elements.sort_unstable();
        elements.dedup();
        let jacobians = mesh
            .minimum_scaled_jacobians()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let tangled = elements
            .iter()
            .filter(|&&element| jacobians[element] < 0.0)
            .count();
        Self {
            surface: Surface::new(triangles, coordinates),
            nodes,
            tangled,
        }
    }
}

fn quadrilateral_split(
    [n0, n1, n2, n3]: [usize; 4],
    point: impl Fn(usize) -> [f64; 3],
) -> [[usize; 3]; 2] {
    let length = |a: usize, b: usize| {
        let (a, b) = (point(a), point(b));
        (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>()
    };
    if length(n1, n3) < length(n0, n2) {
        [[n0, n1, n3], [n1, n2, n3]]
    } else {
        [[n0, n1, n2], [n0, n2, n3]]
    }
}

fn suffixed(prefix: &Path, suffix: &str) -> PathBuf {
    let mut file = prefix.as_os_str().to_owned();
    file.push(suffix);
    file.into()
}

fn write_rows(
    file: &Path,
    comments: &[String],
    header: &str,
    rows: impl Iterator<Item = String>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    crate::echo!(quiet, "     \x1b[1;96mWriting\x1b[0m {}", file.display());
    let time = Instant::now();
    let mut writer = BufWriter::new(File::create(file)?);
    for comment in comments {
        writeln!(writer, "# {comment}")?;
    }
    writeln!(writer, "{header}")?;
    for row in rows {
        writeln!(writer, "{row}")?;
    }
    writer.flush()?;
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    Ok(())
}

struct Summary {
    samples: usize,
    maximum: f64,
    mean: f64,
    rms: f64,
}

impl From<&[f64]> for Summary {
    fn from(distances: &[f64]) -> Self {
        let count = distances.len() as f64;
        Self {
            samples: distances.len(),
            maximum: distances
                .iter()
                .fold(0.0_f64, |maximum, s| maximum.max(s.abs())),
            mean: distances.iter().map(|s| s.abs()).sum::<f64>() / count,
            rms: (distances.iter().map(|s| s * s).sum::<f64>() / count).sqrt(),
        }
    }
}

impl Summary {
    fn echo(&self, direction: &str, quiet: bool) {
        crate::echo!(
            quiet,
            "   \x1b[1;96mHausdorff\x1b[0m {direction}: max |s| = {:e}, mean |s| = {:e}, rms |s| = {:e}",
            self.maximum,
            self.mean,
            self.rms
        );
    }
    fn comments(&self, direction: &str, hausdorff: f64) -> Vec<String> {
        vec![
            format!("direction = {direction}"),
            format!("samples = {}", self.samples),
            format!("max |s| = {}", self.maximum),
            format!("mean |s| = {}", self.mean),
            format!("rms |s| = {}", self.rms),
            format!("H_h = {hausdorff}"),
        ]
    }
}
