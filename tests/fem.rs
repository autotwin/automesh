use automesh::{FiniteElementMethods, Scale, Translate, Voxels, NSD};
use conspire::math::Tensor;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    str,
};

fn read_both_lines(
    gold: &mut String,
    gold_reader: &mut BufReader<File>,
    line: &mut String,
    line_reader: &mut BufReader<File>,
) {
    gold.clear();
    line.clear();
    gold_reader.read_line(gold).unwrap();
    line_reader.read_line(line).unwrap();
}

fn read_both_files(
    gold: &mut String,
    gold_reader: &mut BufReader<File>,
    line: &mut String,
    line_reader: &mut BufReader<File>,
) {
    gold.clear();
    line.clear();
    gold_reader.read_to_string(gold).unwrap();
    line_reader.read_to_string(line).unwrap();
}

fn compare_files(
    file_path: &str,
    gold_path: &str,
    spn_path: &str,
    nel: [usize; NSD],
    scale: Scale,
    translate: Translate,
) {
    let voxels = Voxels::from_spn(spn_path, nel.into()).unwrap();
    let fem = voxels
        .into_finite_elements(Some(vec![0]), scale, translate)
        .unwrap();
    fem.write_inp(file_path).unwrap();
    let mut gold = String::new();
    let mut line = String::new();
    let mut gold_reader = BufReader::new(File::open(gold_path).unwrap());
    let mut line_reader = BufReader::new(File::open(file_path).unwrap());
    for _ in 0..2 {
        read_both_lines(&mut gold, &mut gold_reader, &mut line, &mut line_reader);
        assert_eq!(gold, line);
    }
    read_both_lines(&mut gold, &mut gold_reader, &mut line, &mut line_reader);
    let version_prefix_gold = &gold.as_bytes()[0..8];
    let version_prefix_line = &line.as_bytes()[0..8];
    assert_eq!(version_prefix_gold, version_prefix_line);
    let version_gold = format!("{}\n", env!("CARGO_PKG_VERSION"));
    let version_line = str::from_utf8(&line.as_bytes()[8..]).unwrap();
    assert_eq!(version_gold, version_line);
    read_both_lines(&mut gold, &mut gold_reader, &mut line, &mut line_reader);
    let time_prefix_gold = &gold[0..17];
    let time_prefix_line = &line[0..17];
    assert_eq!(time_prefix_gold, time_prefix_line);
    read_both_files(&mut gold, &mut gold_reader, &mut line, &mut line_reader);
    assert_eq!(gold, line);
}

#[cfg(not(target_os = "windows"))]
mod read_inp {
    use super::*;
    use automesh::HexahedralFiniteElements;
    #[test]
    fn letter_f_3d() {
        let voxels = Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        let fem = voxels
            .into_finite_elements(
                Some(vec![0]),
                [1.0, 1.0, 1.0].into(),
                [0.0, 0.0, 0.0].into(),
            )
            .unwrap();
        fem.write_inp("target/letter_f_3d.inp").unwrap();
        let read = HexahedralFiniteElements::from_inp("target/letter_f_3d.inp").unwrap();
        assert_eq!(fem.get_element_blocks(), read.get_element_blocks());
        assert_eq!(
            fem.get_element_node_connectivity(),
            read.get_element_node_connectivity()
        );
        fem.get_nodal_coordinates()
            .iter()
            .zip(read.get_nodal_coordinates().iter())
            .for_each(|(a, b)| a.iter().zip(b.iter()).for_each(|(c, d)| assert_eq!(c, d)));
    }
}

#[cfg(not(target_os = "windows"))]
mod write_inp {
    use super::*;
    #[test]
    fn letter_f_3d() {
        compare_files(
            "target/letter_f_3d.inp",
            "tests/input/letter_f_3d.inp",
            "tests/input/letter_f_3d.spn",
            [4, 5, 3],
            [1.0, 1.0, 1.0].into(),
            [0.0, 0.0, 0.0].into(),
        );
    }
    #[test]
    fn sparse() {
        compare_files(
            "target/sparse.inp",
            "tests/input/sparse.inp",
            "tests/input/sparse.spn",
            [5, 5, 5],
            [1.0, 1.0, 1.0].into(),
            [0.0, 0.0, 0.0].into(),
        );
    }
}

mod write_mesh {
    use super::*;
    #[test]
    fn letter_f_3d() {
        let voxels = Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        let fem = voxels
            .into_finite_elements(
                Some(vec![0]),
                [1.0, 1.0, 1.0].into(),
                [0.0, 0.0, 0.0].into(),
            )
            .unwrap();
        fem.write_mesh("target/letter_f_3d.mesh").unwrap();
    }
}

mod write_vtk {
    use super::*;
    #[test]
    fn letter_f_3d() {
        let voxels = Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        let fem = voxels
            .into_finite_elements(
                Some(vec![0]),
                [1.0, 1.0, 1.0].into(),
                [0.0, 0.0, 0.0].into(),
            )
            .unwrap();
        fem.write_vtk("target/letter_f_3d.vtk").unwrap();
    }
}
