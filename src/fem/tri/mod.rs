#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    super::tree::Edges, Coordinate, Coordinates, ELEMENT_NUMBERING_OFFSET, FiniteElementMethods,
    FiniteElementSpecifics, FiniteElements, Metrics, NODE_NUMBERING_OFFSET, Tessellation,
    VecConnectivity, Vector,
};
use conspire::{
    math::{Tensor, TensorArray, TensorVec},
    mechanics::Scalar,
};
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    path::Path,
};

const FOUR_THIRDS: Scalar = 4.0 / 3.0;
const FOUR_FIFTHS: Scalar = 4.0 / 5.0;
const J_EQUILATERAL: Scalar = 0.8660254037844387;
const REGULAR_DEGREE: i8 = 6;

/// The number of nodes in a triangular finite element.
pub const TRI: usize = 3;

type Connectivity = super::Connectivity<TRI>;
type Lengths = conspire::math::Vector;

/// The triangular finite elements type.
pub type TriangularFiniteElements = FiniteElements<TRI>;

impl From<Tessellation> for TriangularFiniteElements {
    fn from(tessellation: Tessellation) -> Self {
        let data = tessellation.get_data();
        let element_blocks = vec![1; data.faces.len()];
        let nodal_coordinates = data
            .vertices
            .iter()
            .map(|&vertex| Coordinate::new([vertex[0].into(), vertex[1].into(), vertex[2].into()]))
            .collect();
        let element_node_connectivity = data
            .faces
            .iter()
            .map(|face| {
                [
                    face.vertices[0] + NODE_NUMBERING_OFFSET,
                    face.vertices[1] + NODE_NUMBERING_OFFSET,
                    face.vertices[2] + NODE_NUMBERING_OFFSET,
                ]
            })
            .collect();
        TriangularFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        )
    }
}

impl FiniteElementSpecifics for TriangularFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 2],
            1 => vec![0, 2],
            2 => vec![0, 1],
            _ => panic!(),
        }
    }
    fn maximum_edge_ratios(&self) -> Metrics {
        // Knupp 2006
        // https://www.osti.gov/servlets/purl/901967
        // page 19 and 26
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = 0.0;
        let mut l1 = 0.0;
        let mut l2 = 0.0;
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = (&nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET])
                    .norm();
                l1 = (&nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET])
                    .norm();
                l2 = (&nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET])
                    .norm();
                [l0, l1, l2].into_iter().reduce(f64::max).unwrap()
                    / [l0, l1, l2].into_iter().reduce(f64::min).unwrap()
            })
            .collect()
    }
    fn maximum_skews(&self) -> Metrics {
        let deg_to_rad = std::f64::consts::PI / 180.0;
        let equilateral_rad = 60.0 * deg_to_rad;
        let minimum_angles = self.minimum_angles();
        minimum_angles
            .iter()
            .map(|angle| (equilateral_rad - angle) / (equilateral_rad))
            .collect()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        self.minimum_angles()
            .iter()
            .map(|angle| (angle.sin() / J_EQUILATERAL))
            .collect()
    }
    fn write_metrics(&self, file_path: &str) -> Result<(), ErrorIO> {
        let areas = self.areas();
        let maximum_skews = self.maximum_skews();
        let maximum_edge_ratios = self.maximum_edge_ratios();
        let minimum_angles = self.minimum_angles();
        let minimum_scaled_jacobians = self.minimum_scaled_jacobians();
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut file = BufWriter::new(File::create(file_path)?);
        let input_extension = Path::new(&file_path)
            .extension()
            .and_then(|ext| ext.to_str());
        match input_extension {
            Some("csv") => {
                let header_string =
                    "maximum edge ratio,minimum scaled jacobian,maximum skew,element volume\n"
                        .to_string();
                file.write_all(header_string.as_bytes())?;
                maximum_edge_ratios
                    .iter()
                    .zip(
                        minimum_scaled_jacobians.iter().zip(
                            maximum_skews
                                .iter()
                                .zip(areas.iter().zip(minimum_angles.iter())),
                        ),
                    )
                    .try_for_each(
                        |(
                            maximum_edge_ratio,
                            (minimum_scaled_jacobian, (maximum_skew, (area, minimum_angle))),
                        )| {
                            file.write_all(
                                format!(
                                    "{maximum_edge_ratio:>10.6e},{minimum_scaled_jacobian:>10.6e},{maximum_skew:>10.6e},{area:>10.6e},{minimum_angle:>10.6e}\n",
                                )
                                .as_bytes(),
                            )
                        },
                    )?;
                file.flush()?
            }
            Some("npy") => {
                let n_columns = 5; // total number of triangle metrics
                let idx_ratios = 0; // maximum edge ratios
                let idx_jacobians = 1; // minimum scaled jacobians
                let idx_skews = 2; // maximum skews
                let idx_areas = 3; // areas
                let idx_angles = 4; // minimum angles
                let mut metrics_set =
                    Array2::<f64>::from_elem((minimum_scaled_jacobians.len(), n_columns), 0.0);
                metrics_set
                    .slice_mut(s![.., idx_ratios])
                    .assign(&maximum_edge_ratios);
                metrics_set
                    .slice_mut(s![.., idx_jacobians])
                    .assign(&minimum_scaled_jacobians);
                metrics_set
                    .slice_mut(s![.., idx_skews])
                    .assign(&maximum_skews);
                metrics_set.slice_mut(s![.., idx_areas]).assign(&areas);
                metrics_set
                    .slice_mut(s![.., idx_angles])
                    .assign(&minimum_angles);
                metrics_set.write_npy(file).unwrap();
            }
            _ => panic!("print error message with input and extension"),
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mWriting triangle metrics to file\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(())
    }
}

