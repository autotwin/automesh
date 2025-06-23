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
                0,
                3,
                5,
                4,
                0,
                10,
                10,
                15,
                0,
                0,
                5,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                1,
                0,
                5,
                5,
                1,
                10,
                15,
                10,
                0,
                5,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                2,
                1,
                5,
                7,
                3,
                15,
                15,
                10,
                5,
                5,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                3,
                2,
                5,
                6,
                2,
                15,
                10,
                15,
                5,
                0,
                5,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                4,
                1,
                2,
                3,
                1,
                15,
                5,
                15,
                5,
                0,
                10,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                2,
                4,
                1,
                3,
                2,
                5,
                15,
                5,
                0,
                10,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                3,
                4,
                2,
                2,
                0,
                5,
                10,
                15,
                0,
                0,
                5,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                0,
                4,
                3,
                0,
                1,
                0,
                0,
                10,
                5,
                5,
                15,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                5,
                2,
                1,
                7,
                6,
                15,
                15,
                5,
                10,
                10,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                0,
                5,
                1,
                5,
                4,
                15,
                5,
                15,
                10,
                0,
                10,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                5,
                3,
                2,
                6,
                4,
                10,
                15,
                5,
                0,
                10,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                1,
                5,
                2,
                7,
                5,
                15,
                15,
                10,
                10,
                5,
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
    cell_subcell_c_index: usize,
    face_a_subsubcell_index: usize,
    face_b_subsubcell_index: usize,
    face_ab_subsubcell_index: usize,
    face_c_a_subsubcell_index: usize,
    face_c_b_subsubcell_index: usize,
    face_c_ab_subsubcell_index: usize,
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
                    if let Some((cell_c_subcells, cell_c_faces)) =
                        tree.cell_contains_leaves(&tree[cell_c_index])
                    {
                        if let Some(cell_c_a_index) = cell_c_faces[face_index_a] {
                            if let Some(cell_c_b_index) = cell_c_faces[face_index_b] {
                                if let Some(cell_c_ab_index) =
                                    tree[cell_c_a_index].get_faces()[face_index_b]
                                {
                                    if let Some(face_a_subsubcells) = tree
                                        .cell_subcells_contain_leaves(
                                            &tree[cell_a_index],
                                            face_index_a,
                                        )
                                    {
                                        if let Some(face_b_subsubcells) = tree
                                            .cell_subcells_contain_leaves(
                                                &tree[cell_b_index],
                                                face_index_b,
                                            )
                                        {
                                            if let Some(face_ab_subsubcells) = tree
                                                .cell_subcells_contain_cells(
                                                    &tree[cell_ab_index],
                                                    face_index_b,
                                                )
                                            {
                                                if let Some(face_c_a_subsubcells) = tree
                                                    .cell_subcells_contain_leaves(
                                                        &tree[cell_c_a_index],
                                                        face_index_a,
                                                    )
                                                {
                                                    if let Some(face_c_b_subsubcells) = tree
                                                        .cell_subcells_contain_leaves(
                                                            &tree[cell_c_b_index],
                                                            face_index_b,
                                                        )
                                                    {
                                                        if let Some(face_c_ab_subsubcells) = tree
                                                            .cell_subcells_contain_cells(
                                                                &tree[cell_c_ab_index],
                                                                face_index_b,
                                                            )
                                                        {
                                                            element_node_connectivity.push([
                                                                cells_nodes[cell_subcells
                                                                    [cell_subcell_index]],
                                                                cells_nodes[face_b_subsubcells
                                                                    [face_b_subsubcell_index]],
                                                                cells_nodes[face_ab_subsubcells
                                                                    [face_ab_subsubcell_index]],
                                                                cells_nodes[face_a_subsubcells
                                                                    [face_a_subsubcell_index]],
                                                                cells_nodes[cell_c_subcells
                                                                    [cell_subcell_c_index]],
                                                                cells_nodes[face_c_b_subsubcells
                                                                    [face_c_b_subsubcell_index]],
                                                                cells_nodes[face_c_ab_subsubcells
                                                                    [face_c_ab_subsubcell_index]],
                                                                cells_nodes[face_c_a_subsubcells
                                                                    [face_c_a_subsubcell_index]],
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
