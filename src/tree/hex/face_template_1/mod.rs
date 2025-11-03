use super::super::{
    Coordinate, Coordinates, HexConnectivity, NUM_OCTANTS, NUM_SUBCELLS_FACE, NodeMap, Octree,
    SubSubCellsFace, subcells_on_own_face,
};
use crate::{NSD, Vector};
use conspire::math::{TensorArray, TensorVec, tensor_rank_1};

const SCALE_1: f64 = 0.5;

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
                    if let Some(face_cell_index) = face_cell
                        && let Some(face_subsubcells) =
                            tree.cell_subcells_contain_leaves(&tree[*face_cell_index], face_index)
                    {
                        template(
                            cell_subcells,
                            cells_nodes,
                            nodes_map,
                            face_index,
                            face_subsubcells,
                            tree,
                            element_node_connectivity,
                            nodal_coordinates,
                            node_index,
                        )
                    }
                })
        });
}

#[allow(clippy::too_many_arguments)]
fn template(
    cell_subcells: &[usize; NUM_OCTANTS],
    cells_nodes: &[usize],
    nodes_map: &mut NodeMap,
    face_index: usize,
    face_subsubcells: SubSubCellsFace,
    tree: &Octree,
    element_node_connectivity: &mut HexConnectivity,
    nodal_coordinates: &mut Coordinates,
    node_index: &mut usize,
) {
    let cell_subcells_face_nodes = subcells_on_own_face(face_index)
        .iter()
        .map(|&index| cells_nodes[cell_subcells[index]])
        .collect::<Vec<usize>>()
        .try_into()
        .unwrap();
    let adjacent_exterior_nodes = [
        cells_nodes[face_subsubcells[1]],
        cells_nodes[face_subsubcells[4]],
        cells_nodes[face_subsubcells[7]],
        cells_nodes[face_subsubcells[13]],
        cells_nodes[face_subsubcells[14]],
        cells_nodes[face_subsubcells[11]],
        cells_nodes[face_subsubcells[8]],
        cells_nodes[face_subsubcells[2]],
    ];
    let adjacent_interior_nodes = [
        cells_nodes[face_subsubcells[3]],
        cells_nodes[face_subsubcells[6]],
        cells_nodes[face_subsubcells[12]],
        cells_nodes[face_subsubcells[9]],
    ];
    let (scale_1, scale_2) = translations(face_index, &face_subsubcells, tree);
    let interior_nodes = [
        *node_index,
        *node_index + 1,
        *node_index + 2,
        *node_index + 3,
    ];
    *node_index += 4;
    adjacent_interior_nodes
        .iter()
        .for_each(|&foo_i| nodal_coordinates.push(&nodal_coordinates[foo_i] + scale_1.clone()));
    let mut coordinates = Coordinate::zero();
    let mut indices = [0; NSD];
    let mut exterior_nodes = [0; 8];
    adjacent_exterior_nodes
        .iter()
        .zip(exterior_nodes.iter_mut())
        .for_each(|(&adjacent_exterior_node_i, exterior_node_i)| {
            coordinates = &nodal_coordinates[adjacent_exterior_node_i] + scale_2.clone();
            indices = [
                (2.0 * coordinates[0]) as usize,
                (2.0 * coordinates[1]) as usize,
                (2.0 * coordinates[2]) as usize,
            ];
            if let Some(node_id) = nodes_map.get(&indices) {
                *exterior_node_i = *node_id;
            } else {
                *exterior_node_i = *node_index;
                nodal_coordinates.push(coordinates.clone());
                nodes_map.insert(indices, *node_index);
                *node_index += 1;
            }
        });
    connectivity(
        cells_nodes,
        cell_subcells_face_nodes,
        face_index,
        face_subsubcells,
        interior_nodes,
        exterior_nodes,
        element_node_connectivity,
    )
}

