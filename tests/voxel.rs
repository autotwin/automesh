use automesh::Voxels;

const NELX: usize = 4;
const NELY: usize = 5;
const NELZ: usize = 3;
const NEL: [usize; 3] = [NELX, NELY, NELZ];
const NSD: usize = 3; // number of space dimensions

const GOLD_DATA: [[[u8; NELZ]; NELY]; NELX] = [
    [[1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 1, 1]],
    [[1, 0, 0], [1, 0, 0], [1, 1, 0], [1, 0, 0], [1, 1, 1]],
    [[1, 0, 0], [1, 0, 0], [1, 1, 0], [1, 0, 0], [1, 1, 1]],
    [[1, 0, 0], [1, 0, 0], [1, 1, 0], [1, 0, 0], [1, 1, 1]],
];

fn assert_data_eq(voxels_from_npy: Voxels, voxels_from_spn: Voxels) {
    let voxels_from_npy_data = voxels_from_npy.get_data();
    let voxels_from_spn_data = voxels_from_spn.get_data();
    voxels_from_npy_data
        .shape()
        .iter()
        .zip(voxels_from_spn_data.shape().iter())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
    voxels_from_npy_data
        .iter()
        .zip(voxels_from_spn_data.iter())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
}

fn assert_data_eq_gold(spn: Voxels) {
    let data = spn.get_data();
    data.shape()
        .iter()
        .zip(NEL.iter())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
    data.iter()
        .zip(GOLD_DATA.iter().flatten().flatten())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
}

fn assert_data_eq_gold_1d<T>(data: &Vec<T>, gold: &[T])
where
    T: std::fmt::Debug + std::cmp::PartialEq,
{
    assert_eq!(data.len(), gold.len());
    data.iter()
        .zip(gold.iter())
        .for_each(|(data_entry, gold_entry)| assert_eq!(data_entry, gold_entry));
}

fn assert_data_eq_gold_2d<const N: usize, T>(data: &Vec<Vec<T>>, gold: &[[T; N]])
where
    T: std::fmt::Debug + std::cmp::PartialEq,
{
    assert_eq!(data.len(), gold.len());
    assert_eq!(data[0].len(), gold[0].len());
    data.iter()
        .flatten()
        .zip(gold.iter().flatten())
        .for_each(|(data_entry, gold_entry)| assert_eq!(data_entry, gold_entry));
}

fn assert_fem_data_from_spn_eq_gold<const D: usize, const E: usize, const N: usize>(
    gold: Gold<D, E, N>,
) {
    let voxels = Voxels::from_spn(&gold.file_path, gold.nel);
    let fem = voxels.into_finite_elements(&gold.scale, &gold.translate);
    assert_data_eq_gold_1d(fem.get_element_blocks(), &gold.element_blocks);
    assert_data_eq_gold_2d(fem.get_element_connectivity(), &gold.element_connectivity);
    assert_data_eq_gold_2d(fem.get_nodal_coordinates(), &gold.element_coordinates);
}

/// A Gold struct is a so-called gold standard, taken as a trusted result,
/// used for testing purposes.
struct Gold<const D: usize, const E: usize, const N: usize> {
    /// The block id for each element.
    element_blocks: [usize; E],

    /// The connectivity matrix of a finite element mesh, with E rows of
    /// elements, and with each element composed of N local element node numbers
    /// in columns.
    element_connectivity: [[usize; N]; E],

    /// The matrix of nodal points, with D rows of nodal points, and with each
    /// nodal point composed of (x, y, z) floats in columns.
    element_coordinates: [[f64; NSD]; D],

    /// The fully pathed file input string.
    file_path: String,

    /// The number of voxels that compose the segmentation lattice domain in
    /// the [x, y, z] directions.
    nel: [usize; NSD],

    /// The scaling in the [x, y, z] directions to be applied to the domain.
    scale: [f64; NSD],

    /// The translation in the [x, y, z] directions to be applied to the domain.
    translate: [f64; NSD],
}

/// The default implementation of the `Gold` struct, which is abstract since
/// the fields need to be overwritten with concrete data at time of instantiation.
impl<const D: usize, const E: usize, const N: usize> Default for Gold<D, E, N> {
    fn default() -> Self {
        Self {
            element_blocks: [0; E],
            element_connectivity: [[0; N]; E],
            element_coordinates: [[0.0; NSD]; D],
            file_path: "".to_string(),
            nel: [0; NSD],
            scale: [1.0; NSD],
            translate: [0.0; NSD],
        }
    }
}

