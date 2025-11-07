use super::super::{Cell, Coordinates, HexConnectivity, NodeMap, Octree, mirror_face};
use conspire::math::{TensorRank1, TensorVec, tensor_rank_1};

pub fn apply(
    cells_nodes: &[usize],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    tree.iter().for_each(|cell| {
        template(
            0,
            1,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            0,
            4,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            1,
            2,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            1,
            4,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            2,
            3,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            2,
            5,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            3,
            0,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            4,
            2,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            4,
            3,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            5,
            0,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            5,
            1,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
        template(
            3,
            5,
            cell,
            cells_nodes,
            nodes_map,
            node_index,
            tree,
            element_node_connectivity,
            nodal_coordinates,
        );
    })
}

#[allow(clippy::too_many_arguments)]
fn template(
    face_m: usize,
    face_n: usize,
    cell: &Cell,
    cells_nodes: &[usize],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    let (
        subcell_m_a,
        subcell_m_b,
        subcell_m_c,
        subcell_m_d,
        subcell_face_m_a,
        subcell_face_m_b,
        subcell_face_n_a,
        subcell_face_n_b,
        subcell_diag_mn_a,
        subcell_diag_mn_b,
    ) = match (face_m, face_n) {
        (0, 1) => (7, 13, 5, 15, 3, 7, 0, 4, 2, 6),
        (0, 4) => (1, 4, 0, 5, 2, 3, 4, 5, 6, 7),
        (1, 2) => (7, 13, 5, 15, 2, 6, 1, 5, 0, 4),
        (1, 4) => (1, 4, 0, 5, 0, 2, 5, 7, 4, 6),
        (2, 3) => (2, 8, 0, 10, 0, 4, 3, 7, 1, 5),
        (2, 5) => (11, 14, 10, 15, 4, 5, 2, 3, 0, 1),
        (3, 0) => (2, 8, 0, 10, 1, 5, 2, 6, 3, 7),
        (3, 5) => (11, 14, 10, 15, 5, 7, 0, 2, 1, 3),
        (4, 2) => (11, 14, 10, 15, 6, 7, 0, 1, 4, 5),
        (4, 3) => (2, 8, 0, 10, 4, 6, 1, 3, 5, 7),
        (5, 0) => (1, 4, 0, 5, 0, 1, 6, 7, 2, 3),
        (5, 1) => (7, 13, 5, 15, 1, 3, 4, 6, 0, 2),
        _ => panic!(),
    };
    let directions: [TensorRank1<3, 1>; 2] = [face_m, face_n]
        .iter()
        .map(|face| match face {
            0 => tensor_rank_1([0.0, -1.0, 0.0]),
            1 => tensor_rank_1([1.0, 0.0, 0.0]),
            2 => tensor_rank_1([0.0, 1.0, 0.0]),
            3 => tensor_rank_1([-1.0, 0.0, 0.0]),
            4 => tensor_rank_1([0.0, 0.0, -1.0]),
            5 => tensor_rank_1([0.0, 0.0, 1.0]),
            _ => panic!(),
        })
        .collect::<Vec<TensorRank1<3, 1>>>()
        .try_into()
        .unwrap();
    if let Some(cell_face_m) = cell.get_faces()[face_m]
        && let Some(cell_face_n) = cell.get_faces()[face_n]
        && let Some(cell_diag_mn) = tree[cell_face_m].get_faces()[face_n]
        && let Some((subcells_face_m, _)) = tree.cell_contains_leaves(&tree[cell_face_m])
        && let Some((subcells_face_n, _)) = tree.cell_contains_leaves(&tree[cell_face_n])
        && let Some((subcells_diag_mn, _)) = tree.cell_contains_leaves(&tree[cell_diag_mn])
        && let Some(subcells_m) = tree.cell_subcells_contain_leaves(cell, mirror_face(face_m))
        && tree
            .cell_subcells_contain_leaves(cell, mirror_face(face_n))
            .is_some()
    {
        let lngth = *tree[subcells_m[subcell_m_a]].get_lngth() as f64;
        nodal_coordinates.push(
            &nodal_coordinates[cells_nodes[subcells_m[subcell_m_a]]] + &directions[0] * lngth,
        );
        nodal_coordinates.push(
            &nodal_coordinates[cells_nodes[subcells_m[subcell_m_a]]]
                + &directions[0] * lngth
                + &directions[1] * lngth,
        );
        nodal_coordinates.push(
            &nodal_coordinates[cells_nodes[subcells_m[subcell_m_a]]] + &directions[1] * lngth,
        );
        nodal_coordinates.push(
            &nodal_coordinates[cells_nodes[subcells_m[subcell_m_b]]] + &directions[0] * lngth,
        );
        nodal_coordinates.push(
            &nodal_coordinates[cells_nodes[subcells_m[subcell_m_b]]]
                + &directions[0] * lngth
                + &directions[1] * lngth,
        );
        nodal_coordinates.push(
            &nodal_coordinates[cells_nodes[subcells_m[subcell_m_b]]] + &directions[1] * lngth,
        );
        (0..6).for_each(|k| {
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
        element_node_connectivity.push([
            cells_nodes[subcells_m[subcell_m_a]],
            *node_index,
            *node_index + 1,
            *node_index + 2,
            cells_nodes[subcells_m[subcell_m_b]],
            *node_index + 3,
            *node_index + 4,
            *node_index + 5,
        ]);
        element_node_connectivity.push([
            *node_index,
            cells_nodes[subcells_face_m[subcell_face_m_a]],
            cells_nodes[subcells_diag_mn[subcell_diag_mn_a]],
            *node_index + 1,
            *node_index + 3,
            cells_nodes[subcells_face_m[subcell_face_m_b]],
            cells_nodes[subcells_diag_mn[subcell_diag_mn_b]],
            *node_index + 4,
        ]);
        element_node_connectivity.push([
            *node_index + 1,
            cells_nodes[subcells_diag_mn[subcell_diag_mn_a]],
            cells_nodes[subcells_face_n[subcell_face_n_a]],
            *node_index + 2,
            *node_index + 4,
            cells_nodes[subcells_diag_mn[subcell_diag_mn_b]],
            cells_nodes[subcells_face_n[subcell_face_n_b]],
            *node_index + 5,
        ]);
        element_node_connectivity.push([
            cells_nodes[subcells_m[subcell_m_a]],
            *node_index + 2,
            *node_index + 1,
            *node_index,
            cells_nodes[subcells_m[subcell_m_c]],
            cells_nodes[subcells_face_n[subcell_face_n_a]],
            cells_nodes[subcells_diag_mn[subcell_diag_mn_a]],
            cells_nodes[subcells_face_m[subcell_face_m_a]],
        ]);
        element_node_connectivity.push([
            cells_nodes[subcells_m[subcell_m_b]],
            *node_index + 3,
            *node_index + 4,
            *node_index + 5,
            cells_nodes[subcells_m[subcell_m_d]],
            cells_nodes[subcells_face_m[subcell_face_m_b]],
            cells_nodes[subcells_diag_mn[subcell_diag_mn_b]],
            cells_nodes[subcells_face_n[subcell_face_n_b]],
        ]);
        *node_index += 6;
    }
}
