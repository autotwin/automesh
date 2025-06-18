use super::super::{
    Cell, Coordinates, HexConnectivity, NODE_NUMBERING_OFFSET, NodeMap, Octree, mirror_face,
};
use conspire::math::{TensorRank1, TensorVec, tensor_rank_1};

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
                            // ALL NODES SHOULD BE PRESENT FROM FACE_TEMPLATE_1
                            //
                            // Check 4 relevant face neighbors for similar case:
                            // (1) neighbor contains leaves also
                            // (2) same face of that neighbor has subcells containing leaves
                            //
                            match face_index {
                                2 => {
                                    let adjacent_face = 4;
                                    //
                                    // check neighbors on faces (1, 3, 4, 5)
                                    // could have to place template on any combo of them
                                    // so this will eventually have to call template() 4 times for the 4 cases
                                    //
                                    if let Some(adjacent_cell) = cell_faces[adjacent_face] {
                                        if let Some((adjacent_cell_subcells, adjacent_cell_faces)) =
                                            tree.cell_contains_leaves(&tree[adjacent_cell])
                                        {
                                            if let Some(adjacent_cell_face_cell) =
                                                adjacent_cell_faces[adjacent_face]
                                            {
                                                if let Some((_, adjacent_face_subsubcells)) = tree
                                                    .cell_subcells_contain_leaves(
                                                        &tree[adjacent_cell_face_cell],
                                                        0,
                                                        face_index,
                                                    )
                                                {
                                                    // found one!
                                                    // element_node_connectivity.push([
                                                    //     cells_nodes[]
                                                    // ]);
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    // make this into a panic after get all 6 in above
                                }
                            }
                        }
                    }
                })
        })
}
