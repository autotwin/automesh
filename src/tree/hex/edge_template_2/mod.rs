use super::super::{Coordinates, HexConnectivity, NodeMap, Octree, face_direction};

pub fn apply(
    cells_nodes: &[usize],
    nodes_map: &NodeMap,
    tree: &Octree,
    nodal_coordinates: &Coordinates,
) -> HexConnectivity {
    let mut element_node_connectivity = vec![];
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, cell_faces)| {
            cell_faces
                .iter()
                .enumerate()
                .for_each(|(face_index, face_cell)| {
                    if let Some(face_cell_index) = face_cell
                        && let Some(face_subsubcells) =
                            tree.cell_subcells_contain_leaves(&tree[*face_cell_index], face_index)
                    {
                        template_inner(face_index).into_iter().for_each(
                            |[
                                adjacent_face,
                                cell_subcell_a,
                                cell_subcell_b,
                                adjacent_cell_subcell_a,
                                adjacent_cell_subcell_b,
                                face_subsubcell_a,
                                face_subsubcell_b,
                                face_subsubcell_c,
                                face_subsubcell_d,
                                adjacent_face_subsubcell_a,
                                adjacent_face_subsubcell_b,
                                adjacent_face_subsubcell_c,
                                adjacent_face_subsubcell_d,
                            ]| {
                                if let Some(adjacent_cell) = cell_faces[adjacent_face]
                                    && let Some((adjacent_cell_subcells, adjacent_cell_faces)) =
                                        tree.cell_contains_leaves(&tree[adjacent_cell])
                                    && let Some(adjacent_cell_face_cell) =
                                        adjacent_cell_faces[face_index]
                                    && let Some(adjacent_face_subsubcells) = tree
                                        .cell_subcells_contain_leaves(
                                            &tree[adjacent_cell_face_cell],
                                            face_index,
                                        )
                                {
                                    let lngth = *tree[face_subsubcells[face_subsubcell_a]]
                                        .get_lngth()
                                        as f64;
                                    let coordinates_1 = &nodal_coordinates
                                        [cells_nodes[face_subsubcells[face_subsubcell_a]]]
                                        - face_direction(face_index) * lngth;
                                    let node_1 = *nodes_map
                                        .get(&[
                                            (2.0 * coordinates_1[0]) as usize,
                                            (2.0 * coordinates_1[1]) as usize,
                                            (2.0 * coordinates_1[2]) as usize,
                                        ])
                                        .expect("nonexistent entry");
                                    let coordinates_2 = &nodal_coordinates
                                        [cells_nodes[face_subsubcells[face_subsubcell_b]]]
                                        - face_direction(face_index) * lngth;
                                    let node_2 = *nodes_map
                                        .get(&[
                                            (2.0 * coordinates_2[0]) as usize,
                                            (2.0 * coordinates_2[1]) as usize,
                                            (2.0 * coordinates_2[2]) as usize,
                                        ])
                                        .expect("nonexistent entry");
                                    let coordinates_3 = &nodal_coordinates[cells_nodes
                                        [adjacent_face_subsubcells[adjacent_face_subsubcell_a]]]
                                        - face_direction(face_index) * lngth;
                                    let node_3 = *nodes_map
                                        .get(&[
                                            (2.0 * coordinates_3[0]) as usize,
                                            (2.0 * coordinates_3[1]) as usize,
                                            (2.0 * coordinates_3[2]) as usize,
                                        ])
                                        .expect("nonexistent entry");
                                    let coordinates_4 = &nodal_coordinates[cells_nodes
                                        [adjacent_face_subsubcells[adjacent_face_subsubcell_b]]]
                                        - face_direction(face_index) * lngth;
                                    let node_4 = *nodes_map
                                        .get(&[
                                            (2.0 * coordinates_4[0]) as usize,
                                            (2.0 * coordinates_4[1]) as usize,
                                            (2.0 * coordinates_4[2]) as usize,
                                        ])
                                        .expect("nonexistent entry");
                                    element_node_connectivity.push([
                                        cells_nodes[cell_subcells[cell_subcell_a]],
                                        cells_nodes[cell_subcells[cell_subcell_b]],
                                        node_1,
                                        node_2,
                                        cells_nodes
                                            [adjacent_cell_subcells[adjacent_cell_subcell_a]],
                                        cells_nodes
                                            [adjacent_cell_subcells[adjacent_cell_subcell_b]],
                                        node_3,
                                        node_4,
                                    ]);
                                    element_node_connectivity.push([
                                        cells_nodes[face_subsubcells[face_subsubcell_a]],
                                        cells_nodes[face_subsubcells[face_subsubcell_b]],
                                        node_2,
                                        node_1,
                                        cells_nodes
                                            [adjacent_face_subsubcells[adjacent_face_subsubcell_a]],
                                        cells_nodes
                                            [adjacent_face_subsubcells[adjacent_face_subsubcell_b]],
                                        node_4,
                                        node_3,
                                    ]);
                                    element_node_connectivity.push([
                                        cells_nodes[face_subsubcells[face_subsubcell_b]],
                                        node_2,
                                        node_4,
                                        cells_nodes
                                            [adjacent_face_subsubcells[adjacent_face_subsubcell_b]],
                                        cells_nodes[face_subsubcells[face_subsubcell_d]],
                                        cells_nodes[cell_subcells[cell_subcell_a]],
                                        cells_nodes
                                            [adjacent_cell_subcells[adjacent_cell_subcell_a]],
                                        cells_nodes
                                            [adjacent_face_subsubcells[adjacent_face_subsubcell_d]],
                                    ]);
                                    element_node_connectivity.push([
                                        cells_nodes[face_subsubcells[face_subsubcell_a]],
                                        cells_nodes
                                            [adjacent_face_subsubcells[adjacent_face_subsubcell_a]],
                                        node_3,
                                        node_1,
                                        cells_nodes[face_subsubcells[face_subsubcell_c]],
                                        cells_nodes
                                            [adjacent_face_subsubcells[adjacent_face_subsubcell_c]],
                                        cells_nodes
                                            [adjacent_cell_subcells[adjacent_cell_subcell_b]],
                                        cells_nodes[cell_subcells[cell_subcell_b]],
                                    ]);
                                }
                            },
                        )
                    }
                })
        });
    element_node_connectivity
}