fn connectivity(
    cells_nodes: &[usize],
    cell_subcells_face_nodes: [usize; NUM_SUBCELLS_FACE],
    face_index: usize,
    face_subsubcells: SubSubCellsFace,
    interior_nodes: [usize; 4],
    exterior_nodes: [usize; 8],
    element_node_connectivity: &mut HexConnectivity,
) {
    match face_index {
        2..=4 => {
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[3]],
                cells_nodes[face_subsubcells[6]],
                cells_nodes[face_subsubcells[12]],
                cells_nodes[face_subsubcells[9]],
                interior_nodes[0],
                interior_nodes[1],
                interior_nodes[2],
                interior_nodes[3],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[1]],
                cells_nodes[face_subsubcells[4]],
                cells_nodes[face_subsubcells[6]],
                cells_nodes[face_subsubcells[3]],
                exterior_nodes[0],
                exterior_nodes[1],
                interior_nodes[1],
                interior_nodes[0],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[6]],
                cells_nodes[face_subsubcells[7]],
                cells_nodes[face_subsubcells[13]],
                cells_nodes[face_subsubcells[12]],
                interior_nodes[1],
                exterior_nodes[2],
                exterior_nodes[3],
                interior_nodes[2],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[9]],
                cells_nodes[face_subsubcells[12]],
                cells_nodes[face_subsubcells[14]],
                cells_nodes[face_subsubcells[11]],
                interior_nodes[3],
                interior_nodes[2],
                exterior_nodes[4],
                exterior_nodes[5],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[2]],
                cells_nodes[face_subsubcells[3]],
                cells_nodes[face_subsubcells[9]],
                cells_nodes[face_subsubcells[8]],
                exterior_nodes[7],
                interior_nodes[0],
                interior_nodes[3],
                exterior_nodes[6],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[0]],
                cells_nodes[face_subsubcells[1]],
                cells_nodes[face_subsubcells[3]],
                cells_nodes[face_subsubcells[2]],
                cell_subcells_face_nodes[0],
                exterior_nodes[0],
                interior_nodes[0],
                exterior_nodes[7],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[4]],
                cells_nodes[face_subsubcells[5]],
                cells_nodes[face_subsubcells[7]],
                cells_nodes[face_subsubcells[6]],
                exterior_nodes[1],
                cell_subcells_face_nodes[1],
                exterior_nodes[2],
                interior_nodes[1],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[12]],
                cells_nodes[face_subsubcells[13]],
                cells_nodes[face_subsubcells[15]],
                cells_nodes[face_subsubcells[14]],
                interior_nodes[2],
                exterior_nodes[3],
                cell_subcells_face_nodes[3],
                exterior_nodes[4],
            ]);
            element_node_connectivity.push([
                cells_nodes[face_subsubcells[8]],
                cells_nodes[face_subsubcells[9]],
                cells_nodes[face_subsubcells[11]],
                cells_nodes[face_subsubcells[10]],
                exterior_nodes[6],
                interior_nodes[3],
                exterior_nodes[5],
                cell_subcells_face_nodes[2],
            ]);
            element_node_connectivity.push([
                interior_nodes[0],
                interior_nodes[1],
                interior_nodes[2],
                interior_nodes[3],
                exterior_nodes[0],
                exterior_nodes[1],
                exterior_nodes[4],
                exterior_nodes[5],
            ]);
            element_node_connectivity.push([
                exterior_nodes[0],
                exterior_nodes[1],
                exterior_nodes[4],
                exterior_nodes[5],
                cell_subcells_face_nodes[0],
                cell_subcells_face_nodes[1],
                cell_subcells_face_nodes[3],
                cell_subcells_face_nodes[2],
            ]);
            element_node_connectivity.push([
                exterior_nodes[2],
                exterior_nodes[3],
                interior_nodes[2],
                interior_nodes[1],
                cell_subcells_face_nodes[1],
                cell_subcells_face_nodes[3],
                exterior_nodes[4],
                exterior_nodes[1],
            ]);
            element_node_connectivity.push([
                exterior_nodes[6],
                exterior_nodes[7],
                interior_nodes[0],
                interior_nodes[3],
                cell_subcells_face_nodes[2],
                cell_subcells_face_nodes[0],
                exterior_nodes[0],
                exterior_nodes[5],
            ]);
        }
        0 | 1 | 5 => {
            element_node_connectivity.push([
                interior_nodes[0],
                interior_nodes[1],
                interior_nodes[2],
                interior_nodes[3],
                cells_nodes[face_subsubcells[3]],
                cells_nodes[face_subsubcells[6]],
                cells_nodes[face_subsubcells[12]],
                cells_nodes[face_subsubcells[9]],
            ]);
            element_node_connectivity.push([
                exterior_nodes[0],
                exterior_nodes[1],
                interior_nodes[1],
                interior_nodes[0],
                cells_nodes[face_subsubcells[1]],
                cells_nodes[face_subsubcells[4]],
                cells_nodes[face_subsubcells[6]],
                cells_nodes[face_subsubcells[3]],
            ]);
            element_node_connectivity.push([
                interior_nodes[1],
                exterior_nodes[2],
                exterior_nodes[3],
                interior_nodes[2],
                cells_nodes[face_subsubcells[6]],
                cells_nodes[face_subsubcells[7]],
                cells_nodes[face_subsubcells[13]],
                cells_nodes[face_subsubcells[12]],
            ]);
            element_node_connectivity.push([
                interior_nodes[3],
                interior_nodes[2],
                exterior_nodes[4],
                exterior_nodes[5],
                cells_nodes[face_subsubcells[9]],
                cells_nodes[face_subsubcells[12]],
                cells_nodes[face_subsubcells[14]],
                cells_nodes[face_subsubcells[11]],
            ]);
            element_node_connectivity.push([
                exterior_nodes[7],
                interior_nodes[0],
                interior_nodes[3],
                exterior_nodes[6],
                cells_nodes[face_subsubcells[2]],
                cells_nodes[face_subsubcells[3]],
                cells_nodes[face_subsubcells[9]],
                cells_nodes[face_subsubcells[8]],
            ]);
            element_node_connectivity.push([
                cell_subcells_face_nodes[0],
                exterior_nodes[0],
                interior_nodes[0],
                exterior_nodes[7],
                cells_nodes[face_subsubcells[0]],
                cells_nodes[face_subsubcells[1]],
                cells_nodes[face_subsubcells[3]],
                cells_nodes[face_subsubcells[2]],
            ]);
            element_node_connectivity.push([
                exterior_nodes[1],
                cell_subcells_face_nodes[1],
                exterior_nodes[2],
                interior_nodes[1],
                cells_nodes[face_subsubcells[4]],
                cells_nodes[face_subsubcells[5]],
                cells_nodes[face_subsubcells[7]],
                cells_nodes[face_subsubcells[6]],
            ]);
            element_node_connectivity.push([
                interior_nodes[2],
                exterior_nodes[3],
                cell_subcells_face_nodes[3],
                exterior_nodes[4],
                cells_nodes[face_subsubcells[12]],
                cells_nodes[face_subsubcells[13]],
                cells_nodes[face_subsubcells[15]],
                cells_nodes[face_subsubcells[14]],
            ]);
            element_node_connectivity.push([
                exterior_nodes[6],
                interior_nodes[3],
                exterior_nodes[5],
                cell_subcells_face_nodes[2],
                cells_nodes[face_subsubcells[8]],
                cells_nodes[face_subsubcells[9]],
                cells_nodes[face_subsubcells[11]],
                cells_nodes[face_subsubcells[10]],
            ]);
            element_node_connectivity.push([
                exterior_nodes[0],
                exterior_nodes[1],
                exterior_nodes[4],
                exterior_nodes[5],
                interior_nodes[0],
                interior_nodes[1],
                interior_nodes[2],
                interior_nodes[3],
            ]);
            element_node_connectivity.push([
                cell_subcells_face_nodes[0],
                cell_subcells_face_nodes[1],
                cell_subcells_face_nodes[3],
                cell_subcells_face_nodes[2],
                exterior_nodes[0],
                exterior_nodes[1],
                exterior_nodes[4],
                exterior_nodes[5],
            ]);
            element_node_connectivity.push([
                cell_subcells_face_nodes[1],
                cell_subcells_face_nodes[3],
                exterior_nodes[4],
                exterior_nodes[1],
                exterior_nodes[2],
                exterior_nodes[3],
                interior_nodes[2],
                interior_nodes[1],
            ]);
            element_node_connectivity.push([
                cell_subcells_face_nodes[2],
                cell_subcells_face_nodes[0],
                exterior_nodes[0],
                exterior_nodes[5],
                exterior_nodes[6],
                exterior_nodes[7],
                interior_nodes[0],
                interior_nodes[3],
            ]);
        }
        _ => panic!(),
    }
}

