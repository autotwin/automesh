use automesh::{FiniteElementMethods, Nel, Scale, Translate, Voxels, NSD};
use conspire::math::{Tensor, TensorVec};

const GOLD_DATA: [[[u8; 3]; 5]; 4] = [
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
        .zip([4, 5, 3].iter())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
    data.iter()
        .zip(GOLD_DATA.iter().flatten().flatten())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
}

fn assert_data_eq_gold_1d<T>(data: &[T], gold: &[T])
where
    T: std::fmt::Debug + std::cmp::PartialEq,
{
    assert_eq!(data.len(), gold.len());
    data.iter()
        .zip(gold.iter())
        .for_each(|(data_entry, gold_entry)| assert_eq!(data_entry, gold_entry));
}

fn assert_data_eq_gold_2d<const N: usize, T>(data: &[[T; 8]], gold: &[[T; N]])
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
    let voxels = Voxels::from_spn(&gold.file_path, gold.nel).unwrap();
    let fem = voxels
        .into_finite_elements(gold.remove, gold.scale, gold.translate)
        .unwrap();
    assert_data_eq_gold_1d(fem.get_element_blocks(), &gold.element_blocks);
    assert_data_eq_gold_2d(
        fem.get_element_node_connectivity(),
        &gold.element_node_connectivity,
    );
    // assert_data_eq_gold_2d(fem.get_nodal_coordinates(), &gold.element_coordinates);
    assert_eq!(
        fem.get_nodal_coordinates().len(),
        gold.element_coordinates.len()
    );
    fem.get_nodal_coordinates()
        .iter()
        .zip(gold.element_coordinates.iter())
        .for_each(|(data, gold)| {
            data.iter()
                .zip(gold.iter())
                .for_each(|(data_entry, gold_entry)| assert_eq!(data_entry, gold_entry))
        });
}

/// A Gold struct is a so-called gold standard, taken as a trusted result,
/// used for testing purposes.
struct Gold<const D: usize, const E: usize, const N: usize> {
    /// The block id for each element.
    element_blocks: [u8; E],

    /// The connectivity matrix of a finite element mesh, with E rows of
    /// elements, and with each element composed of N local element node numbers
    /// in columns.
    element_node_connectivity: [[usize; N]; E],

    /// The matrix of nodal points, with D rows of nodal points, and with each
    /// nodal point composed of (x, y, z) floats in columns.
    element_coordinates: [[f64; NSD]; D],

    /// The full path file input string.
    file_path: String,

    /// The number of voxels that compose the segmentation lattice domain in
    /// the [x, y, z] directions.
    nel: Nel,

    /// Option to remove blocks.
    remove: Option<Vec<u8>>,

    /// The scaling in the [x, y, z] directions to be applied to the domain.
    scale: Scale,

    /// The translation in the [x, y, z] directions to be applied to the domain.
    translate: Translate,
}

/// The default implementation of the `Gold` struct, which is abstract since
/// the fields need to be overwritten with concrete data at time of instantiation.
impl<const D: usize, const E: usize, const N: usize> Default for Gold<D, E, N> {
    fn default() -> Self {
        Self {
            element_blocks: [0; E],
            element_node_connectivity: [[0; N]; E],
            element_coordinates: [[0.0; NSD]; D],
            file_path: "".to_string(),
            nel: [1; NSD].into(),
            remove: Option::Some(vec![0]),
            scale: Default::default(),
            translate: Default::default(),
        }
    }
}