impl TriangularFiniteElements {
    fn areas(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = Vector::zero();
        let mut l1 = Vector::zero();
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET];
                l1 = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET];
                0.5 * (l0.cross(&l1)).norm()
            })
            .collect()
    }
    fn minimum_angles(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = Vector::zero();
        let mut l1 = Vector::zero();
        let mut l2 = Vector::zero();
        let flip = -1.0;
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET];
                l1 = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET];
                l2 = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET];
                l0.normalize();
                l1.normalize();
                l2.normalize();
                [
                    ((&l0 * flip) * &l1).acos(),
                    ((&l1 * flip) * &l2).acos(),
                    ((&l2 * flip) * &l0).acos(),
                ]
                .into_iter()
                .reduce(f64::min)
                .unwrap()
            })
            .collect()
    }
    /// Isotropic remeshing of the finite elements.
    pub fn remesh(&mut self, iterations: usize, smoothing_iterations: usize) {
        remesh(self, iterations, smoothing_iterations)
    }
}

fn remesh(fem: &mut TriangularFiniteElements, iterations: usize, smoothing_iterations: usize) {
    (0..iterations).for_each(|_| {
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
        let mut lengths: Lengths = edges
        let lengths: Lengths = edges
            .iter()
            .map(|&[node_a, node_b]| {
                (&fem.get_nodal_coordinates()[node_a - NODE_NUMBERING_OFFSET]
                    - &fem.get_nodal_coordinates()[node_b - NODE_NUMBERING_OFFSET])
                    .norm()
            })
            .collect();
        let average_length = lengths.iter().sum::<Scalar>() / (lengths.len() as Scalar);
        // edges.iter_mut().zip(lengths.iter_mut()).for_each(|([node_a, node_b], length)|
        //     if *length > FOUR_THIRDS * average_length {
        //         println!("large {:?}", length)
        //     } else if *length < FOUR_FIFTHS * average_length {
        //         println!("small {:?}", length)
        //     }
        // );
        // fem.node_element_connectivity().unwrap();
        // fem.node_node_connectivity().unwrap();
        split_edges(fem, &edges, &lengths, average_length);
        // collapse_edges(fem, &mut edges, lengths, average_length);
        // flip_edges(fem, edges);
        // Vertex averaging is Laplace smoothing but while removing the normal component.
    });
    fem.nodal_influencers = vec![];
}

