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
            let subcell_3_faces = tree[cell_subcells[3]].get_faces();
            if let Some(subcell_3_face_1) = subcell_3_faces[1] {
                if let Some(subcell_3_face_2) = subcell_3_faces[2] {
                    if let Some((subcell_3_face_1_subcells, _)) =
                        tree.cell_contains_leaves(&tree[subcell_3_face_1])
                    {
                        if let Some((subcell_3_face_2_subcells, _)) =
                            tree.cell_contains_leaves(&tree[subcell_3_face_2])
                        {
                            if let Some(diagonal_1) =
                                tree[subcell_3_face_1_subcells[2]].get_faces()[2]
                            {
                                if let Some(subdiagonal_1) =
                                    tree[subcell_3_face_1_subcells[6]].get_faces()[2]
                                {
                                    let subcell_7_faces = tree[cell_subcells[7]].get_faces();
                                    if let Some(subcell_7_face_1) = subcell_7_faces[1] {
                                        if let Some(subcell_7_face_2) = subcell_7_faces[2] {
                                            if let Some((subcell_7_face_1_subcells, _)) =
                                                tree.cell_contains_leaves(&tree[subcell_7_face_1])
                                            {
                                                if let Some((subcell_7_face_2_subcells, _)) = tree
                                                    .cell_contains_leaves(&tree[subcell_7_face_2])
                                                {
                                                    if let Some(diagonal_2) = tree
                                                        [subcell_7_face_1_subcells[6]]
                                                        .get_faces()[2]
                                                    {
                                                        if let Some(subdiagonal_2) = tree
                                                            [subcell_7_face_1_subcells[2]]
                                                            .get_faces()[2]
                                                        {
                                                            let lngth = *tree
                                                                [subcell_3_face_1_subcells[6]]
                                                                .get_lngth()
                                                                as f64;
                                                            let bar = tensor_rank_1([
                                                                -2.0 * lngth,
                                                                0.0,
                                                                0.0,
                                                            ]);
                                                            nodal_coordinates.push(
                                                                &nodal_coordinates[cells_nodes
                                                                    [subcell_3_face_1_subcells[6]]]
                                                                    + bar.clone(),
                                                            );
                                                            nodal_coordinates.push(
                                                                &nodal_coordinates[cells_nodes
                                                                    [subcell_7_face_1_subcells[2]]]
                                                                    + bar.clone(),
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
                                                                cells_nodes[cell_subcells[3]],
                                                                cells_nodes[subcell_3_face_1_subcells[2]],
                                                                cells_nodes[diagonal_1],
                                                                cells_nodes[subcell_3_face_2_subcells[1]],
                                                                *node_index,
                                                                cells_nodes
                                                                    [subcell_3_face_1_subcells[6]],
                                                                cells_nodes[subdiagonal_1],
                                                                cells_nodes
                                                                    [subcell_3_face_2_subcells[5]],
                                                            ]);
                                                            element_node_connectivity.push([
                                                                *node_index,
                                                                cells_nodes
                                                                    [subcell_3_face_1_subcells[6]],
                                                                cells_nodes[subdiagonal_1],
                                                                cells_nodes
                                                                    [subcell_3_face_2_subcells[5]],
                                                                *node_index + 1,
                                                                cells_nodes
                                                                    [subcell_7_face_1_subcells[2]],
                                                                cells_nodes[subdiagonal_2],
                                                                cells_nodes
                                                                    [subcell_7_face_2_subcells[1]],
                                                            ]);
                                                            element_node_connectivity.push([
                                                                *node_index + 1,
                                                                cells_nodes
                                                                    [subcell_7_face_1_subcells[2]],
                                                                cells_nodes[subdiagonal_2],
                                                                cells_nodes
                                                                    [subcell_7_face_2_subcells[1]],
                                                                cells_nodes[cell_subcells[7]],
                                                                cells_nodes[subcell_7_face_1_subcells[6]],
                                                                cells_nodes[diagonal_2],
                                                                cells_nodes[subcell_7_face_2_subcells[5]],
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
        })
}

fn template(
    cells_nodes: &Vec<usize>,
    cell_subcells: &[usize; NUM_OCTANTS],
    nodes_map: &mut NodeMap,
    node_index: &mut usize,
    subcell_a: usize,
    subcell_b: usize,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
) {
    //
    // need to match subcells (2 and 6 everywhere, plus stuff in connectivity)
    //
    // need to match sign and direction of `bar`
    //
    let (face_a_index, face_b_index) = match (subcell_a, subcell_b) {
        (3, 7) => (1, 2),
        _=> panic!(),
    };
    let subcell_a_faces = tree[cell_subcells[subcell_a]].get_faces();
    if let Some(subcell_a_face_a) = subcell_a_faces[face_a_index] {
        if let Some(subcell_a_face_b) = subcell_a_faces[face_b_index] {
            if let Some((subcell_a_face_a_subcells, _)) =
                tree.cell_contains_leaves(&tree[subcell_a_face_a])
            {
                if let Some((subcell_a_face_b_subcells, _)) =
                    tree.cell_contains_leaves(&tree[subcell_a_face_b])
                {
                    if let Some(diagonal_a) = tree[subcell_a_face_a_subcells[2]].get_faces()[face_b_index] {
                        if let Some(subdiagonal_a) =
                            tree[subcell_a_face_a_subcells[6]].get_faces()[face_b_index]
                        {
                            let subcell_b_faces = tree[cell_subcells[subcell_a]].get_faces();
                            if let Some(subcell_b_face_a) = subcell_b_faces[face_a_index] {
                                if let Some(subcell_b_face_b) = subcell_b_faces[2] {
                                    if let Some((subcell_b_face_a_subcells, _)) =
                                        tree.cell_contains_leaves(&tree[subcell_b_face_a])
                                    {
                                        if let Some((subcell_b_face_b_subcells, _)) =
                                            tree.cell_contains_leaves(&tree[subcell_b_face_b])
                                        {
                                            if let Some(diagonal_b) =
                                                tree[subcell_b_face_a_subcells[6]].get_faces()[face_b_index]
                                            {
                                                if let Some(subdiagonal_b) = tree
                                                    [subcell_b_face_a_subcells[2]]
                                                    .get_faces()[face_b_index]
                                                {
                                                    let lngth = *tree[subcell_a_face_a_subcells[6]]
                                                        .get_lngth()
                                                        as f64;
                                                    let bar =
                                                        tensor_rank_1([-2.0 * lngth, 0.0, 0.0]);
                                                    nodal_coordinates.push(
                                                        &nodal_coordinates[cells_nodes
                                                            [subcell_a_face_a_subcells[6]]]
                                                            + bar.clone(),
                                                    );
                                                    nodal_coordinates.push(
                                                        &nodal_coordinates[cells_nodes
                                                            [subcell_b_face_a_subcells[2]]]
                                                            + bar.clone(),
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
                                                        cells_nodes[cell_subcells[3]],
                                                        cells_nodes[subcell_a_face_a_subcells[2]],
                                                        cells_nodes[diagonal_a],
                                                        cells_nodes[subcell_a_face_b_subcells[1]],
                                                        *node_index,
                                                        cells_nodes[subcell_a_face_a_subcells[6]],
                                                        cells_nodes[subdiagonal_a],
                                                        cells_nodes[subcell_a_face_b_subcells[5]],
                                                    ]);
                                                    element_node_connectivity.push([
                                                        *node_index,
                                                        cells_nodes[subcell_a_face_a_subcells[6]],
                                                        cells_nodes[subdiagonal_a],
                                                        cells_nodes[subcell_a_face_b_subcells[5]],
                                                        *node_index + 1,
                                                        cells_nodes[subcell_b_face_a_subcells[2]],
                                                        cells_nodes[subdiagonal_b],
                                                        cells_nodes[subcell_b_face_b_subcells[1]],
                                                    ]);
                                                    element_node_connectivity.push([
                                                        *node_index + 1,
                                                        cells_nodes[subcell_b_face_a_subcells[2]],
                                                        cells_nodes[subdiagonal_b],
                                                        cells_nodes[subcell_b_face_b_subcells[1]],
                                                        cells_nodes[cell_subcells[7]],
                                                        cells_nodes[subcell_b_face_a_subcells[6]],
                                                        cells_nodes[diagonal_b],
                                                        cells_nodes[subcell_b_face_b_subcells[5]],
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
