use super::super::{Coordinates, HexConnectivity, NUM_OCTANTS, NodeMap, Octree};
use conspire::math::{TensorVec, tensor_rank_1};

pub fn apply(
    cells_nodes: &[usize],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, _)| {
            template(
                0,
                1,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                0,
                2,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                0,
                4,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                1,
                3,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                1,
                5,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                2,
                3,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                2,
                6,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                3,
                7,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                4,
                5,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                4,
                6,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                5,
                7,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            );
            template(
                6,
                7,
                cells_nodes,
                cell_subcells,
                nodes_map,
                node_index,
                tree,
                element_node_connectivity,
                nodal_coordinates,
            )
        })
}

#[allow(clippy::too_many_arguments)]
fn template(
    subcell_a: usize,
    subcell_b: usize,
    cells_nodes: &[usize],
    cell_subcells: &[usize; NUM_OCTANTS],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    let (face_m, face_n, subcell_c, subcell_d, subcell_e, subcell_f, flip, direction) =
        match (subcell_a, subcell_b) {
            (0, 1) => (0, 4, 2, 4, 3, 5, false, tensor_rank_1([0.0, 1.0, 0.0])),
            (0, 2) => (3, 4, 1, 4, 3, 6, true, tensor_rank_1([1.0, 0.0, 0.0])),
            (0, 4) => (3, 0, 1, 2, 5, 6, false, tensor_rank_1([1.0, 0.0, 0.0])),
            (1, 3) => (1, 4, 0, 5, 2, 7, false, tensor_rank_1([-1.0, 0.0, 0.0])),
            (1, 5) => (0, 1, 3, 0, 7, 4, false, tensor_rank_1([0.0, 1.0, 0.0])),
            (2, 3) => (2, 4, 0, 6, 1, 7, true, tensor_rank_1([0.0, -1.0, 0.0])),
            (2, 6) => (2, 3, 0, 3, 4, 7, false, tensor_rank_1([0.0, -1.0, 0.0])),
            (3, 7) => (1, 2, 2, 1, 6, 5, false, tensor_rank_1([-1.0, 0.0, 0.0])),
            (4, 5) => (5, 0, 0, 6, 1, 7, false, tensor_rank_1([0.0, 0.0, -1.0])),
            (4, 6) => (5, 3, 0, 5, 2, 7, true, tensor_rank_1([0.0, 0.0, -1.0])),
            (5, 7) => (5, 1, 1, 4, 3, 6, false, tensor_rank_1([0.0, 0.0, -1.0])),
            (6, 7) => (5, 2, 2, 4, 3, 5, true, tensor_rank_1([0.0, 0.0, -1.0])),
            _ => panic!(),
        };
    let subcell_a_faces = tree[cell_subcells[subcell_a]].get_faces();
    if let Some(subcell_a_face_m) = subcell_a_faces[face_m]
        && let Some(subcell_a_face_n) = subcell_a_faces[face_n]
        && let Some((subcell_a_face_m_subcells, _)) =
            tree.cell_contains_leaves(&tree[subcell_a_face_m])
        && let Some((subcell_a_face_n_subcells, _)) =
            tree.cell_contains_leaves(&tree[subcell_a_face_n])
        && let Some(diagonal_a) = tree[subcell_a_face_m_subcells[subcell_c]].get_faces()[face_n]
        && tree[diagonal_a].is_leaf()
        && let Some(subdiagonal_a) = tree[subcell_a_face_m_subcells[subcell_e]].get_faces()[face_n]
        && tree[subdiagonal_a].is_leaf()
    {
        let subcell_b_faces = tree[cell_subcells[subcell_b]].get_faces();
        if let Some(subcell_b_face_m) = subcell_b_faces[face_m]
            && let Some(subcell_b_face_n) = subcell_b_faces[face_n]
            && let Some((subcell_b_face_m_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_b_face_m])
            && let Some((subcell_b_face_n_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_b_face_n])
            && let Some(diagonal_b) = tree[subcell_b_face_m_subcells[subcell_e]].get_faces()[face_n]
            && tree[diagonal_b].is_leaf()
            && let Some(subdiagonal_b) =
                tree[subcell_b_face_m_subcells[subcell_c]].get_faces()[face_n]
            && tree[subdiagonal_b].is_leaf()
        {
            let lngth = *tree[subcell_a_face_m_subcells[subcell_e]].get_lngth() as f64;
            nodal_coordinates.push(
                &nodal_coordinates[cells_nodes[subcell_a_face_m_subcells[subcell_e]]]
                    + &direction * lngth,
            );
            nodal_coordinates.push(
                &nodal_coordinates[cells_nodes[subcell_b_face_m_subcells[subcell_c]]]
                    + direction * lngth,
            );
            (0..2).for_each(|k| {
                assert!(
                    nodes_map
                        .insert(
                            [
                                (2.0 * nodal_coordinates[*node_index + k][0]) as usize,
                                (2.0 * nodal_coordinates[*node_index + k][1]) as usize,
                                (2.0 * nodal_coordinates[*node_index + k][2]) as usize,
                            ],
                            *node_index + k,
                        )
                        .is_none(),
                    "duplicate entry"
                )
            });
            if flip {
                element_node_connectivity.push([
                    *node_index,
                    cells_nodes[subcell_a_face_m_subcells[subcell_e]],
                    cells_nodes[subdiagonal_a],
                    cells_nodes[subcell_a_face_n_subcells[subcell_f]],
                    cells_nodes[cell_subcells[subcell_a]],
                    cells_nodes[subcell_a_face_m_subcells[subcell_c]],
                    cells_nodes[diagonal_a],
                    cells_nodes[subcell_a_face_n_subcells[subcell_d]],
                ]);
                element_node_connectivity.push([
                    *node_index + 1,
                    cells_nodes[subcell_b_face_m_subcells[subcell_c]],
                    cells_nodes[subdiagonal_b],
                    cells_nodes[subcell_b_face_n_subcells[subcell_d]],
                    *node_index,
                    cells_nodes[subcell_a_face_m_subcells[subcell_e]],
                    cells_nodes[subdiagonal_a],
                    cells_nodes[subcell_a_face_n_subcells[subcell_f]],
                ]);
                element_node_connectivity.push([
                    cells_nodes[cell_subcells[subcell_b]],
                    cells_nodes[subcell_b_face_m_subcells[subcell_e]],
                    cells_nodes[diagonal_b],
                    cells_nodes[subcell_b_face_n_subcells[subcell_f]],
                    *node_index + 1,
                    cells_nodes[subcell_b_face_m_subcells[subcell_c]],
                    cells_nodes[subdiagonal_b],
                    cells_nodes[subcell_b_face_n_subcells[subcell_d]],
                ]);
            } else {
                element_node_connectivity.push([
                    cells_nodes[cell_subcells[subcell_a]],
                    cells_nodes[subcell_a_face_m_subcells[subcell_c]],
                    cells_nodes[diagonal_a],
                    cells_nodes[subcell_a_face_n_subcells[subcell_d]],
                    *node_index,
                    cells_nodes[subcell_a_face_m_subcells[subcell_e]],
                    cells_nodes[subdiagonal_a],
                    cells_nodes[subcell_a_face_n_subcells[subcell_f]],
                ]);
                element_node_connectivity.push([
                    *node_index,
                    cells_nodes[subcell_a_face_m_subcells[subcell_e]],
                    cells_nodes[subdiagonal_a],
                    cells_nodes[subcell_a_face_n_subcells[subcell_f]],
                    *node_index + 1,
                    cells_nodes[subcell_b_face_m_subcells[subcell_c]],
                    cells_nodes[subdiagonal_b],
                    cells_nodes[subcell_b_face_n_subcells[subcell_d]],
                ]);
                element_node_connectivity.push([
                    *node_index + 1,
                    cells_nodes[subcell_b_face_m_subcells[subcell_c]],
                    cells_nodes[subdiagonal_b],
                    cells_nodes[subcell_b_face_n_subcells[subcell_d]],
                    cells_nodes[cell_subcells[subcell_b]],
                    cells_nodes[subcell_b_face_m_subcells[subcell_e]],
                    cells_nodes[diagonal_b],
                    cells_nodes[subcell_b_face_n_subcells[subcell_f]],
                ]);
            }
            *node_index += 2;
        }
    }
}
