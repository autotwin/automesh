#[cfg(feature = "profile")]
use std::time::Instant;

use crate::{
    Coordinate, Coordinates, Octree,
    fem::hex::{HEX, HexConnectivity, HexahedralFiniteElements},
    tree::{Faces, Indices, NodeMap},
};
use conspire::math::{Tensor, TensorArray, TensorVec};
use ndarray::parallel::prelude::*;
use std::collections::HashMap;

pub mod edge_template_1;
mod edge_template_2;
pub mod edge_template_3;
mod edge_template_4;
mod face_template_0;
pub mod face_template_1;
mod vertex_template_1; // (O, A, AB, B) => (o, a, ab, b)
mod vertex_template_10; // (O, a, AB, b) => (O, a, ab, b)
mod vertex_template_11; // (O, a, AB, b) => (O, a, AB, b)
mod vertex_template_12; // (O, a, AB, b) => (o, a, ab, b)
mod vertex_template_13; // (O, a, AB, b) => (o, A, ab, b)
mod vertex_template_14; // (O, A, AB, b) => (o, a, ab, B)
mod vertex_template_15; // (O, A, AB, b) => (o, a, AB, b)
mod vertex_template_16; // (O, A, AB, b) => (O, a, ab, b)
mod vertex_template_17; // (O, A, AB, b) => (o, a, AB, B)
mod vertex_template_18; // (O, A, AB, B) => (o, A, ab, B)
mod vertex_template_19; // (O, A, AB, b) => (O, a, AB, B)
mod vertex_template_2; // (O, a, ab, b) => (O, a, ab, b)
mod vertex_template_20; // (O, A, AB, b) => (o, A, ab, B)
mod vertex_template_21; // (O, a, AB, b) => (o, A, ab, B)
mod vertex_template_3; // (O, A, AB, B) => (o, A, AB, b)
mod vertex_template_4; // (O, A, AB, B) => (o, A, AB, B)
mod vertex_template_5; // (O, A, AB, B) => (o, A, ab, b)
mod vertex_template_6; // (O, A, AB, b) => (o, A, ab, b)
mod vertex_template_7; // (O, a, ab, b) => (o, a, ab, b)
mod vertex_template_8; // (O, A, AB, b) => (o, a, ab, b)
mod vertex_template_9; // (O, a, ab, b) => (o, a, AB, b)

pub struct HexesAndCoords(HexahedralFiniteElements, Coordinates);

impl From<HexesAndCoords> for (HexahedralFiniteElements, Coordinates) {
    fn from(hexes_and_samples: HexesAndCoords) -> Self {
        (hexes_and_samples.0, hexes_and_samples.1)
    }
}

impl From<&Octree> for HexesAndCoords {
    fn from(tree: &Octree) -> Self {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut cells_nodes = vec![0; tree.len()];
        let mut coordinates = Coordinates::zero(0);
        let mut node_index = 0;
        tree.iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_leaf())
            .for_each(|(leaf_index, leaf)| {
                cells_nodes[leaf_index] = node_index;
                coordinates.push(Coordinate::new([
                    0.5 * (2 * leaf.get_min_x() + leaf.get_lngth()) as f64,
                    0.5 * (2 * leaf.get_min_y() + leaf.get_lngth()) as f64,
                    0.5 * (2 * leaf.get_min_z() + leaf.get_lngth()) as f64,
                ]));
                node_index += 1;
            });
        let mut element_node_connectivity: HexConnectivity = vec![];
        let mut nodes_map = HashMap::new();
        edge_template_1::apply(
            &cells_nodes,
            &mut nodes_map,
            &mut node_index,
            tree,
            &mut element_node_connectivity,
            &mut coordinates,
        );
        edge_template_3::apply(
            &cells_nodes,
            &mut nodes_map,
            &mut node_index,
            tree,
            &mut element_node_connectivity,
            &mut coordinates,
        );
        face_template_1::apply(
            &cells_nodes,
            &mut nodes_map,
            &mut node_index,
            tree,
            &mut element_node_connectivity,
            &mut coordinates,
        );
        element_node_connectivity.append(
            &mut (1..=25)
                .into_par_iter()
                .flat_map(|index| {
                    apply_concurrently(index, &cells_nodes, &nodes_map, tree, &coordinates)
                })
                .collect(),
        );
        let nodal_coordinates = coordinates
            .iter()
            .map(|coordinate| {
                Coordinate::new([
                    coordinate[0] * tree.scale.x() + tree.translate.x(),
                    coordinate[1] * tree.scale.y() + tree.translate.y(),
                    coordinate[2] * tree.scale.z() + tree.translate.z(),
                ])
            })
            .collect();
        let finite_elements = HexahedralFiniteElements::from((
            vec![1; element_node_connectivity.len()],
            element_node_connectivity,
            nodal_coordinates,
        ));
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mDualization of octree\x1b[0m {:?} ",
            time.elapsed()
        );
        Self(finite_elements, coordinates)
    }
}

