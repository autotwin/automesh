use super::super::{Faces, HEX, Indices, Octree};

pub const DATA: [[usize; 11]; 8] = [
    [1, 5, 2, 7, 15, 10, 15, 15, 10, 0, 5],
    [2, 5, 3, 6, 10, 0, 10, 15, 15, 5, 15],
    [3, 5, 0, 4, 10, 5, 0, 10, 15, 15, 10],
    [0, 5, 1, 5, 15, 15, 5, 10, 10, 10, 0],
    [1, 4, 0, 1, 0, 0, 5, 5, 5, 10, 15],
    [0, 4, 3, 0, 0, 10, 0, 0, 5, 15, 5],
    [3, 4, 2, 2, 5, 15, 10, 0, 0, 5, 0],
    [2, 4, 1, 3, 5, 5, 15, 5, 0, 0, 10],
];

#[allow(clippy::too_many_arguments)]
pub fn template(
    face_index_a: usize,
    face_index_b: usize,
    face_index_c: usize,
    cell_subcell_index: usize,
    cell_subsubcell_a_index: usize,
    cell_subsubcell_ab_index: usize,
    cell_subsubcell_b_index: usize,
    cell_subsubcell_c_index: usize,
    cell_subsubcell_c_a_index: usize,
    cell_subsubcell_c_ab_index: usize,
    cell_subsubcell_c_b_index: usize,
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
                                if let Some(cell_a_subsubcells) = tree.cell_subcell_contains_leaves(
                                    &tree[cell_a_index],
                                    face_index_a,
                                    cell_subsubcell_a_index,
                                ) {
                                    if let Some(cell_b_subsubcells) = tree
                                        .cell_subcell_contains_leaves(
                                            &tree[cell_b_index],
                                            face_index_b,
                                            cell_subsubcell_b_index,
                                        )
                                    {
                                        if let Some(cell_ab_subsubcells) = tree
                                            .cell_subcell_contains_leaves(
                                                &tree[cell_ab_index],
                                                face_index_b,
                                                cell_subsubcell_ab_index,
                                            )
                                        {
                                            if let Some(cell_c_subsubcells) = tree
                                                .cell_subcell_contains_leaves(
                                                    &tree[cell_c_index],
                                                    face_index_c,
                                                    cell_subsubcell_c_index,
                                                )
                                            {
                                                if let Some(cell_c_a_subsubcells) = tree
                                                    .cell_subcell_contains_leaves(
                                                        &tree[cell_c_a_index],
                                                        face_index_a,
                                                        cell_subsubcell_c_a_index,
                                                    )
                                                {
                                                    if let Some(cell_c_b_subsubcells) = tree
                                                        .cell_subcell_contains_leaves(
                                                            &tree[cell_c_b_index],
                                                            face_index_b,
                                                            cell_subsubcell_c_b_index,
                                                        )
                                                    {
                                                        tree.cell_subcell_contains_leaves(
                                                            &tree[cell_c_ab_index],
                                                            face_index_b,
                                                            cell_subsubcell_c_ab_index,
                                                        )
                                                        .map(|cell_c_ab_subsubcells| {
                                                            [
                                                                cells_nodes[cell_c_subsubcells
                                                                    [cell_subsubcell_c_index]],
                                                                cells_nodes[cell_c_a_subsubcells
                                                                    [cell_subsubcell_c_a_index]],
                                                                cells_nodes[cell_c_ab_subsubcells
                                                                    [cell_subsubcell_c_ab_index]],
                                                                cells_nodes[cell_c_b_subsubcells
                                                                    [cell_subsubcell_c_b_index]],
                                                                cells_nodes[cell_subcells
                                                                    [cell_subcell_index]],
                                                                cells_nodes[cell_a_subsubcells
                                                                    [cell_subsubcell_a_index]],
                                                                cells_nodes[cell_ab_subsubcells
                                                                    [cell_subsubcell_ab_index]],
                                                                cells_nodes[cell_b_subsubcells
                                                                    [cell_subsubcell_b_index]],
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