mod into_finite_elements {
    use super::*;
    /// A single voxel lattice.
    #[test]
    fn single() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
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
            nel: [1; NSD].into(),
            ..Default::default()
        });
    }
    /// A single voxel lattice scaled up [x, y, z] amount [10.0, 20.0, 30.0]
    #[test]
    fn single_scaled_up() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [0.0, 20.0, 0.0],
                [10.0, 20.0, 0.0],
                [0.0, 0.0, 30.0],
                [10.0, 0.0, 30.0],
                [0.0, 20.0, 30.0],
                [10.0, 20.0, 30.0],
            ],
            file_path: "tests/input/single.spn".to_string(),
            nel: [1; NSD].into(),
            scale: [10.0, 20.0, 30.0].into(),
            ..Default::default()
        });
    }
    /// A single voxel lattice scaled down [x, y, z] amount [0.5, 0.25, 0.125]
    #[test]
    fn single_scaled_down() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [0.0, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [0.0, 0.25, 0.0],
                [0.5, 0.25, 0.0],
                [0.0, 0.0, 0.125],
                [0.5, 0.0, 0.125],
                [0.0, 0.25, 0.125],
                [0.5, 0.25, 0.125],
            ],
            file_path: "tests/input/single.spn".to_string(),
            nel: [1; NSD].into(),
            scale: [0.5, 0.25, 0.125].into(),
            ..Default::default()
        });
    }
    /// A single voxel lattice translated [x, y, z] amount [0.3, 0.6, 0.9]
    #[test]
    fn single_translated_positive() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [0.3, 0.6, 0.9],
                [1.3, 0.6, 0.9],
                [0.3, 1.6, 0.9],
                [1.3, 1.6, 0.9],
                [0.3, 0.6, 1.9],
                [1.3, 0.6, 1.9],
                [0.3, 1.6, 1.9],
                [1.3, 1.6, 1.9],
            ],
            file_path: "tests/input/single.spn".to_string(),
            nel: [1; NSD].into(),
            translate: [0.3, 0.6, 0.9].into(),
            ..Default::default()
        });
    }
    /// A single voxel lattice translated [x, y, z] amount [-1.0, -2.0, -3.0]
    #[test]
    fn single_translated_negative() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [-1.0, -2.0, -3.0],
                [0.0, -2.0, -3.0],
                [-1.0, -1.0, -3.0],
                [0.0, -1.0, -3.0],
                [-1.0, -2.0, -2.0],
                [0.0, -2.0, -2.0],
                [-1.0, -1.0, -2.0],
                [0.0, -1.0, -2.0],
            ],
            file_path: "tests/input/single.spn".to_string(),
            nel: [1; NSD].into(),
            translate: [-1.0, -2.0, -3.0].into(),
            ..Default::default()
        });
    }
    /// A single voxel lattice scaled [10, 11, 12] and translated [0.1, 0.2, 0.3].
    #[test]
    fn single_scaled_and_translated() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [0.1, 0.2, 0.3],
                [10.1, 0.2, 0.3],
                [0.1, 11.2, 0.3],
                [10.1, 11.2, 0.3],
                [0.1, 0.2, 12.3],
                [10.1, 0.2, 12.3],
                [0.1, 11.2, 12.3],
                [10.1, 11.2, 12.3],
            ],
            scale: [10.0, 11.0, 12.0].into(),
            translate: [0.1, 0.2, 0.3].into(),
            file_path: "tests/input/single.spn".to_string(),
            nel: [1; NSD].into(),
            ..Default::default()
        });
    }
    /// A double voxel lattice, coursed along the x-axis.
    #[test]
    fn double_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_node_connectivity: [[1, 2, 5, 4, 7, 8, 11, 10], [2, 3, 6, 5, 8, 9, 12, 11]],
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
            nel: [2, 1, 1].into(),
            ..Default::default()
        });
    }
    /// A double voxel lattice, coursed along the y-axis.
    #[test]
    fn double_y() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_node_connectivity: [[1, 2, 4, 3, 7, 8, 10, 9], [3, 4, 6, 5, 9, 10, 12, 11]],
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
            nel: [1, 2, 1].into(),
            ..Default::default()
        });
    }
    #[test]
    /// A triple voxel lattice, coursed along the x-axis.
    fn triple_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11, 11],
            element_node_connectivity: [
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
            nel: [3, 1, 1].into(),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, coursed along the x-axis.
    #[test]
    fn quadruple_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11, 11, 11],
            element_node_connectivity: [
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
            nel: [4, 1, 1].into(),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, coursed along the x-axis, with two
    /// intermediate voxels in the segmentation being void.
    #[test]
    fn quadruple_2_voids_x() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_node_connectivity: [[1, 2, 6, 5, 9, 10, 14, 13], [3, 4, 8, 7, 11, 12, 16, 15]],
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
            nel: [4, 1, 1].into(),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the two intermediate voxels in the
    /// segmentation being a second block.
    #[test]
    fn quadruple_2_blocks() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 21, 21, 11],
            element_node_connectivity: [
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
            nel: [4, 1, 1].into(),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the two intermediate voxels in the
    /// segmentation being a second block.
    /// The first block is removed from the mesh.
    #[test]
    fn quadruple_2_blocks_remove_1() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [21, 21],
            element_node_connectivity: [[1, 2, 5, 4, 7, 8, 11, 10], [2, 3, 6, 5, 8, 9, 12, 11]],
            element_coordinates: [
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple_2_blocks.spn".to_string(),
            nel: [4, 1, 1].into(),
            remove: Option::Some(vec![11]),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the two intermediate voxels in the
    /// segmentation being a second block.
    /// The second block is removed from the mesh.
    #[test]
    fn quadruple_2_blocks_remove_2() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_node_connectivity: [[1, 2, 6, 5, 9, 10, 14, 13], [3, 4, 8, 7, 11, 12, 16, 15]],
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
            file_path: "tests/input/quadruple_2_blocks.spn".to_string(),
            nel: [4, 1, 1].into(),
            remove: Option::Some(vec![21]),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the first intermediate voxel being
    /// the second block and the second intermediate voxel being void.
    #[test]
    fn quadruple_2_blocks_void() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 21, 11],
            element_node_connectivity: [
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
            nel: [4, 1, 1].into(),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the first intermediate voxel being
    /// the second block and the second intermediate voxel being void.
    #[test]
    fn quadruple_2_blocks_void_remove_0() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 21, 11],
            element_node_connectivity: [
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
            nel: [4, 1, 1].into(),
            remove: Option::Some(vec![0]),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the first intermediate voxel being
    /// the second block and the second intermediate voxel being void.
    /// The first block is removed from the mesh.
    #[test]
    fn quadruple_2_blocks_void_remove_1() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [21],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple_2_blocks_void.spn".to_string(),
            nel: [4, 1, 1].into(),
            remove: Option::Some(vec![0, 11]),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the first intermediate voxel being
    /// the second block and the second intermediate voxel being void.
    /// The second block is removed from the mesh.
    #[test]
    fn quadruple_2_blocks_void_remove_2() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11],
            element_node_connectivity: [[1, 2, 6, 5, 9, 10, 14, 13], [3, 4, 8, 7, 11, 12, 16, 15]],
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
            file_path: "tests/input/quadruple_2_blocks_void.spn".to_string(),
            nel: [4, 1, 1].into(),
            remove: Option::Some(vec![0, 21]),
            ..Default::default()
        });
    }
    /// A quadruple voxel lattice, with the first intermediate voxel being
    /// the second block and the second intermediate voxel being void.
    /// The first and second blocks are removed, the void is retained.
    #[test]
    fn quadruple_2_blocks_void_remove_3() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [0],
            element_node_connectivity: [[1, 2, 4, 3, 5, 6, 8, 7]],
            element_coordinates: [
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
            ],
            file_path: "tests/input/quadruple_2_blocks_void.spn".to_string(),
            nel: [4, 1, 1].into(),
            remove: Option::Some(vec![11, 21]),
            ..Default::default()
        });
    }
    /// A (2 x 2 x 2) voxel cube.
    #[test]
    fn cube() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11, 11, 11, 11, 11, 11, 11, 11],
            element_node_connectivity: [
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
            nel: [2, 2, 2].into(),
            ..Default::default()
        });
    }
    /// A (2 x 2 x 2) voxel cube with two voids and six elements.
    #[test]
    fn cube_multi() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [82, 2, 2, 2, 31, 44],
            element_node_connectivity: [
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
            nel: [2, 2, 2].into(),
            ..Default::default()
        });
    }
    /// A (3 x 3 x 3) voxel cube with a single voxel inclusion
    /// at the center.
    #[test]
    fn cube_with_inclusion() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [
                11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 88, 11, 11, 11, 11, 11, 11, 11,
                11, 11, 11, 11, 11, 11,
            ],
            element_node_connectivity: [
                [1, 2, 6, 5, 17, 18, 22, 21],
                [2, 3, 7, 6, 18, 19, 23, 22],
                [3, 4, 8, 7, 19, 20, 24, 23],
                [5, 6, 10, 9, 21, 22, 26, 25],
                [6, 7, 11, 10, 22, 23, 27, 26],
                [7, 8, 12, 11, 23, 24, 28, 27],
                [9, 10, 14, 13, 25, 26, 30, 29],
                [10, 11, 15, 14, 26, 27, 31, 30],
                [11, 12, 16, 15, 27, 28, 32, 31],
                [17, 18, 22, 21, 33, 34, 38, 37],
                [18, 19, 23, 22, 34, 35, 39, 38],
                [19, 20, 24, 23, 35, 36, 40, 39],
                [21, 22, 26, 25, 37, 38, 42, 41],
                [22, 23, 27, 26, 38, 39, 43, 42],
                [23, 24, 28, 27, 39, 40, 44, 43],
                [25, 26, 30, 29, 41, 42, 46, 45],
                [26, 27, 31, 30, 42, 43, 47, 46],
                [27, 28, 32, 31, 43, 44, 48, 47],
                [33, 34, 38, 37, 49, 50, 54, 53],
                [34, 35, 39, 38, 50, 51, 55, 54],
                [35, 36, 40, 39, 51, 52, 56, 55],
                [37, 38, 42, 41, 53, 54, 58, 57],
                [38, 39, 43, 42, 54, 55, 59, 58],
                [39, 40, 44, 43, 55, 56, 60, 59],
                [41, 42, 46, 45, 57, 58, 62, 61],
                [42, 43, 47, 46, 58, 59, 63, 62],
                [43, 44, 48, 47, 59, 60, 64, 63],
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
                [0.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [3.0, 2.0, 0.0],
                [0.0, 3.0, 0.0],
                [1.0, 3.0, 0.0],
                [2.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [2.0, 0.0, 1.0],
                [3.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 1.0, 1.0],
                [3.0, 1.0, 1.0],
                [0.0, 2.0, 1.0],
                [1.0, 2.0, 1.0],
                [2.0, 2.0, 1.0],
                [3.0, 2.0, 1.0],
                [0.0, 3.0, 1.0],
                [1.0, 3.0, 1.0],
                [2.0, 3.0, 1.0],
                [3.0, 3.0, 1.0],
                [0.0, 0.0, 2.0],
                [1.0, 0.0, 2.0],
                [2.0, 0.0, 2.0],
                [3.0, 0.0, 2.0],
                [0.0, 1.0, 2.0],
                [1.0, 1.0, 2.0],
                [2.0, 1.0, 2.0],
                [3.0, 1.0, 2.0],
                [0.0, 2.0, 2.0],
                [1.0, 2.0, 2.0],
                [2.0, 2.0, 2.0],
                [3.0, 2.0, 2.0],
                [0.0, 3.0, 2.0],
                [1.0, 3.0, 2.0],
                [2.0, 3.0, 2.0],
                [3.0, 3.0, 2.0],
                [0.0, 0.0, 3.0],
                [1.0, 0.0, 3.0],
                [2.0, 0.0, 3.0],
                [3.0, 0.0, 3.0],
                [0.0, 1.0, 3.0],
                [1.0, 1.0, 3.0],
                [2.0, 1.0, 3.0],
                [3.0, 1.0, 3.0],
                [0.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [2.0, 2.0, 3.0],
                [3.0, 2.0, 3.0],
                [0.0, 3.0, 3.0],
                [1.0, 3.0, 3.0],
                [2.0, 3.0, 3.0],
                [3.0, 3.0, 3.0],
            ],
            file_path: "tests/input/cube_with_inclusion.spn".to_string(),
            nel: [3, 3, 3].into(),
            ..Default::default()
        });
    }
    /// A minimal letter F example.
    #[test]
    fn letter_f() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [11; 8],
            element_node_connectivity: [
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
            nel: [3, 5, 1].into(),
            ..Default::default()
        });
    }
    /// A three dimensional variation of the letter F, in a non-standard
    /// orientation.
    #[test]
    fn letter_f_3d() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [1; 39],
            element_node_connectivity: [
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
            nel: [4, 5, 3].into(),
            ..Default::default()
        });
    }
    // A random 5x5x5 domain composed void and two materials.
    #[test]
    fn sparse() {
        assert_fem_data_from_spn_eq_gold(Gold {
            element_blocks: [
                2, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2,
                1, 1, 2, 1, 1, 1, 2, 2, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 1, 1, 1,
                2, 1,
            ],
            element_node_connectivity: [
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
            nel: [5; NSD].into(),
            ..Default::default()
        });
    }
}