pub fn apply_concurrently(
    index: usize,
    cells_nodes: &[usize],
    nodes_map: &NodeMap,
    tree: &Octree,
    nodal_coordinates: &Coordinates,
) -> HexConnectivity {
    match index {
        1 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_1::DATA,
            vertex_template_1::template,
        ),
        2 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_2::DATA,
            vertex_template_2::template,
        ),
        3 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_3::DATA,
            vertex_template_3::template,
        ),
        4 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_4::DATA,
            vertex_template_4::template,
        ),
        5 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_5::DATA,
            vertex_template_5::template,
        ),
        6 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_6::DATA,
            vertex_template_6::template,
        ),
        7 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_7::DATA,
            vertex_template_7::template,
        ),
        8 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_8::DATA,
            vertex_template_8::template,
        ),
        9 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_9::DATA,
            vertex_template_9::template,
        ),
        10 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_10::DATA,
            vertex_template_10::template,
        ),
        11 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_11::DATA,
            vertex_template_11::template,
        ),
        12 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_12::DATA,
            vertex_template_12::template,
        ),
        13 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_13::DATA,
            vertex_template_13::template,
        ),
        14 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_14::DATA,
            vertex_template_14::template,
        ),
        15 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_15::DATA,
            vertex_template_15::template,
        ),
        16 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_16::DATA,
            vertex_template_16::template,
        ),
        17 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_17::DATA,
            vertex_template_17::template,
        ),
        18 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_18::DATA,
            vertex_template_18::template,
        ),
        19 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_19::DATA,
            vertex_template_19::template,
        ),
        20 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_20::DATA,
            vertex_template_20::template,
        ),
        21 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_21::DATA,
            vertex_template_21::template,
        ),
        22 => tree
            .iter()
            .filter_map(|cell| tree.cell_contains_leaves(cell))
            .map(|(cell_subcells, _)| {
                [
                    cells_nodes[cell_subcells[0]],
                    cells_nodes[cell_subcells[1]],
                    cells_nodes[cell_subcells[3]],
                    cells_nodes[cell_subcells[2]],
                    cells_nodes[cell_subcells[4]],
                    cells_nodes[cell_subcells[5]],
                    cells_nodes[cell_subcells[7]],
                    cells_nodes[cell_subcells[6]],
                ]
            })
            .collect(),
        23 => face_template_0::apply(cells_nodes, tree),
        24 => edge_template_2::apply(cells_nodes, nodes_map, tree, nodal_coordinates),
        25 => edge_template_4::apply(cells_nodes, nodes_map, tree, nodal_coordinates),
        _ => panic!(),
    }
}

fn apply_vertex_template<const T: usize>(
    cells_nodes: &[usize],
    tree: &Octree,
    data: [[usize; 11]; T],
    template: impl Fn(
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        &Faces,
        &Indices,
        &[usize],
        &Octree,
    ) -> Option<[usize; HEX]>,
) -> HexConnectivity {
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .flat_map(|(cell_subcells, cell_faces)| {
            data.iter()
                .filter_map(|data| {
                    template(
                        data[0],
                        data[1],
                        data[2],
                        data[3],
                        data[4],
                        data[5],
                        data[6],
                        data[7],
                        data[8],
                        data[9],
                        data[10],
                        cell_faces,
                        cell_subcells,
                        cells_nodes,
                        tree,
                    )
                })
                .collect::<HexConnectivity>()
        })
        .collect()
}
