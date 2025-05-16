use crate::{NSD, Remove, Scale, Translate};

use super::{Nel, Voxels, filter_voxel_data};

const NUM_ELEMENTS: usize = 39;

const BLOCKS_GOLD: [u8; NUM_ELEMENTS] = [1; NUM_ELEMENTS];
const VOXELS_GOLD: [[usize; NSD]; NUM_ELEMENTS] = [
    [0, 0, 0],
    [1, 0, 0],
    [2, 0, 0],
    [3, 0, 0],
    [0, 1, 0],
    [1, 1, 0],
    [2, 1, 0],
    [3, 1, 0],
    [0, 2, 0],
    [1, 2, 0],
    [2, 2, 0],
    [3, 2, 0],
    [0, 3, 0],
    [1, 3, 0],
    [2, 3, 0],
    [3, 3, 0],
    [0, 4, 0],
    [1, 4, 0],
    [2, 4, 0],
    [3, 4, 0],
    [0, 0, 1],
    [0, 1, 1],
    [0, 2, 1],
    [1, 2, 1],
    [2, 2, 1],
    [3, 2, 1],
    [0, 3, 1],
    [0, 4, 1],
    [1, 4, 1],
    [2, 4, 1],
    [3, 4, 1],
    [0, 0, 2],
    [0, 1, 2],
    [0, 2, 2],
    [0, 3, 2],
    [0, 4, 2],
    [1, 4, 2],
    [2, 4, 2],
    [3, 4, 2],
];

#[test]
fn filter() {
    let spn = Voxels::from_npy(
        "tests/input/letter_f_3d.npy",
        Remove::default(),
        Scale::default(),
        Translate::default(),
    )
    .unwrap();
    let (filtered_voxel_data, element_blocks) =
        filter_voxel_data(spn.get_data().to_owned(), Remove::from(vec![0]));
    assert_eq!(element_blocks.len(), NUM_ELEMENTS);
    BLOCKS_GOLD
        .iter()
        .zip(element_blocks.iter())
        .for_each(|(gold_n, block_n)| assert_eq!(gold_n, block_n));
    assert_eq!(filtered_voxel_data.len(), NUM_ELEMENTS);
    VOXELS_GOLD
        .iter()
        .flatten()
        .zip(filtered_voxel_data.iter().flatten())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
}

mod nel {
    use super::*;
    mod from_input {
        use super::*;
        #[test]
        #[should_panic(expected = "Argument nelx was required but was not provided")]
        fn missing_x() {
            let _ = Nel::from_input([None, None, None]).unwrap();
        }
        #[test]
        #[should_panic(expected = "Argument nely was required but was not provided")]
        fn missing_y() {
            let _ = Nel::from_input([Some(1), None, None]).unwrap();
        }
        #[test]
        #[should_panic(expected = "Argument nelz was required but was not provided")]
        fn missing_z() {
            let _ = Nel::from_input([Some(1), Some(1), None]).unwrap();
        }
        #[test]
        fn success() {
            let _ = Nel::from_input([Some(1), Some(1), Some(1)]).unwrap();
        }
    }
    mod from_slice {
        use super::*;
        #[test]
        #[should_panic(expected = "Need to specify nel > 0")]
        fn nonzero() {
            let _ = Nel::from(&[0, 1, 2][..]);
        }
        #[test]
        fn success() {
            let _ = Nel::from(&[1, 2, 3][..]);
        }
    }
    mod from_iter {
        use super::*;
        #[test]
        #[should_panic(expected = "Need to specify nel > 0")]
        fn nonzero() {
            let _ = [0, 2, 3].into_iter().collect::<Nel>();
        }
        #[test]
        fn success() {
            let _ = [1, 2, 3].into_iter().collect::<Nel>();
        }
    }
}
