use super::super::{Faces, HEX, Indices, Octree};

pub const DATA: [[usize; 11]; 12] = [
    [0, 1, 5, 5, 4, 0, 5, 7, 10, 0, 5],
    [0, 4, 1, 1, 5, 4, 0, 3, 15, 10, 0],
    [0, 3, 4, 0, 1, 5, 0, 2, 5, 15, 10],
    [0, 5, 3, 4, 0, 1, 10, 6, 0, 5, 15],
    [1, 2, 5, 7, 5, 1, 15, 6, 10, 0, 5],
    [1, 4, 2, 3, 7, 5, 5, 2, 15, 10, 0],
    [1, 0, 4, 1, 3, 7, 5, 0, 5, 15, 10],
    [2, 3, 5, 6, 7, 3, 10, 4, 15, 5, 0],
    [2, 4, 3, 2, 6, 7, 5, 0, 10, 15, 5],
    [2, 1, 4, 3, 2, 6, 15, 1, 0, 10, 15],
    [3, 0, 5, 4, 6, 2, 0, 5, 15, 5, 0],
    [3, 2, 4, 2, 0, 4, 10, 3, 0, 10, 15],
];

#[allow(clippy::too_many_arguments)]
pub fn template(
    face_index: usize,
    face_index_a: usize,
    face_index_b: usize,
    cell_subcell_index: usize,
    cell_subcell_a_index: usize,
    cell_subcell_ab_index: usize,
    cell_subsubcell_b_index: usize,
    cell_subcell_c_index: usize,
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
            if let Some((cell_a_subcells, cell_a_faces)) =
                tree.cell_contains_leaves(&tree[cell_a_index])
            {
                if let Some(cell_ab_index) = cell_a_faces[face_index_b] {
                    if let Some(cell_b_subsubcells) = tree.cell_subcell_contains_leaves(
                        &tree[cell_b_index],
                        face_index_b,
                        cell_subsubcell_b_index,
                    ) {
                        if let Some((cell_ab_subcells, cell_ab_faces)) =
                            tree.cell_contains_leaves(&tree[cell_ab_index])
                        {
                            if let Some(cell_c_index) = cell_faces[face_index] {
                                if let Some((cell_c_subcells, cell_c_faces)) =
                                    tree.cell_contains_leaves(&tree[cell_c_index])
                                {
                                    if let Some(cell_c_a_index) = cell_a_faces[face_index] {
                                        if let Some(cell_c_a_subsubcells) = tree
                                            .cell_subcell_contains_leaves(
                                                &tree[cell_c_a_index],
                                                face_index,
                                                cell_subsubcell_c_a_index,
                                            )
                                        {
                                            if let Some(cell_c_b_index) = cell_c_faces[face_index_b]
                                            {
                                                if let Some(cell_c_b_subsubcells) = tree
                                                    .cell_subcell_contains_leaves(
                                                        &tree[cell_c_b_index],
                                                        face_index,
                                                        cell_subsubcell_c_b_index,
                                                    )
                                                {
                                                    if let Some(cell_c_ab_index) =
                                                        cell_ab_faces[face_index]
                                                    {
                                                        tree.cell_subcell_contains_leaves(
                                                            &tree[cell_c_ab_index],
                                                            face_index,
                                                            cell_subsubcell_c_ab_index,
                                                        )
                                                        .map(|cell_c_ab_subsubcells| {
                                                            [
                                                                cells_nodes[cell_subcells
                                                                    [cell_subcell_index]],
                                                                cells_nodes[cell_a_subcells
                                                                    [cell_subcell_a_index]],
                                                                cells_nodes[cell_ab_subcells
                                                                    [cell_subcell_ab_index]],
                                                                cells_nodes[cell_b_subsubcells
                                                                    [cell_subsubcell_b_index]],
                                                                cells_nodes[cell_c_subcells
                                                                    [cell_subcell_c_index]],
                                                                cells_nodes[cell_c_a_subsubcells
                                                                    [cell_subsubcell_c_a_index]],
                                                                cells_nodes[cell_c_ab_subsubcells
                                                                    [cell_subsubcell_c_ab_index]],
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
