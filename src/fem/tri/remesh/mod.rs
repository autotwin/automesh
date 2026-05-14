#[cfg(feature = "profile")]
use std::time::Instant;

use crate::{
    fem::{
        Connectivity, FiniteElementMethods, Size, Smoothing, VecConnectivity,
        tri::{Lengths, TRI, TriangularFiniteElements},
    },
    tree::Edges,
};
use conspire::{
    math::{Tensor, TensorVec},
    mechanics::Scalar,
};

const FOUR_THIRDS: Scalar = 4.0 / 3.0;
// const FOUR_FIFTHS: Scalar = 4.0 / 5.0;
const REGULAR_DEGREE: i8 = 6;

pub fn remesh(
    fem: &mut TriangularFiniteElements,
    iterations: usize,
    smoothing_method: &Smoothing,
    size: Size,
) {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut edges: Edges = fem
        .get_element_node_connectivity()
        .iter()
        .flat_map(|&[node_0, node_1, node_2]| {
            [[node_0, node_1], [node_1, node_2], [node_2, node_0]].into_iter()
        })
        .collect();
    edges.iter_mut().for_each(|edge| edge.sort());
    edges.sort();
    edges.dedup();
    let mut average_length = 0.0;
    let mut lengths = Lengths::zero(edges.len());
    edges
        .iter()
        .zip(lengths.iter_mut())
        .for_each(|(&[node_a, node_b], length)| {
            *length =
                (&fem.get_nodal_coordinates()[node_a] - &fem.get_nodal_coordinates()[node_b]).norm()
        });
    fem.boundary_nodes = vec![];
    fem.exterior_nodes = vec![];
    fem.interface_nodes = vec![];
    fem.interior_nodes = vec![];
    (0..iterations).for_each(|_| {
        average_length = if let Some(size) = size {
            size / FOUR_THIRDS
        } else {
            lengths.iter().sum::<Scalar>() / (lengths.len() as Scalar)
        };
        split_edges(fem, &mut edges, &mut lengths, average_length);
        // collapse_edges(fem, &mut edges, &mut lengths, average_length);
        flip_edges(fem, &mut edges, &mut lengths);
        fem.nodal_influencers();
        match smoothing_method {
            Smoothing::None => {}
            _ => {
                fem.smooth(smoothing_method).unwrap();
                edges
                    .iter()
                    .zip(lengths.iter_mut())
                    .for_each(|(&[node_a, node_b], length)| {
                        *length = (&fem.get_nodal_coordinates()[node_a]
                            - &fem.get_nodal_coordinates()[node_b])
                            .norm()
                    });
            }
        }
    });
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mIsotropic remesh tris\x1b[0m {:?}",
        time.elapsed()
    );
}

