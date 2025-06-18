use super::super::{Coordinates, HexConnectivity, NODE_NUMBERING_OFFSET, NodeMap, Octree};
use conspire::math::tensor_rank_1;

pub fn apply(
    cells_nodes: &[usize],
    nodes_map: &mut NodeMap,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, cell_faces)| {
            cell_faces
                .iter()
                .enumerate()
                .for_each(|(face_index, face_cell)| {
                    if let Some(face_cell_index) = face_cell {
                        if let Some((_, face_subsubcells)) = tree.cell_subcells_contain_leaves(
                            &tree[*face_cell_index],
                            0,
                            face_index,
                        ) {
                            //
                            // Check 4 relevant face neighbors for similar case:
                            // (1) neighbor contains leaves also
                            // (2) same face of that neighbor has subcells containing leaves
                            //
                            match face_index {
                                0 => {}
                                1 => {}
                                2 => {
                                    let adjacent_face = 5;
                                    //
                                    // check neighbors on faces (1, 3, 4, 5)
                                    // could have to place template on any combo of them
                                    // so this will eventually have to call template() 4 times for the 4 cases
                                    //
                                    let direction = tensor_rank_1([0.0, -1.0, 0.0]);
                                    //
                                    // Can match above direction to face_index
                                    //
                                    if let Some(adjacent_cell) = cell_faces[adjacent_face] {
                                        if let Some((adjacent_cell_subcells, adjacent_cell_faces)) =
                                            tree.cell_contains_leaves(&tree[adjacent_cell])
                                        {
                                            if let Some(adjacent_cell_face_cell) =
                                                adjacent_cell_faces[face_index]
                                            {
                                                if let Some((_, adjacent_face_subsubcells)) = tree
                                                    .cell_subcells_contain_leaves(
                                                        &tree[adjacent_cell_face_cell],
                                                        0,
                                                        face_index,
                                                    )
                                                {
                                                    let lngth = *tree[face_subsubcells[14]]
                                                        .get_lngth()
                                                        as f64;
                                                    let coordinates = &nodal_coordinates[cells_nodes
                                                        [face_subsubcells[14]]
                                                        - NODE_NUMBERING_OFFSET]
                                                        + &direction * lngth;
                                                    let indices = (
                                                        (2.0 * coordinates[0]) as usize,
                                                        (2.0 * coordinates[1]) as usize,
                                                        (2.0 * coordinates[2]) as usize,
                                                    );
                                                    let foo_1 = nodes_map
                                                        .get(&(indices))
                                                        .expect("nonexistent entry");
                                                    let coordinates = &nodal_coordinates[cells_nodes
                                                        [face_subsubcells[11]]
                                                        - NODE_NUMBERING_OFFSET]
                                                        + &direction * lngth;
                                                    let indices = (
                                                        (2.0 * coordinates[0]) as usize,
                                                        (2.0 * coordinates[1]) as usize,
                                                        (2.0 * coordinates[2]) as usize,
                                                    );
                                                    let foo_2 = nodes_map
                                                        .get(&(indices))
                                                        .expect("nonexistent entry");
                                                    let coordinates = &nodal_coordinates[cells_nodes
                                                        [adjacent_face_subsubcells[4]]
                                                        - NODE_NUMBERING_OFFSET]
                                                        + &direction * lngth;
                                                    let indices = (
                                                        (2.0 * coordinates[0]) as usize,
                                                        (2.0 * coordinates[1]) as usize,
                                                        (2.0 * coordinates[2]) as usize,
                                                    );
                                                    let foo_3 = nodes_map
                                                        .get(&(indices))
                                                        .expect("nonexistent entry");
                                                    let coordinates = &nodal_coordinates[cells_nodes
                                                        [adjacent_face_subsubcells[1]]
                                                        - NODE_NUMBERING_OFFSET]
                                                        + &direction * lngth;
                                                    let indices = (
                                                        (2.0 * coordinates[0]) as usize,
                                                        (2.0 * coordinates[1]) as usize,
                                                        (2.0 * coordinates[2]) as usize,
                                                    );
                                                    let foo_4 = nodes_map
                                                        .get(&(indices))
                                                        .expect("nonexistent entry");
                                                    element_node_connectivity.push([
                                                        cells_nodes[cell_subcells[6]],
                                                        cells_nodes[cell_subcells[7]],
                                                        *foo_1,
                                                        *foo_2,
                                                        cells_nodes[adjacent_cell_subcells[2]],
                                                        cells_nodes[adjacent_cell_subcells[3]],
                                                        *foo_3,
                                                        *foo_4,
                                                    ]);
                                                    element_node_connectivity.push([
                                                        cells_nodes[face_subsubcells[14]],
                                                        cells_nodes[face_subsubcells[11]],
                                                        *foo_2,
                                                        *foo_1,
                                                        cells_nodes[adjacent_face_subsubcells[4]],
                                                        cells_nodes[adjacent_face_subsubcells[1]],
                                                        *foo_4,
                                                        *foo_3,
                                                    ]);
                                                    element_node_connectivity.push([
                                                        cells_nodes[face_subsubcells[11]],
                                                        *foo_2,
                                                        *foo_4,
                                                        cells_nodes[adjacent_face_subsubcells[1]],
                                                        cells_nodes[face_subsubcells[10]],
                                                        cells_nodes[cell_subcells[6]],
                                                        cells_nodes[adjacent_cell_subcells[2]],
                                                        cells_nodes[adjacent_face_subsubcells[0]],
                                                    ]);
                                                    element_node_connectivity.push([
                                                        cells_nodes[face_subsubcells[14]],
                                                        cells_nodes[adjacent_face_subsubcells[4]],
                                                        *foo_3,
                                                        *foo_1,
                                                        cells_nodes[face_subsubcells[15]],
                                                        cells_nodes[adjacent_face_subsubcells[5]],
                                                        cells_nodes[adjacent_cell_subcells[3]],
                                                        cells_nodes[cell_subcells[7]],
                                                    ]);
                                                }
                                            }
                                        }
                                    }
                                }
                                3 => {}
                                4 => {}
                                5 => {}
                                _ => panic!(),
                            }
                        }
                    }
                })
        })
}