fn split_edges(
    fem: &mut TriangularFiniteElements,
    edges: &Edges,
    lengths: &Lengths,
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
    let element_node_connectivity = &mut fem.element_node_connectivity;
    let node_element_connectivity = &mut fem.node_element_connectivity;
    let node_node_connectivity = &mut fem.node_node_connectivity;
    let nodal_coordinates = &mut fem.nodal_coordinates;
    edges
        .iter()
        .zip(lengths.iter())
        .filter(|&(_, &length)| length > FOUR_THIRDS * average_length)
        .for_each(|(&[node_a, node_b], _)| {
            [element_index_1, element_index_2, node_c, node_d] = edge_info(
                node_a,
                node_b,
                element_node_connectivity,
                node_element_connectivity,
            );
            nodal_coordinates.push(
                (nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone()
                    + &nodal_coordinates[node_b - NODE_NUMBERING_OFFSET])
                    / 2.0,
            );
            node_e = nodal_coordinates.len();
            spot_a = element_node_connectivity[element_index_1]
                .iter()
                .position(|node| node == &node_a)
                .unwrap();
            spot_b = element_node_connectivity[element_index_1]
                .iter()
                .position(|node| node == &node_b)
                .unwrap();
            if (spot_a == 0 && spot_b == 1)
                || (spot_a == 1 && spot_b == 2)
                || (spot_a == 2 && spot_b == 0)
            {
                element_node_connectivity[element_index_1] = [node_c, node_e, node_b];
                element_node_connectivity[element_index_2] = [node_a, node_e, node_c];
                element_node_connectivity.push([node_d, node_e, node_a]);
                element_node_connectivity.push([node_b, node_e, node_d]);
            } else {
                element_node_connectivity[element_index_1] = [node_e, node_c, node_b];
                element_node_connectivity[element_index_2] = [node_e, node_a, node_c];
                element_node_connectivity.push([node_e, node_d, node_a]);
                element_node_connectivity.push([node_e, node_b, node_d]);
            }
            element_index_3 = element_node_connectivity.len() - 2;
            element_index_4 = element_node_connectivity.len() - 1;
            node_element_connectivity[node_a - NODE_NUMBERING_OFFSET]
                .retain(|element| element - ELEMENT_NUMBERING_OFFSET != element_index_1);
            node_element_connectivity[node_a - NODE_NUMBERING_OFFSET]
                .push(element_index_3 + ELEMENT_NUMBERING_OFFSET);
            node_element_connectivity[node_b - NODE_NUMBERING_OFFSET]
                .retain(|element| element - ELEMENT_NUMBERING_OFFSET != element_index_2);
            node_element_connectivity[node_b - NODE_NUMBERING_OFFSET]
                .push(element_index_4 + ELEMENT_NUMBERING_OFFSET);
            node_element_connectivity[node_c - NODE_NUMBERING_OFFSET]
                .push(element_index_2 + ELEMENT_NUMBERING_OFFSET);
            node_element_connectivity[node_d - NODE_NUMBERING_OFFSET]
                .push(element_index_1 + ELEMENT_NUMBERING_OFFSET);
            node_element_connectivity[node_d - NODE_NUMBERING_OFFSET].retain(|element| {
                element - ELEMENT_NUMBERING_OFFSET != element_index_1
                    && element - ELEMENT_NUMBERING_OFFSET != element_index_2
            });
            node_element_connectivity[node_d - NODE_NUMBERING_OFFSET].extend(vec![
                element_index_3 + ELEMENT_NUMBERING_OFFSET,
                element_index_4 + ELEMENT_NUMBERING_OFFSET,
            ]);
            node_element_connectivity.push(vec![
                element_index_1 + ELEMENT_NUMBERING_OFFSET,
                element_index_2 + ELEMENT_NUMBERING_OFFSET,
                element_index_3 + ELEMENT_NUMBERING_OFFSET,
                element_index_4 + ELEMENT_NUMBERING_OFFSET,
            ]);
            node_node_connectivity[node_a - NODE_NUMBERING_OFFSET].retain(|node| node != &node_b);
            node_node_connectivity[node_a - NODE_NUMBERING_OFFSET].push(node_e);
            node_node_connectivity[node_a - NODE_NUMBERING_OFFSET].sort();
            node_node_connectivity[node_b - NODE_NUMBERING_OFFSET].retain(|node| node != &node_a);
            node_node_connectivity[node_b - NODE_NUMBERING_OFFSET].push(node_e);
            node_node_connectivity[node_b - NODE_NUMBERING_OFFSET].sort();
            node_node_connectivity[node_c - NODE_NUMBERING_OFFSET].push(node_e);
            node_node_connectivity[node_c - NODE_NUMBERING_OFFSET].sort();
            node_node_connectivity[node_d - NODE_NUMBERING_OFFSET].push(node_e);
            node_node_connectivity[node_d - NODE_NUMBERING_OFFSET].sort();
            node_node_connectivity.push(vec![node_a, node_b, node_c, node_d]);
            node_node_connectivity[node_e - NODE_NUMBERING_OFFSET].sort();
        })
    //
    // edit edges and lengths?
    //
}