// fn split_edges(
pub fn split_edges(
    fem: &mut TriangularFiniteElements,
    edges: &mut Edges,
    lengths: &mut Lengths,
    average_length: Scalar,
) {
    let mut element_index_1 = 0;
    let mut element_index_2 = 0;
    let mut element_index_3 = 0;
    let mut element_index_4 = 0;
    let mut node_c = 0;
    let mut node_d = 0;
    let mut node_e = 0;
    let mut spot_a = 0;
    let mut spot_b = 0;
    let mut edge_eb = [0; 2];
    let mut edge_ec = [0; 2];
    let mut edge_ed = [0; 2];
    let mut new_edges = vec![];
    let mut new_lengths = Lengths::zero(0);
    let element_blocks = &mut fem.element_blocks;
    let element_node_connectivity = &mut fem.element_node_connectivity;
    let node_element_connectivity = &mut fem.node_element_connectivity;
    let node_node_connectivity = &mut fem.node_node_connectivity;
    let nodal_coordinates = &mut fem.nodal_coordinates;
    edges
        .iter_mut()
        .zip(lengths.iter_mut())
        .filter(|&(_, &mut length)| length > FOUR_THIRDS * average_length)
        .for_each(|([node_a, node_b], length)| {
            [element_index_1, element_index_2, node_c, node_d] = edge_info(
                *node_a,
                *node_b,
                element_node_connectivity,
                node_element_connectivity,
            );
            element_blocks.extend(vec![
                element_blocks[element_index_1],
                element_blocks[element_index_2],
            ]);
            nodal_coordinates
                .push((nodal_coordinates[*node_a].clone() + &nodal_coordinates[*node_b]) / 2.0);
            node_e = nodal_coordinates.len() - 1;
            spot_a = element_node_connectivity[element_index_1]
                .iter()
                .position(|node| node == node_a)
                .unwrap();
            spot_b = element_node_connectivity[element_index_1]
                .iter()
                .position(|node| node == node_b)
                .unwrap();
            if (spot_a == 0 && spot_b == 1)
                || (spot_a == 1 && spot_b == 2)
                || (spot_a == 2 && spot_b == 0)
            {
                element_node_connectivity[element_index_1] = [node_c, node_e, *node_b];
                element_node_connectivity[element_index_2] = [*node_a, node_e, node_c];
                element_node_connectivity.push([node_d, node_e, *node_a]);
                element_node_connectivity.push([*node_b, node_e, node_d]);
            } else {
                element_node_connectivity[element_index_1] = [node_e, node_c, *node_b];
                element_node_connectivity[element_index_2] = [node_e, *node_a, node_c];
                element_node_connectivity.push([node_e, node_d, *node_a]);
                element_node_connectivity.push([node_e, *node_b, node_d]);
            }
            element_index_3 = element_node_connectivity.len() - 2;
            element_index_4 = element_node_connectivity.len() - 1;
            node_element_connectivity[*node_a].retain(|element| element != &element_index_1);
            node_element_connectivity[*node_a].push(element_index_3);
            node_element_connectivity[*node_b].retain(|element| element != &element_index_2);
            node_element_connectivity[*node_b].push(element_index_4);
            node_element_connectivity[node_c].push(element_index_2);
            node_element_connectivity[node_d].push(element_index_1);
            node_element_connectivity[node_d]
                .retain(|element| element != &element_index_1 && element != &element_index_2);
            node_element_connectivity[node_d].extend(vec![element_index_3, element_index_4]);
            node_element_connectivity.push(vec![
                element_index_1,
                element_index_2,
                element_index_3,
                element_index_4,
            ]);
            node_node_connectivity[*node_a].retain(|node| node != node_b);
            node_node_connectivity[*node_a].push(node_e);
            node_node_connectivity[*node_a].sort();
            node_node_connectivity[*node_b].retain(|node| node != node_a);
            node_node_connectivity[*node_b].push(node_e);
            node_node_connectivity[*node_b].sort();
            node_node_connectivity[node_c].push(node_e);
            node_node_connectivity[node_c].sort();
            node_node_connectivity[node_d].push(node_e);
            node_node_connectivity[node_d].sort();
            node_node_connectivity.push(vec![*node_a, *node_b, node_c, node_d]);
            node_node_connectivity[node_e].sort();
            edge_eb = [node_e, *node_b];
            edge_eb.sort();
            new_edges.push(edge_eb);
            *node_b = node_e;
            edge_ec = [node_e, node_c];
            edge_ec.sort();
            new_edges.push(edge_ec);
            edge_ed = [node_e, node_d];
            edge_ed.sort();
            new_edges.push(edge_ed);
            *length *= 0.5;
            new_lengths.push(*length);
            new_lengths.push((&nodal_coordinates[node_e] - &nodal_coordinates[node_c]).norm());
            new_lengths.push((&nodal_coordinates[node_e] - &nodal_coordinates[node_d]).norm());
        });
    edges.append(&mut new_edges);
    lengths.append(&mut new_lengths);
}

// fn collapse_edges(
//     fem: &mut TriangularFiniteElements,
//     edges: &mut Edges,
//     lengths: &mut Lengths,
//     average_length: Scalar,
// ) {
//     //
//     // should try to handle degenerate tris etc. only when hit them in edge info
//     // and then clean up everything else once at the end
//     //
//     let mut element_index_1 = 0;
//     let mut element_index_2 = 0;
//     let mut node_c = 0;
//     let mut node_d = 0;
//     let element_node_connectivity = &mut fem.element_node_connectivity;
//     let node_element_connectivity = &mut fem.node_element_connectivity;
//     edges
//         .iter()
//         .zip(lengths.iter())
//         .filter(|&(_, &mut length)| length < FOUR_FIFTHS * average_length)
//         .for_each(|([node_a, node_b], length)| {
//             [element_index_1, element_index_2, node_c, node_d] = edge_info(
//                 *node_a,
//                 *node_b,
//                 element_node_connectivity,
//                 node_element_connectivity,
//             );
//         })
// }

