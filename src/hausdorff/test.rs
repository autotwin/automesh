use super::{Boundary, Surface, quadrilateral_split, read_tessellation, read_volume};
use conspire::{
    geometry::{
        Coordinate, Coordinates,
        mesh::{Connectivity, Mesh},
    },
    units::Length,
};

const TOLERANCE: f64 = 1e-12;

fn coordinates(points: &[[f64; 3]]) -> Coordinates<3> {
    points
        .iter()
        .map(|point| Coordinate::from(point.map(Length::meters)))
        .collect()
}

const CUBE_POINTS: [[f64; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];

const CUBE_HEXAHEDRON: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

fn cube_triangles() -> Vec<[usize; 3]> {
    [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ]
    .into_iter()
    .flat_map(|[a, b, c, d]| [[a, b, c], [a, c, d]])
    .collect()
}

fn cube() -> Surface {
    Surface::new(cube_triangles(), coordinates(&CUBE_POINTS))
}

fn signed_distance(surface: &Surface, sample: [f64; 3]) -> f64 {
    surface.signed_distances(&[sample])[0]
}

fn assert_close(found: f64, expected: f64) {
    assert!(
        (found - expected).abs() < TOLERANCE,
        "{found} != {expected}"
    );
}

#[test]
fn facet_offset_outward_is_positive() {
    assert_close(signed_distance(&cube(), [0.5, 0.5, 1.1]), 0.1);
}

#[test]
fn facet_offset_inward_is_negative() {
    assert_close(signed_distance(&cube(), [0.5, 0.5, 0.9]), -0.1);
}

#[test]
fn closest_point_on_an_edge() {
    assert_close(
        signed_distance(&cube(), [0.5, 1.1, 1.1]),
        0.1 * 2.0_f64.sqrt(),
    );
}

#[test]
fn closest_point_on_a_vertex() {
    assert_close(
        signed_distance(&cube(), [1.1, 1.1, 1.1]),
        0.1 * 3.0_f64.sqrt(),
    );
    assert_close(
        signed_distance(&cube(), [-0.1, -0.1, -0.1]),
        0.1 * 3.0_f64.sqrt(),
    );
}

#[test]
fn sample_on_the_surface_is_zero() {
    let surface = cube();
    [[1.0, 1.0, 1.0], [0.5, 0.5, 1.0], [0.5, 0.0, 0.0]]
        .into_iter()
        .for_each(|sample| assert_eq!(signed_distance(&surface, sample), 0.0));
}

fn tip() -> (Surface, [f64; 3], [[f64; 3]; 2]) {
    let half_width = 20.0_f64.to_radians().tan();
    let profile = [[-half_width, 0.0], [half_width, 0.0], [0.0, 1.0]];
    let points = [0.0, 1.0]
        .into_iter()
        .flat_map(|z| profile.map(|[x, y]| [x, y, z]))
        .collect::<Vec<_>>();
    let mut triangles = vec![[0, 2, 1], [3, 4, 5]];
    (0..3).for_each(|i| {
        let j = (i + 1) % 3;
        triangles.push([i, j, j + 3]);
        triangles.push([i, j + 3, i + 3]);
    });
    let normal = |[x0, y0]: [f64; 2], [x1, y1]: [f64; 2]| {
        let (dx, dy) = (x1 - x0, y1 - y0);
        let length = (dx * dx + dy * dy).sqrt();
        [dy / length, -dx / length, 0.0]
    };
    let faces = [
        normal(profile[1], profile[2]),
        normal(profile[2], profile[0]),
    ];
    (
        Surface::new(triangles, coordinates(&points)),
        [0.0, 1.0, 0.5],
        faces,
    )
}

#[test]
fn sharp_tip_surface_passes_the_checks() {
    tip().0.check("tip").unwrap();
}

#[test]
fn sharp_edge_takes_its_sign_from_the_pseudonormal() {
    let (surface, tip, faces) = tip();
    let radius = 0.1;
    let mut wrong = 0;
    (25..=155).step_by(10).for_each(|degrees| {
        let angle = f64::from(degrees).to_radians();
        let direction = [angle.cos(), angle.sin(), 0.0];
        wrong += faces
            .iter()
            .filter(|face| (0..3).map(|i| face[i] * direction[i]).sum::<f64>() < 0.0)
            .count();
        let sample = [0, 1, 2].map(|i| tip[i] + radius * direction[i]);
        assert_close(signed_distance(&surface, sample), radius);
    });
    assert!(wrong > 0, "no sample would get a wrong sign from one face");
}

#[test]
fn sharp_vertex_takes_its_sign_from_the_pseudonormal() {
    let (surface, _, faces) = tip();
    let radius = 0.1;
    let mut wrong = 0;
    [0.05_f64, 0.2, 1.0].into_iter().for_each(|rise| {
        (25..=155).step_by(10).for_each(|degrees| {
            let angle = f64::from(degrees).to_radians();
            let length = (1.0 + rise * rise).sqrt();
            let direction = [angle.cos(), angle.sin(), rise].map(|entry| entry / length);
            wrong += faces
                .iter()
                .filter(|face| (0..3).map(|i| face[i] * direction[i]).sum::<f64>() < 0.0)
                .count();
            let sample = [
                radius * direction[0],
                1.0 + radius * direction[1],
                1.0 + radius * direction[2],
            ];
            assert_close(signed_distance(&surface, sample), radius);
        })
    });
    assert!(wrong > 0, "no sample would get a wrong sign from one face");
}

#[test]
fn cube_surface_passes_the_checks() {
    let surface = cube();
    surface.check("cube").unwrap();
    assert_close(surface.volume(), 1.0);
}

#[test]
fn open_surface_is_rejected() {
    let mut triangles = cube_triangles();
    triangles.pop();
    let error = Surface::new(triangles, coordinates(&CUBE_POINTS))
        .check("cube")
        .unwrap_err();
    assert!(format!("{error:?}").contains("not closed"), "{error:?}");
}

#[test]
fn inconsistently_wound_surface_is_rejected() {
    let mut triangles = cube_triangles();
    triangles[0].swap(1, 2);
    let error = Surface::new(triangles, coordinates(&CUBE_POINTS))
        .check("cube")
        .unwrap_err();
    assert!(
        format!("{error:?}").contains("not consistently wound"),
        "{error:?}"
    );
}

#[test]
fn inside_out_surface_is_rejected() {
    let triangles = cube_triangles()
        .into_iter()
        .map(|[a, b, c]| [a, c, b])
        .collect();
    let error = Surface::new(triangles, coordinates(&CUBE_POINTS))
        .check("cube")
        .unwrap_err();
    assert!(format!("{error:?}").contains("not outward"), "{error:?}");
}

fn hexahedron(points: &[[f64; 3]]) -> Mesh<3> {
    Mesh::from((
        vec![Connectivity::Hexahedral(vec![CUBE_HEXAHEDRON].into())],
        coordinates(points),
    ))
}

#[test]
fn hexahedron_boundary_is_outward() {
    let boundary = Boundary::from(&hexahedron(&CUBE_POINTS));
    boundary.surface.check("hexahedron").unwrap();
    assert_close(boundary.surface.volume(), 1.0);
    assert_eq!(boundary.nodes, (0..8).collect::<Vec<_>>());
    assert_eq!(boundary.tangled, 0);
}

#[test]
fn mirrored_hexahedron_is_rejected() {
    let points = CUBE_POINTS.map(|[x, y, z]| [-x, y, z]);
    let error = Boundary::from(&hexahedron(&points))
        .surface
        .check("mesh boundary")
        .unwrap_err();
    assert!(format!("{error:?}").contains("not outward"), "{error:?}");
}

#[test]
fn quadrilateral_splits_along_the_shorter_diagonal() {
    let points = [
        [0.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let point = |node: usize| points[node];
    assert_eq!(
        quadrilateral_split([0, 1, 2, 3], point),
        [[0, 1, 3], [1, 2, 3]]
    );
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let point = |node: usize| points[node];
    assert_eq!(
        quadrilateral_split([0, 1, 2, 3], point),
        [[0, 1, 2], [0, 2, 3]]
    );
}

fn scratch(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("automesh_hausdorff_{}_{name}", std::process::id()))
        .to_str()
        .unwrap()
        .to_string()
}

#[test]
fn zeroed_facet_normals_do_not_change_the_sign() {
    let file = scratch("zeroed.stl");
    let mut stl = String::from("solid zeroed\n");
    cube_triangles().iter().for_each(|triangle| {
        stl.push_str("  facet normal 0 0 0\n    outer loop\n");
        triangle.iter().for_each(|&node| {
            let [x, y, z] = CUBE_POINTS[node];
            stl.push_str(&format!("      vertex {x} {y} {z}\n"));
        });
        stl.push_str("    endloop\n  endfacet\n");
    });
    stl.push_str("endsolid zeroed\n");
    std::fs::write(&file, stl).unwrap();
    let surface = read_tessellation(&file, true).unwrap();
    std::fs::remove_file(&file).unwrap();
    surface.check("tessellation").unwrap();
    assert_close(signed_distance(&surface, [0.5, 0.5, 1.1]), 0.1);
    assert_close(signed_distance(&surface, [0.5, 0.5, 0.9]), -0.1);
}

#[test]
fn tessellation_must_be_stl() {
    assert!(read_tessellation("surface.off", true).is_err());
    assert!(read_tessellation("surface.inp", true).is_err());
}

#[test]
fn mesh_must_be_volumetric_by_extension() {
    let error = read_volume("surface.stl", true).err().unwrap();
    assert!(format!("{error:?}").contains(".stl"), "{error:?}");
    let error = read_volume("surface.off", true).err().unwrap();
    assert!(format!("{error:?}").contains(".off"), "{error:?}");
}

#[test]
fn mesh_must_hold_hexahedra_or_tetrahedra() {
    let file = scratch("triangles.inp");
    std::fs::write(
        &file,
        "*Node\n1, 0, 0, 0\n2, 1, 0, 0\n3, 0, 1, 0\n*Element, type=S3, elset=EB1\n1, 1, 2, 3\n",
    )
    .unwrap();
    let error = read_volume(&file, true).err().unwrap();
    std::fs::remove_file(&file).unwrap();
    assert!(
        format!("{error:?}").contains("only hexahedra or only tetrahedra"),
        "{error:?}"
    );
}