mod defeature {
    use super::*;
    #[test]
    fn cube_with_inclusion() {
        let nel = 3;
        let voxels =
            Voxels::from_spn("tests/input/cube_with_inclusion.spn", [nel; NSD].into()).unwrap();
        let voxels = voxels.defeature(2);
        voxels.get_data().outer_iter().take(nel).for_each(|a| {
            a.outer_iter()
                .take(nel)
                .for_each(|b| b.iter().take(nel).for_each(|&c| assert_eq!(c, 11)))
        });
        voxels.get_data().outer_iter().skip(nel).for_each(|a| {
            a.outer_iter()
                .skip(nel)
                .for_each(|b| b.iter().skip(nel).for_each(|&c| assert_eq!(c, 0)))
        })
    }
}

mod from_npy {
    use super::*;
    #[test]
    #[cfg(not(target_os = "windows"))]
    #[should_panic(expected = "No such file or directory")]
    fn file_nonexistent() {
        Voxels::from_npy("tests/input/f_file_nonexistent.npy")
            .map_err(|e| e.to_string())
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "error parsing header: start does not match magic string")]
    fn file_unreadable() {
        Voxels::from_npy("tests/input/letter_f_3d.txt")
            .map_err(|e| e.to_string())
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "error parsing header: start does not match magic string")]
    fn file_unopenable() {
        Voxels::from_npy("tests/input/encrypted.npy")
            .map_err(|e| e.to_string())
            .unwrap();
    }
    #[test]
    fn success() {
        let voxels = Voxels::from_npy("tests/input/letter_f_3d.npy").unwrap();
        assert_data_eq_gold(voxels);
    }
    #[test]
    #[should_panic(expected = "Need to specify scale > 0.")]
    fn xscale_positive() {
        let voxels = Voxels::from_npy("tests/input/letter_f_3d.npy").unwrap();
        voxels
            .into_finite_elements(None, [0.0, 1.0, 1.0].into(), [0.0, 0.0, 0.0].into())
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "Need to specify scale > 0.")]
    fn yscale_positive() {
        let voxels = Voxels::from_npy("tests/input/letter_f_3d.npy").unwrap();
        voxels
            .into_finite_elements(None, [1.0, 0.0, 1.0].into(), [0.0, 0.0, 0.0].into())
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "Need to specify scale > 0.")]
    fn zscale_positive() {
        let voxels = Voxels::from_npy("tests/input/letter_f_3d.npy").unwrap();
        voxels
            .into_finite_elements(None, [1.0, 1.0, 0.0].into(), [0.0, 0.0, 0.0].into())
            .unwrap();
    }
}

