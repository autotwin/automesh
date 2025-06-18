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
}

// if cell faces 1 level of refinement (face template 1)
// and 1 of 4 cell neighbors is unrefined and has same face bove 1 level of refinement