fn flip_edges(fem: &mut TriangularFiniteElements, edges: &mut Edges, lengths: &mut Lengths) {
    let mut before = 0;
    let mut after = 0;
    let mut element_index_1 = 0;
    let mut element_index_2 = 0;
    let mut node_c = 0;
    let mut node_d = 0;
    let mut spot_a = 0;
    let mut spot_b = 0;
    let element_node_connectivity = &mut fem.element_node_connectivity;
    let node_element_connectivity = &mut fem.node_element_connectivity;
    let node_node_connectivity = &mut fem.node_node_connectivity;
    let nodal_coordinates = &fem.nodal_coordinates;
    edges
        .iter_mut()
        .zip(lengths.iter_mut())
        .for_each(|([node_a, node_b], length)| {
            [element_index_1, element_index_2, node_c, node_d] = edge_info(
                *node_a,
                *node_b,
                element_node_connectivity,
                node_element_connectivity,
            );
            before = [*node_a, *node_b, node_c, node_d]
                .iter()
                .map(|&node| (node_node_connectivity[node].len() as i8 - REGULAR_DEGREE).abs())
                .sum();
            after = [*node_a, *node_b, node_c, node_d]
                .iter()
                .zip([-1, -1, 1, 1].iter())
                .map(|(&node, change)| {
                    (node_node_connectivity[node].len() as i8 - REGULAR_DEGREE + change).abs()
                })
                .sum();
            if before > after {
                spot_a = element_node_connectivity[element_index_1]
                    .iter()
                    .position(|node| node == node_a)
                    .unwrap();
                spot_b = element_node_connectivity[element_index_1]
                    .iter()
                    .position(|node| node == node_b)
                    .unwrap();
                if (spot_a == 0 && spot_b == 1)
                    || (spot_a == 1 && spot_b == 2)
                    || (spot_a == 2 && spot_b == 0)
                {
                    element_node_connectivity[element_index_1] = [*node_b, node_c, node_d];
                    element_node_connectivity[element_index_2] = [*node_a, node_d, node_c];
                } else {
                    element_node_connectivity[element_index_1] = [node_c, *node_b, node_d];
                    element_node_connectivity[element_index_2] = [node_d, *node_a, node_c];
                }
                node_element_connectivity[*node_a].retain(|element| element != &element_index_1);
                node_element_connectivity[*node_b].retain(|element| element != &element_index_2);
                node_element_connectivity[node_c].push(element_index_2);
                node_element_connectivity[node_d].push(element_index_1);
                node_node_connectivity[*node_a].retain(|node| node != node_b);
                node_node_connectivity[*node_b].retain(|node| node != node_a);
                node_node_connectivity[node_c].push(node_d);
                node_node_connectivity[node_c].sort();
                node_node_connectivity[node_d].push(node_c);
                node_node_connectivity[node_d].sort();
                if node_c < node_d {
                    *node_a = node_c;
                    *node_b = node_d;
                } else {
                    *node_a = node_d;
                    *node_b = node_c;
                }
                *length = (&nodal_coordinates[*node_a] - &nodal_coordinates[*node_b]).norm();
            }
        });
}

// fn edge_info(
pub fn edge_info(
    node_a: usize,
    node_b: usize,
    element_node_connectivity: &Connectivity<TRI>,
    node_element_connectivity: &VecConnectivity,
) -> [usize; 4] {
    let [&element_index_1, &element_index_2] = node_element_connectivity[node_a]
        .iter()
        .filter(|element_a| node_element_connectivity[node_b].contains(element_a))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let node_c = *element_node_connectivity[element_index_1]
        .iter()
        .find(|node_1| !element_node_connectivity[element_index_2].contains(node_1))
        .unwrap();
    let node_d = *element_node_connectivity[element_index_2]
        .iter()
        .find(|node_2| !element_node_connectivity[element_index_1].contains(node_2))
        .unwrap();
    [element_index_1, element_index_2, node_c, node_d]
}
