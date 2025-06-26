use super::super::{Faces, HEX, Indices, Octree};

pub const DATA: [[usize; 11]; 4] = [
    [0, 1, 5, 5, 10, 0, 5, 15, 10, 2, 5],
    [1, 2, 5, 7, 15, 5, 15, 15, 10, 0, 5],
    [2, 3, 5, 6, 15, 5, 10, 10, 15, 1, 0],
    [3, 0, 5, 4, 10, 0, 0, 10, 15, 3, 0],
];

#[allow(clippy::too_many_arguments)]
pub fn template(
    face_index: usize,
    face_index_a: usize,
    face_index_b: usize,
    cell_subcell_index: usize,
    cell_subsubcell_a_index: usize,
    cell_subsubcell_ab_index: usize,
    cell_subsubcell_b_index: usize,
    cell_subsubcell_c_index: usize,
    cell_subsubcell_c_a_index: usize,
    cell_subcell_c_ab_index: usize,
    cell_subsubcell_c_b_index: usize,
    cell_faces: &Faces,
    cell_subcells: &Indices,
    cells_nodes: &[usize],
    tree: &Octree,
) -> Option<[usize; HEX]> {
    if let Some(cell_a_index) = cell_faces[face_index_a] {
        if let Some(cell_b_index) = cell_faces[face_index_b] {
            if let Some(cell_a_subsubcells) = tree.cell_subcell_contains_leaves(
                &tree[cell_a_index],
                face_index_a,
                cell_subsubcell_a_index,
            ) {
                if let Some(cell_ab_index) = tree[cell_a_index].get_faces()[face_index_b] {
                    if let Some(cell_b_subsubcells) = tree.cell_subcell_contains_leaves(
                        &tree[cell_b_index],
                        face_index_b,
                        cell_subsubcell_b_index,
                    ) {
                        if let Some(cell_ab_subsubcells) = tree.cell_subcell_contains_leaves(
                            &tree[cell_ab_index],
                            face_index_a,
                            cell_subsubcell_ab_index,
                        ) {
                            if let Some(cell_c_index) = cell_faces[face_index] {
                                if let Some(cell_c_subsubcells) = tree.cell_subcell_contains_leaves(
                                    &tree[cell_c_index],
                                    face_index,
                                    cell_subsubcell_c_index,
                                ) {
                                    if let Some(cell_c_a_index) =
                                        tree[cell_c_index].get_faces()[face_index_a]
                                    {
                                        if let Some(cell_c_a_subsubcells) = tree
                                            .cell_subcell_contains_leaves(
                                                &tree[cell_c_a_index],
                                                face_index,
                                                cell_subsubcell_c_a_index,
                                            )
                                        {
                                            if let Some(cell_c_b_index) =
                                                tree[cell_b_index].get_faces()[face_index]
                                            {
                                                if let Some(cell_c_b_subsubcells) = tree
                                                    .cell_subcell_contains_leaves(
                                                        &tree[cell_c_b_index],
                                                        face_index,
                                                        cell_subsubcell_c_b_index,
                                                    )
                                                {
                                                    if let Some(cell_c_ab_index) = tree
                                                        [cell_c_a_index]
                                                        .get_faces()[face_index_b]
                                                    {
                                                        tree.cell_contains_leaves(
                                                            &tree[cell_c_ab_index],
                                                        )
                                                        .map(|(cell_c_ab_subcells, _)| {
                                                            [
                                                                cells_nodes[cell_subcells
                                                                    [cell_subcell_index]],
                                                                cells_nodes[cell_a_subsubcells
                                                                    [cell_subsubcell_a_index]],
                                                                cells_nodes[cell_ab_subsubcells
                                                                    [cell_subsubcell_ab_index]],
                                                                cells_nodes[cell_b_subsubcells
                                                                    [cell_subsubcell_b_index]],
                                                                cells_nodes[cell_c_subsubcells
                                                                    [cell_subsubcell_c_index]],
                                                                cells_nodes[cell_c_a_subsubcells
                                                                    [cell_subsubcell_c_a_index]],
                                                                cells_nodes[cell_c_ab_subcells
                                                                    [cell_subcell_c_ab_index]],
                                                                cells_nodes[cell_c_b_subsubcells
                                                                    [cell_subsubcell_c_b_index]],
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