mod from_spn {
    use super::*;
    #[test]
    #[cfg(not(target_os = "windows"))]
    #[should_panic(expected = "No such file or directory")]
    fn file_nonexistent() {
        Voxels::from_spn("tests/input/f_file_nonexistent.spn", [4, 5, 3].into())
            .map_err(|e| e.to_string())
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "ParseIntError { kind: InvalidDigit }")]
    fn file_unreadable() {
        Voxels::from_spn("tests/input/letter_f_3d.txt", [4, 5, 3].into())
            .map_err(|e| e.to_string())
            .unwrap();
    }
    #[test]
    #[should_panic(expected = "Need to specify nel > 0.")]
    fn nelx_positive() {
        Voxels::from_spn("tests/input/single.spn", [0, 1, 1].into()).unwrap();
    }
    #[test]
    #[should_panic(expected = "Need to specify nel > 0.")]
    fn nely_positive() {
        Voxels::from_spn("tests/input/single.spn", [1, 0, 1].into()).unwrap();
    }
    #[test]
    #[should_panic(expected = "Need to specify nel > 0.")]
    fn nelz_positive() {
        Voxels::from_spn("tests/input/single.spn", [1, 1, 0].into()).unwrap();
    }
    #[test]
    fn success() {
        let voxels = Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        assert_data_eq_gold(voxels);
    }
}

