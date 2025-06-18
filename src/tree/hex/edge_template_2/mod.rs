use super::super::{
    Coordinates, HexConnectivity, NODE_NUMBERING_OFFSET, NodeMap, Octree, mirror_face,
};
use conspire::math::{TensorVec, tensor_rank_1};

pub fn apply(
    cells_nodes: &[usize],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    let direction_m = tensor_rank_1::<3, 1>([0.0, -1.0, 0.0]);
    let direction_n = tensor_rank_1::<3, 1>([1.0, 0.0, 0.0]);
    let face_m = 0;
    let face_n = 1;
    tree.iter().enumerate().for_each(|(cell_index, cell)| {
        if let Some(cell_face_m) = cell.get_faces()[face_m] {
            if let Some(cell_face_n) = cell.get_faces()[face_n] {
                if let Some(cell_diag_mn) = tree[cell_face_m].get_faces()[face_n] {
                    if let Some((subcells_face_m, _)) =
                        tree.cell_contains_leaves(&tree[cell_face_m])
                    {
                        if let Some((subcells_face_n, _)) =
                            tree.cell_contains_leaves(&tree[cell_face_n])
                        {
                            if let Some((subcells_diag_mn, _)) =
                                tree.cell_contains_leaves(&tree[cell_diag_mn])
                            {
                                if let Some((_, subcells_m)) = tree.cell_subcells_contain_leaves(
                                    cell,
                                    cell_index,
                                    mirror_face(face_m),
                                ) {
                                    if let Some((_, subcells_n)) = tree
                                        .cell_subcells_contain_leaves(
                                            cell,
                                            cell_index,
                                            mirror_face(face_n),
                                        )
                                    {
                                        let lngth = *tree[subcells_m[7]].get_lngth() as f64;
                                        nodal_coordinates.push(
                                            &nodal_coordinates[cells_nodes[subcells_m[7]]
                                                - NODE_NUMBERING_OFFSET]
                                                + &direction_m * lngth,
                                        );
                                        nodal_coordinates.push(
                                            &nodal_coordinates[cells_nodes[subcells_m[7]]
                                                - NODE_NUMBERING_OFFSET]
                                                + &direction_m * lngth
                                                + &direction_n * lngth,
                                        );
                                        nodal_coordinates.push(
                                            &nodal_coordinates[cells_nodes[subcells_m[7]]
                                                - NODE_NUMBERING_OFFSET]
                                                + &direction_n * lngth,
                                        );
                                        nodal_coordinates.push(
                                            &nodal_coordinates[cells_nodes[subcells_m[13]]
                                                - NODE_NUMBERING_OFFSET]
                                                + &direction_m * lngth,
                                        );
                                        nodal_coordinates.push(
                                            &nodal_coordinates[cells_nodes[subcells_m[13]]
                                                - NODE_NUMBERING_OFFSET]
                                                + &direction_m * lngth
                                                + &direction_n * lngth,
                                        );
                                        nodal_coordinates.push(
                                            &nodal_coordinates[cells_nodes[subcells_m[13]]
                                                - NODE_NUMBERING_OFFSET]
                                                + &direction_n * lngth,
                                        );
                                        (0..6).for_each(|k| {
                                            assert!(
                                                nodes_map
                                                    .insert(
                                                        (
                                                            (2.0 * nodal_coordinates[*node_index
                                                                + k
                                                                - NODE_NUMBERING_OFFSET][0])
                                                                as usize,
                                                            (2.0 * nodal_coordinates[*node_index
                                                                + k
                                                                - NODE_NUMBERING_OFFSET][1])
                                                                as usize,
                                                            (2.0 * nodal_coordinates[*node_index
                                                                + k
                                                                - NODE_NUMBERING_OFFSET][2])
                                                                as usize,
                                                        ),
                                                        *node_index + k,
                                                    )
                                                    .is_none(),
                                                "duplicate entry"
                                            )
                                        });
                                        element_node_connectivity.push([
                                            cells_nodes[subcells_m[7]],
                                            *node_index,
                                            *node_index + 1,
                                            *node_index + 2,
                                            cells_nodes[subcells_m[13]],
                                            *node_index + 3,
                                            *node_index + 4,
                                            *node_index + 5,
                                        ]);
                                        element_node_connectivity.push([
                                            *node_index,
                                            cells_nodes[subcells_face_m[3]],
                                            cells_nodes[subcells_diag_mn[2]],
                                            *node_index + 1,
                                            *node_index + 3,
                                            cells_nodes[subcells_face_m[7]],
                                            cells_nodes[subcells_diag_mn[6]],
                                            *node_index + 4,
                                        ]);
                                        element_node_connectivity.push([
                                            *node_index + 1,
                                            cells_nodes[subcells_diag_mn[2]],
                                            cells_nodes[subcells_face_n[0]],
                                            *node_index + 2,
                                            *node_index + 4,
                                            cells_nodes[subcells_diag_mn[6]],
                                            cells_nodes[subcells_face_n[4]],
                                            *node_index + 5,
                                        ]);
                                        element_node_connectivity.push([
                                            cells_nodes[subcells_m[7]],
                                            *node_index + 2,
                                            *node_index + 1,
                                            *node_index,
                                            cells_nodes[subcells_m[5]],
                                            cells_nodes[subcells_face_n[0]],
                                            cells_nodes[subcells_diag_mn[2]],
                                            cells_nodes[subcells_face_m[3]],
                                        ]);
                                        element_node_connectivity.push([
                                            cells_nodes[subcells_m[13]],
                                            *node_index + 3,
                                            *node_index + 4,
                                            *node_index + 5,
                                            cells_nodes[subcells_m[15]],
                                            cells_nodes[subcells_face_m[7]],
                                            cells_nodes[subcells_diag_mn[6]],
                                            cells_nodes[subcells_face_n[4]],
                                        ]);
                                        *node_index += 6;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

// #[allow(clippy::too_many_arguments)]
// fn template(
//     subcell_a: usize,
//     subcell_b: usize,
//     cells_nodes: &[usize],
//     cell_subcells: &[usize; NUM_OCTANTS],
//     nodes_map: &mut NodeMap,
//     node_index: &mut usize,
//     tree: &Octree,
//     element_node_connectivity: &mut HexConnectivity,
//     nodal_coordinates: &mut Coordinates,
// ) {
//     //
//     // if cell subcells contain leaves
//     //     if consecutive faces are less refined
//     //         corner between them is this template
//     //
// }
