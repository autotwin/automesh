use super::{element_connectivity_node_renumbering, filter_voxel_data, Voxels};

const NUM_ELEMENTS: usize = 39;

const BLOCKS_GOLD: [usize; NUM_ELEMENTS] = [1; NUM_ELEMENTS];
const VOXELS_GOLD: [[usize; 3]; NUM_ELEMENTS] = [
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
fn connectivity_node_renumbering() {
    let mut element_connectivity = vec![vec![1, 6, 3, 4], vec![3, 9, 6, 11], vec![13, 17, 16, 19]];
    let element_connectivity_gold = vec![vec![1, 4, 2, 3], vec![2, 5, 4, 6], vec![7, 9, 8, 10]];
    element_connectivity_node_renumbering(&mut element_connectivity);
    element_connectivity
        .iter()
        .flatten()
        .zip(element_connectivity_gold.iter().flatten())
        .for_each(|(entry, gold)| assert_eq!(entry, gold));
}

#[test]
fn filter() {
    let spn = Voxels::from_npy("tests/input/letter_f_3d.npy");
    let (filtered_voxel_data, element_blocks) = filter_voxel_data(spn.get_data());
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