#[test]
fn from_spn() {
    let voxels = Voxels::from_spn("tests/input/letter_f_3d.spn", NEL);
    assert_data_eq_gold(voxels);
}

mod into_finite_elements {
    use super::*;
    /// A single voxel lattice.
    #[test]
    fn single() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            file_path: "tests/input/single.spn".to_string(),
            nel: [1; NSD],
            ..Default::default()
        });
    }
    /// A double voxel lattice, coursed along the x-axis.
    #[test]
    fn double_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_connectivity: [[1, 2, 5, 4, 7, 8, 11, 10], [2, 3, 6, 5, 8, 9, 12, 11]],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
            ],
            file_path: "tests/input/double.spn".to_string(),
            nel: [2, 1, 1],
            ..Default::default()
        });
    }
    /// A double voxel lattice, coursed along the y-axis.
    #[test]
    fn double_y() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_connectivity: [[1, 2, 4, 3, 7, 8, 10, 9], [3, 4, 6, 5, 9, 10, 12, 11]],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
            ],
            file_path: "tests/input/double.spn".to_string(),
            nel: [1, 2, 1],
            ..Default::default()
        });
    }
    #[test]
    /// A triple voxel lattice, coursed along the x-axis.
    fn triple_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11, 11],
            element_connectivity: [
                [1, 2, 6, 5, 9, 10, 14, 13],
                [2, 3, 7, 6, 10, 11, 15, 14],
                [3, 4, 8, 7, 11, 12, 16, 15],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
            ],
            file_path: "tests/input/triple.spn".to_string(),
            nel: [3, 1, 1],
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, coursed along the x-axis.
    #[test]
    fn quadruple_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11, 11, 11],
            element_connectivity: [
                [1, 2, 7, 6, 11, 12, 17, 16],
                [2, 3, 8, 7, 12, 13, 18, 17],
                [3, 4, 9, 8, 13, 14, 19, 18],
                [4, 5, 10, 9, 14, 15, 20, 19],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [4.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple.spn".to_string(),
            nel: [4, 1, 1],
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, coursed along the x-axis, with two
    /// intermediate voxels in the segmentation being void.
    #[test]
    fn quadruple_2_voids_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_connectivity: [[1, 2, 6, 5, 9, 10, 14, 13], [3, 4, 8, 7, 11, 12, 16, 15]],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [4.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple_2_voids.spn".to_string(),
            nel: [4, 1, 1],
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the two intermediate voxels in the
    /// segmentation being a second block.
    #[test]
    fn quadruple_2_blocks() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 21, 21, 11],
            element_connectivity: [
                [1, 2, 7, 6, 11, 12, 17, 16],
                [2, 3, 8, 7, 12, 13, 18, 17],
                [3, 4, 9, 8, 13, 14, 19, 18],
                [4, 5, 10, 9, 14, 15, 20, 19],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [4.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple_2_blocks.spn".to_string(),
            nel: [4, 1, 1],
            ..Default::default()
        });
    }
    #[test]
    /// A quadruple voxel lattice, with the first intermediate voxel being
    /// the second block and the second intermediate voxel being void.
    fn quadruple_2_blocks_void() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 21, 11],
            element_connectivity: [
                [1, 2, 7, 6, 11, 12, 17, 16],
                [2, 3, 8, 7, 12, 13, 18, 17],
                [4, 5, 10, 9, 14, 15, 20, 19],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [4.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple_2_blocks_void.spn".to_string(),
            nel: [4, 1, 1],
            ..Default::default()
        });
    }
    /// A (2 x 2 x 2) voxel cube.
    #[test]
    fn cube() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11, 11, 11, 11, 11, 11, 11],
            element_connectivity: [
                [1, 2, 5, 4, 10, 11, 14, 13],
                [2, 3, 6, 5, 11, 12, 15, 14],
                [4, 5, 8, 7, 13, 14, 17, 16],
                [5, 6, 9, 8, 14, 15, 18, 17],
                [10, 11, 14, 13, 19, 20, 23, 22],
                [11, 12, 15, 14, 20, 21, 24, 23],
                [13, 14, 17, 16, 22, 23, 26, 25],
                [14, 15, 18, 17, 23, 24, 27, 26],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
                [2.0, 2.0, 1.0],
                [0.0, 0.0, 2.0],
                [1.0, 0.0, 2.0],
                [2.0, 0.0, 2.0],
                [0.0, 1.0, 2.0],
                [1.0, 1.0, 2.0],
                [2.0, 1.0, 2.0],
                [0.0, 2.0, 2.0],
                [1.0, 2.0, 2.0],
                [2.0, 2.0, 2.0],
            ],
            file_path: "tests/input/cube.spn".to_string(),
            nel: [2, 2, 2],
            ..Default::default()
        });
    }
    /// A (2 x 2 x 2) voxel cube with two voids and six elements.
    #[test]
    fn cube_multi() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [82, 2, 2, 2, 31, 44],
            element_connectivity: [
                [1, 2, 5, 4, 10, 11, 14, 13],
                [2, 3, 6, 5, 11, 12, 15, 14],
                [4, 5, 8, 7, 13, 14, 17, 16],
                [5, 6, 9, 8, 14, 15, 18, 17],
                [11, 12, 15, 14, 19, 20, 22, 21],
                [14, 15, 18, 17, 21, 22, 24, 23],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
                [2.0, 2.0, 1.0],
                [1.0, 0.0, 2.0],
                [2.0, 0.0, 2.0],
                [1.0, 1.0, 2.0],
                [2.0, 1.0, 2.0],
                [1.0, 2.0, 2.0],
                [2.0, 2.0, 2.0],
            ],
            file_path: "tests/input/cube_multi.spn".to_string(),
            nel: [2, 2, 2],
            ..Default::default()
        });
    }
    /// A minimal letter F example.
    #[test]
    fn letter_f() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11; 8],
            element_connectivity: [
                [1, 2, 4, 3, 19, 20, 22, 21],
                [3, 4, 6, 5, 21, 22, 24, 23],
                [5, 6, 9, 8, 23, 24, 27, 26],
                [6, 7, 10, 9, 24, 25, 28, 27],
                [8, 9, 12, 11, 26, 27, 30, 29],
                [11, 12, 16, 15, 29, 30, 34, 33],
                [12, 13, 17, 16, 30, 31, 35, 34],
                [13, 14, 18, 17, 31, 32, 36, 35],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 3.0, 0.0],
                [1.0, 3.0, 0.0],
                [2.0, 3.0, 0.0],
                [0.0, 4.0, 0.0],
                [1.0, 4.0, 0.0],
                [2.0, 4.0, 0.0],
                [3.0, 4.0, 0.0],
                [0.0, 5.0, 0.0],
                [1.0, 5.0, 0.0],
                [2.0, 5.0, 0.0],
                [3.0, 5.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
                [2.0, 2.0, 1.0],
                [0.0, 3.0, 1.0],
                [1.0, 3.0, 1.0],
                [2.0, 3.0, 1.0],
                [0.0, 4.0, 1.0],
                [1.0, 4.0, 1.0],
                [2.0, 4.0, 1.0],
                [3.0, 4.0, 1.0],
                [0.0, 5.0, 1.0],
                [1.0, 5.0, 1.0],
                [2.0, 5.0, 1.0],
                [3.0, 5.0, 1.0],
            ],
            file_path: "tests/input/letter_f.spn".to_string(),
            nel: [3, 5, 1],
            ..Default::default()
        });
    }
    /// A three dimensional variation of the letter F, in a non-standard
    /// orientation.
    #[test]
    fn letter_f_3d() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [1; 39],
            element_connectivity: [
                [1, 2, 7, 6, 31, 32, 37, 36],
                [2, 3, 8, 7, 32, 33, 38, 37],
                [3, 4, 9, 8, 33, 34, 39, 38],
                [4, 5, 10, 9, 34, 35, 40, 39],
                [6, 7, 12, 11, 36, 37, 42, 41],
                [7, 8, 13, 12, 37, 38, 43, 42],
                [8, 9, 14, 13, 38, 39, 44, 43],
                [9, 10, 15, 14, 39, 40, 45, 44],
                [11, 12, 17, 16, 41, 42, 47, 46],
                [12, 13, 18, 17, 42, 43, 48, 47],
                [13, 14, 19, 18, 43, 44, 49, 48],
                [14, 15, 20, 19, 44, 45, 50, 49],
                [16, 17, 22, 21, 46, 47, 52, 51],
                [17, 18, 23, 22, 47, 48, 53, 52],
                [18, 19, 24, 23, 48, 49, 54, 53],
                [19, 20, 25, 24, 49, 50, 55, 54],
                [21, 22, 27, 26, 51, 52, 57, 56],
                [22, 23, 28, 27, 52, 53, 58, 57],
                [23, 24, 29, 28, 53, 54, 59, 58],
                [24, 25, 30, 29, 54, 55, 60, 59],
                [31, 32, 37, 36, 61, 62, 64, 63],
                [36, 37, 42, 41, 63, 64, 66, 65],
                [41, 42, 47, 46, 65, 66, 71, 70],
                [42, 43, 48, 47, 66, 67, 72, 71],
                [43, 44, 49, 48, 67, 68, 73, 72],
                [44, 45, 50, 49, 68, 69, 74, 73],
                [46, 47, 52, 51, 70, 71, 76, 75],
                [51, 52, 57, 56, 75, 76, 81, 80],
                [52, 53, 58, 57, 76, 77, 82, 81],
                [53, 54, 59, 58, 77, 78, 83, 82],
                [54, 55, 60, 59, 78, 79, 84, 83],
                [61, 62, 64, 63, 85, 86, 88, 87],
                [63, 64, 66, 65, 87, 88, 90, 89],
                [65, 66, 71, 70, 89, 90, 92, 91],
                [70, 71, 76, 75, 91, 92, 94, 93],
                [75, 76, 81, 80, 93, 94, 99, 98],
                [76, 77, 82, 81, 94, 95, 100, 99],
                [77, 78, 83, 82, 95, 96, 101, 100],
                [78, 79, 84, 83, 96, 97, 102, 101],
            ],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [3.0, 2.0, 0.0],
                [4.0, 2.0, 0.0],
                [0.0, 3.0, 0.0],
                [1.0, 3.0, 0.0],
                [2.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
                [4.0, 3.0, 0.0],
                [0.0, 4.0, 0.0],
                [1.0, 4.0, 0.0],
                [2.0, 4.0, 0.0],
                [3.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
                [0.0, 5.0, 0.0],
                [1.0, 5.0, 0.0],
                [2.0, 5.0, 0.0],
                [3.0, 5.0, 0.0],
                [4.0, 5.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [4.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
                [2.0, 2.0, 1.0],
                [3.0, 2.0, 1.0],
                [4.0, 2.0, 1.0],
                [0.0, 3.0, 1.0],
                [1.0, 3.0, 1.0],
                [2.0, 3.0, 1.0],
                [3.0, 3.0, 1.0],
                [4.0, 3.0, 1.0],
                [0.0, 4.0, 1.0],
                [1.0, 4.0, 1.0],
                [2.0, 4.0, 1.0],
                [3.0, 4.0, 1.0],
                [4.0, 4.0, 1.0],
                [0.0, 5.0, 1.0],
                [1.0, 5.0, 1.0],
                [2.0, 5.0, 1.0],
                [3.0, 5.0, 1.0],
                [4.0, 5.0, 1.0],
                [0.0, 0.0, 2.0],
                [1.0, 0.0, 2.0],
                [0.0, 1.0, 2.0],
                [1.0, 1.0, 2.0],
                [0.0, 2.0, 2.0],
                [1.0, 2.0, 2.0],
                [2.0, 2.0, 2.0],
                [3.0, 2.0, 2.0],
                [4.0, 2.0, 2.0],
                [0.0, 3.0, 2.0],
                [1.0, 3.0, 2.0],
                [2.0, 3.0, 2.0],
                [3.0, 3.0, 2.0],
                [4.0, 3.0, 2.0],
                [0.0, 4.0, 2.0],
                [1.0, 4.0, 2.0],
                [2.0, 4.0, 2.0],
                [3.0, 4.0, 2.0],
                [4.0, 4.0, 2.0],
                [0.0, 5.0, 2.0],
                [1.0, 5.0, 2.0],
                [2.0, 5.0, 2.0],
                [3.0, 5.0, 2.0],
                [4.0, 5.0, 2.0],
                [0.0, 0.0, 3.0],
                [1.0, 0.0, 3.0],
                [0.0, 1.0, 3.0],
                [1.0, 1.0, 3.0],
                [0.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [0.0, 3.0, 3.0],
                [1.0, 3.0, 3.0],
                [0.0, 4.0, 3.0],
                [1.0, 4.0, 3.0],
                [2.0, 4.0, 3.0],
                [3.0, 4.0, 3.0],
                [4.0, 4.0, 3.0],
                [0.0, 5.0, 3.0],
                [1.0, 5.0, 3.0],
                [2.0, 5.0, 3.0],
                [3.0, 5.0, 3.0],
                [4.0, 5.0, 3.0],
            ],
            file_path: "tests/input/letter_f_3d.spn".to_string(),
            nel: [4, 5, 3],
            ..Default::default()
        });
    }
    #[test]
    fn sparse() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [
                2, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2,
                1, 1, 2, 1, 1, 1, 2, 2, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 1, 1, 1,
                2, 1,
            ],
            element_connectivity: [
                [1, 2, 4, 3, 29, 30, 36, 35],
                [3, 4, 10, 9, 35, 36, 42, 41],
                [5, 6, 12, 11, 37, 38, 44, 43],
                [6, 7, 13, 12, 38, 39, 45, 44],
                [8, 9, 15, 14, 40, 41, 47, 46],
                [9, 10, 16, 15, 41, 42, 48, 47],
                [11, 12, 18, 17, 43, 44, 50, 49],
                [15, 16, 22, 21, 47, 48, 54, 53],
                [17, 18, 24, 23, 49, 50, 56, 55],
                [18, 19, 25, 24, 50, 51, 57, 56],
                [20, 21, 27, 26, 52, 53, 59, 58],
                [21, 22, 28, 27, 53, 54, 60, 59],
                [31, 32, 38, 37, 64, 65, 71, 70],
                [32, 33, 39, 38, 65, 66, 72, 71],
                [34, 35, 41, 40, 67, 68, 74, 73],
                [35, 36, 42, 41, 68, 69, 75, 74],
                [40, 41, 47, 46, 73, 74, 80, 79],
                [43, 44, 50, 49, 76, 77, 83, 82],
                [44, 45, 51, 50, 77, 78, 84, 83],
                [46, 47, 53, 52, 79, 80, 86, 85],
                [49, 50, 56, 55, 82, 83, 89, 88],
                [54, 55, 61, 60, 87, 88, 93, 92],
                [62, 63, 69, 68, 96, 97, 102, 101],
                [63, 64, 70, 69, 97, 98, 103, 102],
                [64, 65, 71, 70, 98, 99, 104, 103],
                [70, 71, 77, 76, 103, 104, 110, 109],
                [75, 76, 82, 81, 108, 109, 114, 113],
                [76, 77, 83, 82, 109, 110, 115, 114],
                [81, 82, 88, 87, 113, 114, 119, 118],
                [82, 83, 89, 88, 114, 115, 120, 119],
                [86, 87, 92, 91, 117, 118, 123, 122],
                [88, 89, 94, 93, 119, 120, 125, 124],
                [89, 90, 95, 94, 120, 121, 126, 125],
                [98, 99, 104, 103, 130, 131, 137, 136],
                [99, 100, 105, 104, 131, 132, 138, 137],
                [101, 102, 108, 107, 134, 135, 141, 140],
                [102, 103, 109, 108, 135, 136, 142, 141],
                [106, 107, 112, 111, 139, 140, 146, 145],
                [108, 109, 114, 113, 141, 142, 148, 147],
                [111, 112, 117, 116, 145, 146, 151, 150],
                [112, 113, 118, 117, 146, 147, 152, 151],
                [114, 115, 120, 119, 148, 149, 154, 153],
                [118, 119, 124, 123, 152, 153, 159, 158],
                [120, 121, 126, 125, 154, 155, 161, 160],
                [127, 128, 134, 133, 162, 163, 168, 167],
                [129, 130, 136, 135, 164, 165, 170, 169],
                [130, 131, 137, 136, 165, 166, 171, 170],
                [133, 134, 140, 139, 167, 168, 174, 173],
                [134, 135, 141, 140, 168, 169, 175, 174],
                [135, 136, 142, 141, 169, 170, 176, 175],
                [136, 137, 143, 142, 170, 171, 177, 176],
                [137, 138, 144, 143, 171, 172, 178, 177],
                [141, 142, 148, 147, 175, 176, 180, 179],
                [147, 148, 153, 152, 179, 180, 185, 184],
                [148, 149, 154, 153, 180, 181, 186, 185],
                [150, 151, 157, 156, 182, 183, 189, 188],
                [151, 152, 158, 157, 183, 184, 190, 189],
                [154, 155, 161, 160, 186, 187, 192, 191],
            ],
            element_coordinates: [
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
                [5.0, 1.0, 0.0],
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [3.0, 2.0, 0.0],
                [4.0, 2.0, 0.0],
                [5.0, 2.0, 0.0],
                [0.0, 3.0, 0.0],
                [1.0, 3.0, 0.0],
                [2.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
                [4.0, 3.0, 0.0],
                [5.0, 3.0, 0.0],
                [0.0, 4.0, 0.0],
                [1.0, 4.0, 0.0],
                [2.0, 4.0, 0.0],
                [3.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
                [5.0, 4.0, 0.0],
                [0.0, 5.0, 0.0],
                [1.0, 5.0, 0.0],
                [2.0, 5.0, 0.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
                [5.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [4.0, 1.0, 1.0],
                [5.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
                [2.0, 2.0, 1.0],
                [3.0, 2.0, 1.0],
                [4.0, 2.0, 1.0],
                [5.0, 2.0, 1.0],
                [0.0, 3.0, 1.0],
                [1.0, 3.0, 1.0],
                [2.0, 3.0, 1.0],
                [3.0, 3.0, 1.0],
                [4.0, 3.0, 1.0],
                [5.0, 3.0, 1.0],
                [0.0, 4.0, 1.0],
                [1.0, 4.0, 1.0],
                [2.0, 4.0, 1.0],
                [3.0, 4.0, 1.0],
                [4.0, 4.0, 1.0],
                [5.0, 4.0, 1.0],
                [0.0, 5.0, 1.0],
                [1.0, 5.0, 1.0],
                [2.0, 5.0, 1.0],
                [3.0, 5.0, 1.0],
                [1.0, 0.0, 2.0],
                [2.0, 0.0, 2.0],
                [3.0, 0.0, 2.0],
                [4.0, 0.0, 2.0],
                [5.0, 0.0, 2.0],
                [0.0, 1.0, 2.0],
                [1.0, 1.0, 2.0],
                [2.0, 1.0, 2.0],
                [3.0, 1.0, 2.0],
                [4.0, 1.0, 2.0],
                [5.0, 1.0, 2.0],
                [0.0, 2.0, 2.0],
                [1.0, 2.0, 2.0],
                [2.0, 2.0, 2.0],
                [3.0, 2.0, 2.0],
                [4.0, 2.0, 2.0],
                [5.0, 2.0, 2.0],
                [0.0, 3.0, 2.0],
                [1.0, 3.0, 2.0],
                [2.0, 3.0, 2.0],
                [3.0, 3.0, 2.0],
                [4.0, 3.0, 2.0],
                [5.0, 3.0, 2.0],
                [0.0, 4.0, 2.0],
                [1.0, 4.0, 2.0],
                [2.0, 4.0, 2.0],
                [3.0, 4.0, 2.0],
                [4.0, 4.0, 2.0],
                [5.0, 4.0, 2.0],
                [1.0, 5.0, 2.0],
                [2.0, 5.0, 2.0],
                [3.0, 5.0, 2.0],
                [4.0, 5.0, 2.0],
                [5.0, 5.0, 2.0],
                [1.0, 0.0, 3.0],
                [2.0, 0.0, 3.0],
                [3.0, 0.0, 3.0],
                [4.0, 0.0, 3.0],
                [5.0, 0.0, 3.0],
                [1.0, 1.0, 3.0],
                [2.0, 1.0, 3.0],
                [3.0, 1.0, 3.0],
                [4.0, 1.0, 3.0],
                [5.0, 1.0, 3.0],
                [0.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [2.0, 2.0, 3.0],
                [3.0, 2.0, 3.0],
                [4.0, 2.0, 3.0],
                [0.0, 3.0, 3.0],
                [1.0, 3.0, 3.0],
                [2.0, 3.0, 3.0],
                [3.0, 3.0, 3.0],
                [4.0, 3.0, 3.0],
                [0.0, 4.0, 3.0],
                [1.0, 4.0, 3.0],
                [2.0, 4.0, 3.0],
                [3.0, 4.0, 3.0],
                [4.0, 4.0, 3.0],
                [5.0, 4.0, 3.0],
                [1.0, 5.0, 3.0],
                [2.0, 5.0, 3.0],
                [3.0, 5.0, 3.0],
                [4.0, 5.0, 3.0],
                [5.0, 5.0, 3.0],
                [0.0, 0.0, 4.0],
                [1.0, 0.0, 4.0],
                [2.0, 0.0, 4.0],
                [3.0, 0.0, 4.0],
                [4.0, 0.0, 4.0],
                [5.0, 0.0, 4.0],
                [0.0, 1.0, 4.0],
                [1.0, 1.0, 4.0],
                [2.0, 1.0, 4.0],
                [3.0, 1.0, 4.0],
                [4.0, 1.0, 4.0],
                [5.0, 1.0, 4.0],
                [0.0, 2.0, 4.0],
                [1.0, 2.0, 4.0],
                [2.0, 2.0, 4.0],
                [3.0, 2.0, 4.0],
                [4.0, 2.0, 4.0],
                [5.0, 2.0, 4.0],
                [0.0, 3.0, 4.0],
                [1.0, 3.0, 4.0],
                [2.0, 3.0, 4.0],
                [3.0, 3.0, 4.0],
                [4.0, 3.0, 4.0],
                [0.0, 4.0, 4.0],
                [1.0, 4.0, 4.0],
                [2.0, 4.0, 4.0],
                [3.0, 4.0, 4.0],
                [4.0, 4.0, 4.0],
                [5.0, 4.0, 4.0],
                [0.0, 5.0, 4.0],
                [1.0, 5.0, 4.0],
                [2.0, 5.0, 4.0],
                [3.0, 5.0, 4.0],
                [4.0, 5.0, 4.0],
                [5.0, 5.0, 4.0],
                [0.0, 0.0, 5.0],
                [1.0, 0.0, 5.0],
                [2.0, 0.0, 5.0],
                [3.0, 0.0, 5.0],
                [4.0, 0.0, 5.0],
                [0.0, 1.0, 5.0],
                [1.0, 1.0, 5.0],
                [2.0, 1.0, 5.0],
                [3.0, 1.0, 5.0],
                [4.0, 1.0, 5.0],
                [5.0, 1.0, 5.0],
                [0.0, 2.0, 5.0],
                [1.0, 2.0, 5.0],
                [2.0, 2.0, 5.0],
                [3.0, 2.0, 5.0],
                [4.0, 2.0, 5.0],
                [5.0, 2.0, 5.0],
                [2.0, 3.0, 5.0],
                [3.0, 3.0, 5.0],
                [4.0, 3.0, 5.0],
                [0.0, 4.0, 5.0],
                [1.0, 4.0, 5.0],
                [2.0, 4.0, 5.0],
                [3.0, 4.0, 5.0],
                [4.0, 4.0, 5.0],
                [5.0, 4.0, 5.0],
                [0.0, 5.0, 5.0],
                [1.0, 5.0, 5.0],
                [2.0, 5.0, 5.0],
                [4.0, 5.0, 5.0],
                [5.0, 5.0, 5.0],
            ],
            file_path: "tests/input/sparse.spn".to_string(),
            nel: [5; NSD],
            ..Default::default()
        });
    }
}

mod from_npy {
    use super::*;
    #[test]
    #[should_panic(expected = "File type must be .npy")]
    fn file_unreadable() {
        let _ = Voxels::from_npy("tests/input/letter_f_3d.txt");
    }
    #[test]
    #[should_panic(expected = "Could not find the .npy file")]
    fn file_nonexistent() {
        let _ = Voxels::from_npy("tests/input/f_file_nonexistent.npy");
    }
    #[test]
    #[should_panic(expected = "Could not open the .npy file")]
    fn file_unopenable() {
        let _ = Voxels::from_npy("tests/input/encrypted.npy");
    }
    #[test]
    fn success() {
        let voxels = Voxels::from_npy("tests/input/letter_f_3d.npy");
        assert_data_eq_gold(voxels);
    }
}

mod write_npy {
    use super::*;
    #[test]
    fn letter_f_3d() {
        let voxels_from_spn = Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3]);
        voxels_from_spn.write_npy("target/letter_f_3d.npy");
        let voxels_from_npy = Voxels::from_npy("target/letter_f_3d.npy");
        assert_data_eq(voxels_from_npy, voxels_from_spn);
    }
    #[test]
    fn sparse() {
        let voxels_from_spn = Voxels::from_spn("tests/input/sparse.spn", [5, 5, 5]);
        voxels_from_spn.write_npy("target/sparse.npy");
        let voxels_from_npy = Voxels::from_npy("target/sparse.npy");
        assert_data_eq(voxels_from_npy, voxels_from_spn);
    }
}