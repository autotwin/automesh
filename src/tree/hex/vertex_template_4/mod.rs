use super::super::{Faces, HEX, Indices, Octree};

pub const DATA: [[usize; 11]; 8] = [
    [1, 5, 2, 7, 6, 2, 3, 15, 4, 0, 1], // (1, 5, 2) = (0, 1, 4) = (5, 0, 3)
    [2, 3, 4, 2, 0, 1, 3, 10, 4, 5, 7], // (2, 3, 4) = (3, 5, 0) = (5, 2, 1)
    [2, 5, 3, 6, 4, 0, 2, 15, 5, 1, 3], // (2, 5, 3) = (5, 1, 2) = (1, 2, 4)
    [0, 5, 1, 5, 7, 3, 1, 10, 6, 2, 0], // (0, 5, 1) = (5, 2, 1) = (2, 0, 4)
    [1, 0, 5, 5, 4, 6, 7, 5, 0, 2, 3],  // (1, 0, 5) = (0, 4, 1) = (4, 1, 2)
    [2, 1, 5, 7, 5, 4, 6, 15, 1, 0, 2], // (2, 1, 5) = (1, 2, 4) = (5, 1, 0)
    [0, 3, 5, 4, 6, 7, 5, 0, 2, 3, 1],  // (0, 3, 5) = (3, 4, 2) = (4, 0, 1)
    [3, 2, 5, 6, 7, 5, 4, 10, 3, 1, 0], // (3, 2, 5) = (2, 4, 1) = (4, 3, 0)
];

#[allow(clippy::too_many_arguments)]
pub fn template(
    face_index_a: usize,
    face_index_b: usize,
    face_index_c: usize,
    cell_subcell_index: usize,
    cell_subcell_a_index: usize,
    cell_subcell_ab_index: usize,
    cell_subcell_b_index: usize,
    cell_subsubcell_c_index: usize,
    cell_subcell_c_a_index: usize,
    cell_subcell_c_ab_index: usize,
    cell_subcell_c_b_index: usize,
    cell_faces: &Faces,
    cell_subcells: &Indices,
    cells_nodes: &[usize],
    tree: &Octree,
) -> Option<[usize; HEX]> {
    if let Some(cell_a_index) = cell_faces[face_index_a] {
        if let Some(cell_b_index) = cell_faces[face_index_b] {
            if let Some(cell_ab_index) = tree[cell_a_index].get_faces()[face_index_b] {
                if let Some(cell_c_index) = cell_faces[face_index_c] {
                    if let Some(cell_c_a_index) = tree[cell_c_index].get_faces()[face_index_a] {
                        if let Some(cell_c_b_index) = tree[cell_c_index].get_faces()[face_index_b] {
                            if let Some(cell_c_ab_index) =
                                tree[cell_c_a_index].get_faces()[face_index_b]
                            {
                                if let Some((cell_a_subcells, _)) =
                                    tree.cell_contains_leaves(&tree[cell_a_index])
                                {
                                    if let Some((cell_b_subcells, _)) =
                                        tree.cell_contains_leaves(&tree[cell_b_index])
                                    {
                                        if let Some((cell_ab_subcells, _)) =
                                            tree.cell_contains_leaves(&tree[cell_ab_index])
                                        {
                                            if let Some(cell_c_subsubcells) = tree
                                                .cell_subcell_contains_leaves(
                                                    &tree[cell_c_index],
                                                    face_index_c,
                                                    cell_subsubcell_c_index,
                                                )
                                            {
                                                if let Some((cell_c_a_subcells, _)) =
                                                    tree.cell_contains_leaves(&tree[cell_c_a_index])
                                                {
                                                    if let Some((cell_c_b_subcells, _)) = tree
                                                        .cell_contains_leaves(&tree[cell_c_b_index])
                                                    {
                                                        tree.cell_contains_leaves(
                                                            &tree[cell_c_ab_index],
                                                        )
                                                        .map(|(cell_c_ab_subcells, _)| {
                                                            [
                                                                cells_nodes[cell_c_subsubcells
                                                                    [cell_subsubcell_c_index]],
                                                                cells_nodes[cell_c_a_subcells
                                                                    [cell_subcell_c_a_index]],
                                                                cells_nodes[cell_c_ab_subcells
                                                                    [cell_subcell_c_ab_index]],
                                                                cells_nodes[cell_c_b_subcells
                                                                    [cell_subcell_c_b_index]],
                                                                cells_nodes[cell_subcells
                                                                    [cell_subcell_index]],
                                                                cells_nodes[cell_a_subcells
                                                                    [cell_subcell_a_index]],
                                                                cells_nodes[cell_ab_subcells
                                                                    [cell_subcell_ab_index]],
                                                                cells_nodes[cell_b_subcells
                                                                    [cell_subcell_b_index]],
                                                            ]
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
