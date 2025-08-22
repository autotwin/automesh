// use automesh::{Tessellation, NSD};
use automesh::Tessellation;
use std::fs::OpenOptions;
use stl_io::{IndexedMesh, IndexedTriangle, Normal, Triangle, Vertex};

fn tessellation_one_facet() -> Tessellation {
    let vertices = vec![
        Vertex::new([0.0, 0.0, 1.0]),
        Vertex::new([0.0, 0.0, 0.0]),
        Vertex::new([1.0, 0.0, 0.0]),
    ];
    let normal = Normal::new([0.0, -1.0, 0.0]);
    let indexed_triangle = IndexedTriangle {
        normal,
        vertices: [0, 1, 2],
    };

    let indexed_mesh = IndexedMesh {
        vertices,
        faces: vec![indexed_triangle],
    };
    Tessellation::new(indexed_mesh)
}

fn tessellation_two_facet() -> Tessellation {
    let vertices = vec![
        Vertex::new([0.0, 0.0, 1.0]),
        Vertex::new([0.0, 0.0, 0.0]),
        Vertex::new([1.0, 0.0, 0.0]),
        Vertex::new([1.0, 0.0, 1.0]),
    ];
    let faces = vec![
        IndexedTriangle {
            normal: Normal::new([0.0, -1.0, 0.0]),
            vertices: [0, 1, 2],
        },
        IndexedTriangle {
            normal: Normal::new([0.0, -1.0, 0.0]),
            vertices: [2, 3, 0],
        },
    ];
    let indexed_mesh = IndexedMesh { vertices, faces };
    Tessellation::new(indexed_mesh)
}

mod try_from {
    use super::*;
    #[test]
    #[cfg(not(target_os = "windows"))]
    #[should_panic(expected = "No such file or directory")]
    fn file_nonexistent() {
        Tessellation::try_from("tests/input/f_file_nonexistent.stl").unwrap();
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_one_facet() {
        let tess = Tessellation::try_from("tests/input/one_facet.stl").unwrap();
        println!("{tess:?}");
        assert_eq!(tess, tessellation_one_facet());
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_two_facet() {
        let tess = Tessellation::try_from("tests/input/two_facet.stl").unwrap();
        // println!("{:?}", tess);
        assert_eq!(tess, tessellation_two_facet());
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_single() {
        let _tess = Tessellation::try_from("tests/input/single.stl");
        // println!("{:?}", _tess);
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_double() {
        let _tess = Tessellation::try_from("tests/input/double.stl");
        // println!("{:?}", _tess);
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn file_single_valence_04_noise2() {
        let _tess = Tessellation::try_from("tests/input/single_valence_04_noise2.stl");
        // println!("{:?}", _tess);
        println!("{_tess:#?}"); // pretty-print
    }
}

mod write_stl {
    use super::*;
    use std::fs::remove_file;
    #[test]
    fn one_facet_write_read() {
        // Write a binary stl from a gold standard.
        let file_gold = "tests/input/one_facet.stl";
        let tess_gold = Tessellation::try_from(file_gold).unwrap();
        println!("gold: {tess_gold:?}");
        let file_test = "tests/input/one_facet_test.stl";
        let mesh_iter = tess_gold.get_data().faces.iter().map(|face| Triangle {
            normal: face.normal,
            vertices: face
                .vertices
                .iter()
                .map(|&vertex| tess_gold.get_data().vertices[vertex])
                .collect::<Vec<Vertex>>()
                .try_into()
                .unwrap(),
        });
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_test)
            .unwrap();
        stl_io::write_stl(&mut file, mesh_iter).unwrap();
        println!("tess -> stl(binary), wrote temporary test file: {file_test}",);
        // Read the binary data back in and assure it equals the gold standard.
        let tess_test = Tessellation::try_from(file_test).unwrap();
        assert_eq!(tess_test, tessellation_one_facet());
        // Delete the temporary test stl.
        match remove_file(file_test) {
            Ok(_) => println!("Successfully deleted temporary test file: {file_test}"),
            Err(e) => eprintln!("Error deleting temporary test file: {file_test} {e}"),
        }
    }
    #[test]
    fn two_facet_write_read() {
        // Write a binary stl from a gold standard.
        let file_gold = "tests/input/two_facet.stl";
        let tess_gold = Tessellation::try_from(file_gold).unwrap();
        println!("{tess_gold:?}");
        let file_test = "tests/input/two_facet_test.stl";
        let mesh_iter = tess_gold.get_data().faces.iter().map(|face| Triangle {
            normal: face.normal,
            vertices: face
                .vertices
                .iter()
                .map(|&vertex| tess_gold.get_data().vertices[vertex])
                .collect::<Vec<Vertex>>()
                .try_into()
                .unwrap(),
        });
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file_test)
            .unwrap();
        stl_io::write_stl(&mut file, mesh_iter).unwrap();
        println!("tess -> stl(binary), wrote test file: {file_test}");
        // Read the binary data back in and assure it equals the gold standard.
        let tess_test = Tessellation::try_from(file_test).unwrap();
        assert_eq!(tess_test, tessellation_two_facet());
        // Delete the temporary test stl.
        match remove_file(file_test) {
            Ok(_) => println!("Successfully deleted temporary test file: {file_test}"),
            Err(e) => eprintln!("Error deleting temporary test file: {file_test} {e}"),
        }
    }
}