fn translations(
    face_index: usize,
    face_subsubcells: &SubSubCellsFace,
    tree: &Octree,
) -> (Vector, Vector) {
    match face_index {
        0 => (
            tensor_rank_1([
                0.0,
                SCALE_1 * *tree[face_subsubcells[0]].get_lngth() as f64,
                0.0,
            ]),
            tensor_rank_1([0.0, *tree[face_subsubcells[0]].get_lngth() as f64, 0.0]),
        ),
        1 => (
            tensor_rank_1([
                -SCALE_1 * *tree[face_subsubcells[0]].get_lngth() as f64,
                0.0,
                0.0,
            ]),
            tensor_rank_1([-(*tree[face_subsubcells[0]].get_lngth() as f64), 0.0, 0.0]),
        ),
        2 => (
            tensor_rank_1([
                0.0,
                -SCALE_1 * *tree[face_subsubcells[0]].get_lngth() as f64,
                0.0,
            ]),
            tensor_rank_1([0.0, -(*tree[face_subsubcells[0]].get_lngth() as f64), 0.0]),
        ),
        3 => (
            tensor_rank_1([
                SCALE_1 * *tree[face_subsubcells[0]].get_lngth() as f64,
                0.0,
                0.0,
            ]),
            tensor_rank_1([*tree[face_subsubcells[0]].get_lngth() as f64, 0.0, 0.0]),
        ),
        4 => (
            tensor_rank_1([
                0.0,
                0.0,
                SCALE_1 * *tree[face_subsubcells[0]].get_lngth() as f64,
            ]),
            tensor_rank_1([0.0, 0.0, *tree[face_subsubcells[0]].get_lngth() as f64]),
        ),
        5 => (
            tensor_rank_1([
                0.0,
                0.0,
                -SCALE_1 * *tree[face_subsubcells[0]].get_lngth() as f64,
            ]),
            tensor_rank_1([0.0, 0.0, -(*tree[face_subsubcells[0]].get_lngth() as f64)]),
        ),
        _ => panic!(),
    }
}