fn template_inner(face_index: usize) -> [[usize; 13]; 2] {
    match face_index {
        0 => [
            [1, 1, 5, 0, 4, 13, 7, 15, 5, 8, 2, 10, 0],
            [4, 0, 1, 4, 5, 4, 1, 5, 0, 14, 11, 15, 10],
        ],
        1 => [
            [0, 5, 1, 7, 3, 2, 8, 0, 10, 7, 13, 5, 15],
            [4, 1, 3, 5, 7, 4, 1, 5, 0, 14, 11, 15, 10],
        ],
        2 => [
            [1, 7, 3, 6, 2, 7, 13, 5, 15, 2, 8, 0, 10],
            [5, 6, 7, 2, 3, 14, 11, 15, 10, 4, 1, 5, 0],
        ],
        3 => [
            [2, 6, 2, 4, 0, 7, 13, 5, 15, 2, 8, 0, 10],
            [5, 4, 6, 0, 2, 14, 11, 15, 10, 4, 1, 5, 0],
        ],
        4 => [
            [1, 3, 1, 2, 0, 7, 13, 5, 15, 2, 8, 0, 10],
            [2, 2, 3, 0, 1, 14, 11, 15, 10, 4, 1, 5, 0],
        ],
        5 => [
            [0, 4, 5, 6, 7, 4, 1, 5, 0, 14, 11, 15, 10],
            [3, 6, 4, 7, 5, 2, 8, 0, 10, 7, 13, 5, 15],
        ],
        _ => panic!(),
    }
}