fn collapse_edges(
    fem: &mut TriangularFiniteElements,
    edges: &mut Edges,
    lengths: Lengths,
    average_length: Scalar,
) {
    let mut element_index_1 = 0;
    let mut element_index_2 = 0;
    let element_node_connectivity = &mut fem.element_node_connectivity;
    let node_element_connectivity = &mut fem.node_element_connectivity;
    let node_node_connectivity = &mut fem.node_node_connectivity;
    let nodal_coordinates = &mut fem.nodal_coordinates;
    let mut coincident_nodes = vec![None; nodal_coordinates.len()];
    let mut degenerate_elements = vec![false; element_node_connectivity.len()];
    edges
        .iter()
        // .zip(lengths.iter())
        // .filter(|&(_, &length)| length < FOUR_FIFTHS * average_length)
        // .filter(|&[node_a, node_b]|
        //     (&nodal_coordinates[node_a - NODE_NUMBERING_OFFSET] - &nodal_coordinates[node_b - NODE_NUMBERING_OFFSET]).norm() < FOUR_FIFTHS * average_length
        // )
        // .for_each(|(&[node_a, node_b], _)| {
        .for_each(|&[node_a, node_b]| {
            if (&nodal_coordinates[node_a - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[node_b - NODE_NUMBERING_OFFSET])
                .norm()
                < FOUR_FIFTHS * average_length
            {
                [element_index_1, element_index_2, _, _] = edge_info(
                    node_a,
                    node_b,
                    element_node_connectivity,
                    node_element_connectivity,
                );
                // nodal_coordinates[node_b - NODE_NUMBERING_OFFSET] =
                //     nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone();
                // coincident_nodes[node_b - NODE_NUMBERING_OFFSET] =
                //     coincident_nodes[node_a - NODE_NUMBERING_OFFSET].or(Some(node_a));

                nodal_coordinates[node_a - NODE_NUMBERING_OFFSET] =
                    (nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone()
                        + &nodal_coordinates[node_b - NODE_NUMBERING_OFFSET])
                        / 2.0;
                nodal_coordinates[node_b - NODE_NUMBERING_OFFSET] =
                    nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone();
                coincident_nodes[node_b - NODE_NUMBERING_OFFSET] =
                    coincident_nodes[node_a - NODE_NUMBERING_OFFSET].or(Some(node_a));

                let foo = coincident_nodes[node_a - NODE_NUMBERING_OFFSET].or(Some(node_a));

                // coincident_nodes
                //     .iter()
                //     .filter(|&&coincident_node| {
                //         coincident_node != Some(node_a) && coincident_nodes[node_a - NODE_NUMBERING_OFFSET].is_some() && coincident_node == coincident_nodes[node_a - NODE_NUMBERING_OFFSET]
                //     })
                //     .for_each(|coincident_node| {
                //         panic!("{:?}", (coincident_node, node_a, coincident_nodes[node_a - NODE_NUMBERING_OFFSET]))
                //     });

                // coincident_nodes
                //     .iter()
                //     .filter(|&&coincident_node| {
                //         coincident_node == Some(node_b)
                //     })
                //     .for_each(|coincident_node| {
                //         panic!()
                //     });

                coincident_nodes
                    .iter()
                    .enumerate()
                    // .filter_map(|(_, &coincident_node)| coincident_node)
                    .filter(|&(_, &coincident_node)| {
                        coincident_node == Some(node_a) || coincident_node == Some(node_b) || coincident_node == foo
                    })
                    .for_each(|(node_index, coincident_node)| {
                        nodal_coordinates[node_index] =
                            nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone();
                        nodal_coordinates[coincident_node.unwrap() - NODE_NUMBERING_OFFSET] =
                            nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone()
                    });

                // nodal_coordinates[node_a - NODE_NUMBERING_OFFSET] =
                //     (nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone()
                //         + &nodal_coordinates[node_b - NODE_NUMBERING_OFFSET])
                //         / 2.0;
                // nodal_coordinates[node_b - NODE_NUMBERING_OFFSET] =
                //     nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone();
                // if let Some(coincident_node) = coincident_nodes[node_a - NODE_NUMBERING_OFFSET] {
                //     nodal_coordinates[coincident_node - NODE_NUMBERING_OFFSET] =
                //         nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone();
                //     coincident_nodes[node_b - NODE_NUMBERING_OFFSET] = Some(coincident_node);
                // } else {
                //     coincident_nodes[node_b - NODE_NUMBERING_OFFSET] = Some(node_a);
                // }

                // nodal_coordinates[node_a - NODE_NUMBERING_OFFSET] =
                //     (nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone()
                //         + &nodal_coordinates[node_b - NODE_NUMBERING_OFFSET])
                //         / 2.0;
                // nodal_coordinates[node_b - NODE_NUMBERING_OFFSET] =
                //     nodal_coordinates[node_a - NODE_NUMBERING_OFFSET].clone();
                // coincident_nodes[node_b - NODE_NUMBERING_OFFSET] =
                //     coincident_nodes[node_a - NODE_NUMBERING_OFFSET].or(Some(node_a));
                //
                // Do you have to move the other coordinates too?
                //
                degenerate_elements[element_index_1] = true;
                degenerate_elements[element_index_2] = true;
            }
        });

    let mut node_mapping = vec![0; coincident_nodes.len()];
    coincident_nodes
        .iter()
        .enumerate()
        .filter(|&(_, coincident_node)| coincident_node.is_none())
        .enumerate()
        .for_each(|(node, (index, _))| node_mapping[index] = node + NODE_NUMBERING_OFFSET);

    let mut element_mapping = vec![0; degenerate_elements.len()];
    degenerate_elements
        .iter()
        .enumerate()
        .filter(|&(_, &is_degenerate)| is_degenerate)
        .enumerate()
        .for_each(|(element, (index, _))| {
            element_mapping[index] = element + ELEMENT_NUMBERING_OFFSET
        });

    let mut offset = 0;
    coincident_nodes
        .iter()
        .enumerate()
        .filter(|(_, coincident_node)| coincident_node.is_some())
        .for_each(|(node, _)| {
            // nodal_coordinates.remove(node - NODE_NUMBERING_OFFSET - offset);
            // node_element_connectivity.remove(node - NODE_NUMBERING_OFFSET - offset);
            // node_node_connectivity.remove(node - NODE_NUMBERING_OFFSET - offset);
            nodal_coordinates.remove(node - offset);
            node_element_connectivity.remove(node - offset);
            node_node_connectivity.remove(node - offset);
            offset += 1;
        });

    offset = 0;
    degenerate_elements
        .iter()
        .enumerate()
        .filter(|&(_, &degenerate_element)| degenerate_element)
        .for_each(|(element, _)| {
            // element_node_connectivity.remove(element - ELEMENT_NUMBERING_OFFSET - offset);
            element_node_connectivity.remove(element - offset);
            offset += 1;
        });

    // println!("{:?}", element_node_connectivity[20]);

    coincident_nodes
        .iter()
        .enumerate()
        .filter(|(_, coincident_node)| coincident_node.is_some())
        .for_each(|(node_b_index, node_a)| {
            element_node_connectivity
                .iter_mut()
                .for_each(|connectivity| {
                    connectivity.iter_mut().for_each(|entry| {
                        if *entry == node_b_index + NODE_NUMBERING_OFFSET {
                            *entry = node_a.unwrap()
                            // *entry = node_mapping[node_a.unwrap() - NODE_NUMBERING_OFFSET]
                        }
                    })
                })
        });
    // need to fix node_node_conn and edges here too?

    // println!("{:?}", element_node_connectivity[20]);

    // // println!("{:?}, {:?}, {:?}, {:?}", coincident_nodes[14 - 1], coincident_nodes[13 - 1], node_mapping[14 - 1], node_mapping[13 - 1]);
    // println!(
    //     "{:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
    //     coincident_nodes[26 - 1],
    //     coincident_nodes[20 - 1],
    //     coincident_nodes[19 - 1],
    //     node_mapping[26 - 1],
    //     node_mapping[20 - 1],
    //     node_mapping[19 - 1]
    // );

    element_node_connectivity
        .iter_mut()
        .for_each(|connectivity| {
            connectivity
                .iter_mut()
                .for_each(|node| *node = node_mapping[*node - NODE_NUMBERING_OFFSET])
        });

    // let foo = element_node_connectivity.iter().flatten().max().unwrap();
    // let bar = node_mapping.iter().max().unwrap();
    // let baz = node_mapping.iter().filter(|entry| entry != &&0).count();
    // println!("{:?}", (nodal_coordinates.len(), foo, bar, baz));

    node_node_connectivity.iter_mut().for_each(|connectivity| {
        connectivity
            .iter_mut()
            .for_each(|node| *node = node_mapping[*node - NODE_NUMBERING_OFFSET])
    });

    node_element_connectivity
        .iter_mut()
        .for_each(|connectivity| {
            connectivity
                .iter_mut()
                .for_each(|element| *element = element_mapping[*element - ELEMENT_NUMBERING_OFFSET])
        });

    //
    // edit edges?
    //
}

fn flip_edges(fem: &mut TriangularFiniteElements, edges: Edges) {
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
    edges.into_iter().for_each(|[node_a, node_b]| {
        [element_index_1, element_index_2, node_c, node_d] = edge_info(
            node_a,
            node_b,
            element_node_connectivity,
            node_element_connectivity,
        );
        before = [node_a, node_b, node_c, node_d]
            .iter()
            .map(|node| {
                (node_node_connectivity[node - NODE_NUMBERING_OFFSET].len() as i8 - REGULAR_DEGREE)
                    .abs()
            })
            .sum();
        after = [node_a, node_b, node_c, node_d]
            .iter()
            .zip([-1, -1, 1, 1].iter())
            .map(|(node, change)| {
                (node_node_connectivity[node - NODE_NUMBERING_OFFSET].len() as i8 - REGULAR_DEGREE
                    + change)
                    .abs()
            })
            .sum();
        if before > after {
            spot_a = element_node_connectivity[element_index_1]
                .iter()
                .position(|node| node == &node_a)
                .unwrap();
            spot_b = element_node_connectivity[element_index_1]
                .iter()
                .position(|node| node == &node_b)
                .unwrap();
            if (spot_a == 0 && spot_b == 1)
                || (spot_a == 1 && spot_b == 2)
                || (spot_a == 2 && spot_b == 0)
            {
                element_node_connectivity[element_index_1] = [node_b, node_c, node_d];
                element_node_connectivity[element_index_2] = [node_a, node_d, node_c];
            } else {
                element_node_connectivity[element_index_1] = [node_c, node_b, node_d];
                element_node_connectivity[element_index_2] = [node_d, node_a, node_c];
            }
            node_element_connectivity[node_a - NODE_NUMBERING_OFFSET]
                .retain(|element| element - ELEMENT_NUMBERING_OFFSET != element_index_1);
            node_element_connectivity[node_b - NODE_NUMBERING_OFFSET]
                .retain(|element| element - ELEMENT_NUMBERING_OFFSET != element_index_2);
            node_element_connectivity[node_c - NODE_NUMBERING_OFFSET]
                .push(element_index_2 + ELEMENT_NUMBERING_OFFSET);
            node_element_connectivity[node_d - NODE_NUMBERING_OFFSET]
                .push(element_index_1 + ELEMENT_NUMBERING_OFFSET);
            node_node_connectivity[node_a - NODE_NUMBERING_OFFSET].retain(|node| node != &node_b);
            node_node_connectivity[node_b - NODE_NUMBERING_OFFSET].retain(|node| node != &node_a);
            node_node_connectivity[node_c - NODE_NUMBERING_OFFSET].push(node_d);
            node_node_connectivity[node_c - NODE_NUMBERING_OFFSET].sort();
            node_node_connectivity[node_d - NODE_NUMBERING_OFFSET].push(node_c);
            node_node_connectivity[node_d - NODE_NUMBERING_OFFSET].sort();
        }
    });
}

fn edge_info(
    node_a: usize,
    node_b: usize,
    element_node_connectivity: &Connectivity,
    node_element_connectivity: &VecConnectivity,
) -> [usize; 4] {
    let [element_index_1, element_index_2] = node_element_connectivity
        [node_a - NODE_NUMBERING_OFFSET]
        .iter()
        .filter(|element_a| {
            node_element_connectivity[node_b - NODE_NUMBERING_OFFSET].contains(element_a)
        })
        .map(|element_a| element_a - ELEMENT_NUMBERING_OFFSET)
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
