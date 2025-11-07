use super::super::{Coordinates, HexConnectivity, NUM_OCTANTS, NodeMap, Octree, face_direction};

pub fn apply(
    cells_nodes: &[usize],
    nodes_map: &NodeMap,
    tree: &Octree,
    nodal_coordinates: &Coordinates,
) -> HexConnectivity {
    let mut element_node_connectivity = vec![];
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, _)| {
            template(
                4,
                6,
                3,
                5,
                7,
                5,
                2,
                0,
                cells_nodes,
                cell_subcells,
                nodes_map,
                tree,
                &mut element_node_connectivity,
                nodal_coordinates,
            );
            template(
                5,
                7,
                5,
                1,
                3,
                1,
                6,
                4,
                cells_nodes,
                cell_subcells,
                nodes_map,
                tree,
                &mut element_node_connectivity,
                nodal_coordinates,
            );
            template(
                5,
                4,
                0,
                5,
                6,
                7,
                0,
                1,
                cells_nodes,
                cell_subcells,
                nodes_map,
                tree,
                &mut element_node_connectivity,
                nodal_coordinates,
            );
            template(
                7,
                6,
                5,
                2,
                2,
                3,
                4,
                5,
                cells_nodes,
                cell_subcells,
                nodes_map,
                tree,
                &mut element_node_connectivity,
                nodal_coordinates,
            );
            template(
                6,
                2,
                3,
                2,
                3,
                7,
                0,
                4,
                cells_nodes,
                cell_subcells,
                nodes_map,
                tree,
                &mut element_node_connectivity,
                nodal_coordinates,
            );
            template(
                7,
                3,
                2,
                1,
                1,
                5,
                2,
                6,
                cells_nodes,
                cell_subcells,
                nodes_map,
                tree,
                &mut element_node_connectivity,
                nodal_coordinates,
            );
        });
    element_node_connectivity
}

#[allow(clippy::too_many_arguments)]
fn template(
    subcell_a: usize,
    subcell_b: usize,
    face_m: usize,
    face_n: usize,
    subcell_face_m_p: usize,
    subcell_face_m_q: usize,
    subcell_face_n_p: usize,
    subcell_face_n_q: usize,
    cells_nodes: &[usize],
    cell_subcells: &[usize; NUM_OCTANTS],
    nodes_map: &NodeMap,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &Coordinates,
) {
    let subcell_a_faces = tree[cell_subcells[subcell_a]].get_faces();
    if let Some(subcell_a_face_m) = subcell_a_faces[face_m]
        && let Some(subcell_a_face_n) = subcell_a_faces[face_n]
        && let Some(diagonal_a) = tree[subcell_a_face_m].get_faces()[face_n]
        && tree[diagonal_a].is_leaf()
    {
        let subcell_b_faces = tree[cell_subcells[subcell_b]].get_faces();
        if let Some(subcell_b_face_m) = subcell_b_faces[face_m]
            && let Some(subcell_b_face_n) = subcell_b_faces[face_n]
            && let Some(diagonal_b) = tree[subcell_b_face_m].get_faces()[face_n]
            && tree[diagonal_b].is_leaf()
            && let Some((subcell_a_face_m_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_a_face_m])
            && let Some((subcell_a_face_n_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_a_face_n])
            && let Some((subcell_b_face_m_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_b_face_m])
            && let Some((subcell_b_face_n_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_b_face_n])
        {
            let lngth = *tree[subcell_a_face_m_subcells[subcell_face_m_p]].get_lngth() as f64;
            let coordinates_1 = &nodal_coordinates
                [cells_nodes[subcell_a_face_m_subcells[subcell_face_m_p]]]
                - face_direction(face_m) * lngth;
            let node_1 = *nodes_map
                .get(&[
                    (2.0 * coordinates_1[0]) as usize,
                    (2.0 * coordinates_1[1]) as usize,
                    (2.0 * coordinates_1[2]) as usize,
                ])
                .expect("nonexistent entry");
            let coordinates_2 = &nodal_coordinates
                [cells_nodes[subcell_b_face_m_subcells[subcell_face_m_q]]]
                - face_direction(face_m) * lngth;
            let node_2 = *nodes_map
                .get(&[
                    (2.0 * coordinates_2[0]) as usize,
                    (2.0 * coordinates_2[1]) as usize,
                    (2.0 * coordinates_2[2]) as usize,
                ])
                .expect("nonexistent entry");
            let coordinates_3 = &nodal_coordinates
                [cells_nodes[subcell_a_face_m_subcells[subcell_face_m_p]]]
                + face_direction(face_n) * lngth;
            let node_3 = *nodes_map
                .get(&[
                    (2.0 * coordinates_3[0]) as usize,
                    (2.0 * coordinates_3[1]) as usize,
                    (2.0 * coordinates_3[2]) as usize,
                ])
                .expect("nonexistent entry");
            let coordinates_4 = &nodal_coordinates
                [cells_nodes[subcell_b_face_m_subcells[subcell_face_m_q]]]
                + face_direction(face_n) * lngth;
            let node_4 = *nodes_map
                .get(&[
                    (2.0 * coordinates_4[0]) as usize,
                    (2.0 * coordinates_4[1]) as usize,
                    (2.0 * coordinates_4[2]) as usize,
                ])
                .expect("nonexistent entry");
            element_node_connectivity.push([
                cells_nodes[subcell_a_face_m_subcells[subcell_face_m_p]],
                node_1,
                node_2,
                cells_nodes[subcell_b_face_m_subcells[subcell_face_m_q]],
                node_3,
                cells_nodes[subcell_a_face_n_subcells[subcell_face_n_p]],
                cells_nodes[subcell_b_face_n_subcells[subcell_face_n_q]],
                node_4,
            ]);
            element_node_connectivity.push([
                cells_nodes[subcell_b_face_m_subcells[subcell_face_m_q]],
                node_2,
                cells_nodes[cell_subcells[subcell_b]],
                cells_nodes[subcell_b_face_m_subcells[subcell_face_m_p]],
                node_4,
                cells_nodes[subcell_b_face_n_subcells[subcell_face_n_q]],
                cells_nodes[subcell_b_face_n_subcells[subcell_face_n_p]],
                cells_nodes[diagonal_b],
            ]);
            element_node_connectivity.push([
                cells_nodes[subcell_a_face_m_subcells[subcell_face_m_q]],
                cells_nodes[cell_subcells[subcell_a]],
                node_1,
                cells_nodes[subcell_a_face_m_subcells[subcell_face_m_p]],
                cells_nodes[diagonal_a],
                cells_nodes[subcell_a_face_n_subcells[subcell_face_n_q]],
                cells_nodes[subcell_a_face_n_subcells[subcell_face_n_p]],
                node_3,
            ]);
        }
    }
}
