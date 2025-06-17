use super::super::{Coordinates, HexConnectivity, NODE_NUMBERING_OFFSET, NUM_OCTANTS, NodeMap, Octree};
use conspire::math::{TensorVec, tensor_rank_1};

pub fn apply(
    cells_nodes: &Vec<usize>,
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, _)| {
            template(3, 7, cells_nodes, &cell_subcells, nodes_map, node_index, tree, element_node_connectivity, nodal_coordinates)
        })
}

fn template(
    subcell_a: usize,
    subcell_b: usize,
    cells_nodes: &Vec<usize>,
    cell_subcells: &[usize; NUM_OCTANTS],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    let (face_m, face_n, subcell_c, subcell_d, subcell_e, subcell_f, direction) = match (subcell_a, subcell_b) {
        (3, 7) => (1, 2, 2, 1, 6, 5, tensor_rank_1::<3, 1>([-2.0, 0.0, 0.0])),
        _=> panic!(),
    };
    let subcell_a_faces = tree[cell_subcells[subcell_a]].get_faces();
    if let Some(subcell_a_face_a) = subcell_a_faces[face_m] {
        if let Some(subcell_a_face_b) = subcell_a_faces[face_n] {
            if let Some((subcell_a_face_a_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_a_face_a])
            {
                if let Some((subcell_a_face_b_subcells, _)) =
                    tree.cell_contains_leaves(&tree[subcell_a_face_b])
                {
                    if let Some(diagonal_a) = tree[subcell_a_face_a_subcells[subcell_c]].get_faces()[face_n] {
                        if let Some(subdiagonal_a) =
                            tree[subcell_a_face_a_subcells[subcell_e]].get_faces()[face_n]
                        {
                            let subcell_b_faces = tree[cell_subcells[subcell_a]].get_faces();
                            if let Some(subcell_b_face_a) = subcell_b_faces[face_m] {
                                if let Some(subcell_b_face_b) = subcell_b_faces[face_n] {
                                    if let Some((subcell_b_face_a_subcells, _)) =
                                        tree.cell_contains_leaves(&tree[subcell_b_face_a])
                                    {
                                        if let Some((subcell_b_face_b_subcells, _)) =
                                            tree.cell_contains_leaves(&tree[subcell_b_face_b])
                                        {
                                            if let Some(diagonal_b) =
                                                tree[subcell_b_face_a_subcells[subcell_e]].get_faces()[face_n]
                                            {
                                                if let Some(subdiagonal_b) = tree
                                                    [subcell_b_face_a_subcells[subcell_c]]
                                                    .get_faces()[face_n]
                                                {
                                                    let lngth = *tree[subcell_a_face_a_subcells[subcell_e]]
                                                        .get_lngth()
                                                        as f64;
                                                    nodal_coordinates.push(
                                                        &nodal_coordinates[cells_nodes
                                                            [subcell_a_face_a_subcells[subcell_e]]]
                                                            + &direction * lngth,
                                                    );
                                                    nodal_coordinates.push(
                                                        &nodal_coordinates[cells_nodes
                                                            [subcell_b_face_a_subcells[subcell_c]]]
                                                            + direction * lngth,
                                                    );
                                                    nodes_map.insert(
                                                        (
                                                            (2.0 * nodal_coordinates[*node_index
                                                                - NODE_NUMBERING_OFFSET][0])
                                                                as usize,
                                                            (2.0 * nodal_coordinates[*node_index
                                                                - NODE_NUMBERING_OFFSET][1])
                                                                as usize,
                                                            (2.0 * nodal_coordinates[*node_index
                                                                - NODE_NUMBERING_OFFSET][2])
                                                                as usize,
                                                        ),
                                                        *node_index,
                                                    );
                                                    nodes_map.insert(
                                                        (
                                                            (2.0 * nodal_coordinates[*node_index
                                                                + 1
                                                                - NODE_NUMBERING_OFFSET][0])
                                                                as usize,
                                                            (2.0 * nodal_coordinates[*node_index
                                                                + 1
                                                                - NODE_NUMBERING_OFFSET][1])
                                                                as usize,
                                                            (2.0 * nodal_coordinates[*node_index
                                                                + 1
                                                                - NODE_NUMBERING_OFFSET][2])
                                                                as usize,
                                                        ),
                                                        *node_index + 1,
                                                    );
                                                    element_node_connectivity.push([
                                                        cells_nodes[cell_subcells[subcell_a]],
                                                        cells_nodes[subcell_a_face_a_subcells[subcell_c]],
                                                        cells_nodes[diagonal_a],
                                                        cells_nodes[subcell_a_face_b_subcells[subcell_d]],
                                                        *node_index,
                                                        cells_nodes[subcell_a_face_a_subcells[subcell_e]],
                                                        cells_nodes[subdiagonal_a],
                                                        cells_nodes[subcell_a_face_b_subcells[subcell_f]],
                                                    ]);
                                                    element_node_connectivity.push([
                                                        *node_index,
                                                        cells_nodes[subcell_a_face_a_subcells[subcell_e]],
                                                        cells_nodes[subdiagonal_a],
                                                        cells_nodes[subcell_a_face_b_subcells[subcell_f]],
                                                        *node_index + 1,
                                                        cells_nodes[subcell_b_face_a_subcells[subcell_c]],
                                                        cells_nodes[subdiagonal_b],
                                                        cells_nodes[subcell_b_face_b_subcells[subcell_d]],
                                                    ]);
                                                    element_node_connectivity.push([
                                                        *node_index + 1,
                                                        cells_nodes[subcell_b_face_a_subcells[subcell_c]],
                                                        cells_nodes[subdiagonal_b],
                                                        cells_nodes[subcell_b_face_b_subcells[subcell_d]],
                                                        cells_nodes[cell_subcells[subcell_b]],
                                                        cells_nodes[subcell_b_face_a_subcells[subcell_e]],
                                                        cells_nodes[diagonal_b],
                                                        cells_nodes[subcell_b_face_b_subcells[subcell_f]],
                                                    ]);
                                                    *node_index += 2;
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