mod write_npy {
    use super::*;
    #[test]
    fn letter_f_3d() {
        let voxels_from_spn =
            Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        voxels_from_spn.write_npy("target/letter_f_3d.npy").unwrap();
        let voxels_from_npy = Voxels::from_npy("target/letter_f_3d.npy").unwrap();
        assert_data_eq(voxels_from_npy, voxels_from_spn);
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    #[should_panic(expected = "No such file or directory")]
    fn no_such_directory() {
        let voxels = Voxels::from_spn("tests/input/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        voxels.write_npy("no_such_directory/foo.npy").unwrap();
    }
    #[test]
    fn sparse() {
        let voxels_from_spn = Voxels::from_spn("tests/input/sparse.spn", [5, 5, 5].into()).unwrap();
        voxels_from_spn.write_npy("target/sparse.npy").unwrap();
        let voxels_from_npy = Voxels::from_npy("target/sparse.npy").unwrap();
        assert_data_eq(voxels_from_npy, voxels_from_spn);
    }
}

mod write_spn {
    use super::*;
    #[test]
    fn letter_f_3d() {
        let voxels_from_npy = Voxels::from_npy("tests/input/letter_f_3d.npy").unwrap();
        voxels_from_npy.write_spn("target/letter_f_3d.spn").unwrap();
        let voxels_from_spn = Voxels::from_spn("target/letter_f_3d.spn", [4, 5, 3].into()).unwrap();
        assert_data_eq(voxels_from_npy, voxels_from_spn);
    }
    #[test]
    #[cfg(not(target_os = "windows"))]
    #[should_panic(expected = "No such file or directory")]
    fn no_such_directory() {
        let voxels = Voxels::from_npy("tests/input/letter_f_3d.npy").unwrap();
        voxels.write_spn("no_such_directory/foo.spn").unwrap();
    }
    #[test]
    fn sparse() {
        let voxels_from_npy = Voxels::from_npy("tests/input/sparse.npy").unwrap();
        voxels_from_npy.write_spn("target/sparse.spn").unwrap();
        let voxels_from_spn = Voxels::from_spn("target/sparse.spn", [5, 5, 5].into()).unwrap();
        assert_data_eq(voxels_from_npy, voxels_from_spn);
    }
}
