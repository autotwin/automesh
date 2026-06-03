use automesh::Tessellation;
use conspire::{
    geometry::mesh::{Connectivity, Mesh},
    io::Write,
    math::Tensor,
};
use std::path::Path;

fn vertices_and_faces(tessellation: Tessellation) -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
    let mesh = Mesh::from(tessellation);
    let vertices = mesh
        .coordinates()
        .iter()
        .map(|coordinate| [coordinate[0], coordinate[1], coordinate[2]])
        .collect();
    let faces = mesh
        .connectivities()
        .iter()
        .flat_map(|connectivity| match connectivity {
            Connectivity::Triangular(triangles) => triangles.iter().copied().collect::<Vec<_>>(),
            _ => panic!("expected triangular connectivity"),
        })
        .collect();
    (vertices, faces)
}

fn one_facet() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
    (
        vec![[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        vec![[0, 1, 2]],
    )
}

fn two_facet() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
    (
        vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
        vec![[0, 1, 2], [2, 3, 0]],
    )
}

mod try_from {
    use super::*;
    #[test]
    #[cfg(not(target_os = "windows"))]
    #[should_panic(expected = "No such file or directory")]
    fn file_nonexistent() {
        Tessellation::try_from(Path::new("tests/input/f_file_nonexistent.stl")).unwrap();
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_one_facet() {
        let tess = Tessellation::try_from(Path::new("tests/input/one_facet.stl")).unwrap();
        assert_eq!(vertices_and_faces(tess), one_facet());
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_two_facet() {
        let tess = Tessellation::try_from(Path::new("tests/input/two_facet.stl")).unwrap();
        assert_eq!(vertices_and_faces(tess), two_facet());
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_single() {
        let _tess = Tessellation::try_from(Path::new("tests/input/single.stl"));
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_double() {
        let _tess = Tessellation::try_from(Path::new("tests/input/double.stl"));
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_single_valence_04_noise2() {
        let _tess = Tessellation::try_from(Path::new("tests/input/single_valence_04_noise2.stl"));
    }
}

mod write {
    use super::*;
    use std::fs::remove_file;
    #[test]
    fn one_facet_write_read() {
        let file_gold = Path::new("tests/input/one_facet.stl");
        let tess_gold = Tessellation::try_from(file_gold).unwrap();
        let file_test = "tests/input/one_facet_test.stl";
        tess_gold.write(file_test).unwrap();
        let tess_test = Tessellation::try_from(Path::new(file_test)).unwrap();
        assert_eq!(vertices_and_faces(tess_test), one_facet());
        remove_file(file_test).unwrap();
    }
    #[test]
    fn two_facet_write_read() {
        let file_gold = Path::new("tests/input/two_facet.stl");
        let tess_gold = Tessellation::try_from(file_gold).unwrap();
        let file_test = "tests/input/two_facet_test.stl";
        tess_gold.write(file_test).unwrap();
        let tess_test = Tessellation::try_from(Path::new(file_test)).unwrap();
        assert_eq!(vertices_and_faces(tess_test), two_facet());
        remove_file(file_test).unwrap();
    }
}
