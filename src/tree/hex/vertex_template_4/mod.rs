use super::super::{Faces, HexConnectivity, Indices, Octree};

pub fn apply(
    cells_nodes: &[usize],
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
) {
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, cell_faces)| {
            template(
                // (1, 5, 2) = (0, 1, 4) = (5, 0, 3)
                1,
                5,
                2,
                7,
                6,
                2,
                3,
                15,
                4,
                0,
                1,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (2, 3, 4) = (3, 5, 0) = (5, 2, 1)
                2,
                3,
                4,
                2,
                0,
                1,
                3,
                10,
                4,
                5,
                7,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (2, 5, 3) = (5, 1, 2) = (1, 2, 4)
                2,
                5,
                3,
                6,
                4,
                0,
                2,
                15,
                5,
                1,
                3,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (0, 5, 1) = (5, 2, 1) = (2, 0, 4)
                0,
                5,
                1,
                5,
                7,
                3,
                1,
                10,
                6,
                2,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (1, 0, 5) = (0, 4, 1) = (4, 1, 2)
                1,
                0,
                5,
                5,
                4,
                6,
                7,
                5,
                0,
                2,
                3,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (2, 1, 5) = (1, 2, 4) = (5, 1, 0)
                2,
                1,
                5,
                7,
                5,
                4,
                6,
                15,
                1,
                0,
                2,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (0, 3, 5) = (3, 4, 2) = (4, 0, 1)
                0,
                3,
                5,
                4,
                6,
                7,
                5,
                0,
                2,
                3,
                1,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                // (3, 2, 5) = (2, 4, 1) = (4, 3, 0)
                3,
                2,
                5,
                6,
                7,
                5,
                4,
                10,
                3,
                1,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
        })
}

#[allow(clippy::too_many_arguments)]
fn template(
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
    element_node_connectivity: &mut HexConnectivity,
) {
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
                                                        if let Some((cell_c_ab_subcells, _)) = tree
                                                            .cell_contains_leaves(
                                                                &tree[cell_c_ab_index],
                                                            )
                                                        {
                                                            element_node_connectivity.push([
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
                                                            ])
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
