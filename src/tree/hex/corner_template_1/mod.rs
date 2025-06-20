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
                1,
                5,
                5,
                4,
                1,
                0,
                15,
                10,
                5,
                0,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                1,
                2,
                5,
                7,
                5,
                3,
                1,
                15,
                10,
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
                3,
                5,
                6,
                7,
                2,
                3,
                10,
                15,
                0,
                5,
                cell_faces,
                cell_subcells,
                cells_nodes,
                tree,
                element_node_connectivity,
            );
            template(
                3,
                0,
                5,
                4,
                6,
                0,
                2,
                10,
                15,
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
                0,
                3,
                0,
                2,
                1,
                3,
                0,
                10,
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
                0,
                1,
                5,
                7,
                4,
                6,
                5,
                15,
                0,
                10,
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
    face_index: usize,
    face_index_a: usize,
    face_index_b: usize,
    cell_subcell_index: usize,
    cell_subcell_a_index: usize,
    cell_subcell_b_index: usize,
    cell_subcell_ab_index: usize,
    face_subsubcell_index: usize,
    face_subsubcell_a_index: usize,
    face_subsubcell_b_index: usize,
    face_subsubcell_ab_index: usize,
    cell_faces: &Faces,
    cell_subcells: &Indices,
    cells_nodes: &[usize],
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
) {
    if let Some(cell_a_index) = cell_faces[face_index_a] {
        if let Some(cell_b_index) = cell_faces[face_index_b] {
            if let Some((cell_a_subcells, cell_a_faces)) =
                tree.cell_contains_leaves(&tree[cell_a_index])
            {
                if let Some(cell_ab_index) = cell_a_faces[face_index_b] {
                    if let Some((cell_b_subcells, cell_b_faces)) =
                        tree.cell_contains_leaves(&tree[cell_b_index])
                    {
                        if let Some((cell_ab_subcells, cell_ab_faces)) =
                            tree.cell_contains_leaves(&tree[cell_ab_index])
                        {
                            if let Some(face_cell_index) = cell_faces[face_index] {
                                if let Some(face_subsubcells) = tree.cell_subcells_contain_leaves(
                                    &tree[face_cell_index],
                                    face_index,
                                ) {
                                    if let Some(face_cell_a_index) = cell_a_faces[face_index] {
                                        if let Some(face_a_subsubcells) = tree
                                            .cell_subcells_contain_leaves(
                                                &tree[face_cell_a_index],
                                                face_index,
                                            )
                                        {
                                            if let Some(face_cell_b_index) =
                                                cell_b_faces[face_index]
                                            {
                                                if let Some(face_b_subsubcells) = tree
                                                    .cell_subcells_contain_leaves(
                                                        &tree[face_cell_b_index],
                                                        face_index,
                                                    )
                                                {
                                                    if let Some(face_cell_ab_index) =
                                                        cell_ab_faces[face_index]
                                                    {
                                                        if let Some(face_ab_subsubcells) = tree
                                                            .cell_subcells_contain_leaves(
                                                                &tree[face_cell_ab_index],
                                                                face_index,
                                                            )
                                                        {
                                                            element_node_connectivity.push([
                                                                cells_nodes[cell_subcells
                                                                    [cell_subcell_index]],
                                                                cells_nodes[cell_a_subcells
                                                                    [cell_subcell_a_index]],
                                                                cells_nodes[cell_ab_subcells
                                                                    [cell_subcell_ab_index]],
                                                                cells_nodes[cell_b_subcells
                                                                    [cell_subcell_b_index]],
                                                                cells_nodes[face_subsubcells
                                                                    [face_subsubcell_index]],
                                                                cells_nodes[face_a_subsubcells
                                                                    [face_subsubcell_a_index]],
                                                                cells_nodes[face_ab_subsubcells
                                                                    [face_subsubcell_ab_index]],
                                                                cells_nodes[face_b_subsubcells
                                                                    [face_subsubcell_b_index]],
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
