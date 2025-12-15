#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    super::{Vectors, tree::Edges},
    Connectivity, Coordinate, Coordinates, FiniteElementMethods, FiniteElementSpecifics,
    FiniteElements, Metrics, Size, Smoothing, Tessellation, VecConnectivity, Vector,
};
use conspire::{
    math::{Tensor, TensorArray, TensorVec, Vector as VectorConspire},
    mechanics::Scalar,
};
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    f64::consts::PI,
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

const NUM_NODES_FACE: usize = 1;
const NUM_NODES_TRI: usize = 3;

type Curvatures = VectorConspire;
type Lengths = conspire::math::Vector;

/// The triangular finite elements type.
pub type TriangularFiniteElements = FiniteElements<TRI>;

impl From<Tessellation> for TriangularFiniteElements {
    fn from(tessellation: Tessellation) -> Self {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let data = tessellation.data();
        let element_blocks = vec![1; data.faces.len()];
        let nodal_coordinates = data
            .vertices
            .into_iter()
            .map(|vertex| Coordinate::new([vertex[0] as f64, vertex[1] as f64, vertex[2] as f64]))
            .collect();
        let element_node_connectivity = data.faces.into_iter().map(|face| face.vertices).collect();
        let triangular_finite_elements = TriangularFiniteElements::from((
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ));
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mSerializing triangles\x1b[0m {:?}",
            time.elapsed()
        );
        triangular_finite_elements
    }
}

impl FiniteElementSpecifics<NUM_NODES_FACE> for TriangularFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 2],
            1 => vec![0, 2],
            2 => vec![0, 1],
            _ => panic!(),
        }
    }
    fn exterior_faces(&self) -> Connectivity<NUM_NODES_FACE> {
        unimplemented!()
    }
    fn exterior_faces_interior_points(&self, _grid_length: usize) -> Coordinates {
        todo!()
    }
    fn faces(&self) -> Connectivity<NUM_NODES_FACE> {
        unimplemented!()
    }
    fn interior_points(&self, _grid_length: usize) -> Coordinates {
        todo!()
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
                l0 = (&nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]])
                    .norm();
                l1 = (&nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]])
                    .norm();
                l2 = (&nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]])
                    .norm();
                [l0, l1, l2].into_iter().reduce(f64::max).unwrap()
                    / [l0, l1, l2].into_iter().reduce(f64::min).unwrap()
            })
            .collect()
    }
    fn maximum_skews(&self) -> Metrics {
        let deg_to_rad = PI / 180.0;
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
            .map(|angle| angle.sin() / J_EQUILATERAL)
            .collect()
    }
    fn remesh(&mut self, iterations: usize, smoothing_method: &Smoothing, size: Size) {
        remesh(self, iterations, smoothing_method, size)
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
            _ => panic!(
                "Unsupported file extension for metrics output: {:?}.  Please use 'csv' or 'npy'.",
                input_extension
            ),
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mWriting triangular metrics to file\x1b[0m {:?}",
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
                l0 = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]];
                l1 = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]];
                0.5 * (l0.cross(&l1)).norm()
            })
            .collect()
    }
    /// Computes and returns the closest point in the triangle to another point.
    pub fn closest_point(
        point: &Coordinate,
        coordinates: &Coordinates,
        [node_0, node_1, node_2]: [usize; TRI],
    ) -> Coordinate {
        let coordinates_0 = &coordinates[node_0];
        let coordinates_1 = &coordinates[node_1];
        let coordinates_2 = &coordinates[node_2];
        let v_01 = coordinates_1 - coordinates_0;
        let v_02 = coordinates_2 - coordinates_0;
        let v_0p = point - coordinates_0;
        let d1 = &v_01 * &v_0p;
        let d2 = &v_02 * v_0p;
        if d1 <= 0.0 && d2 <= 0.0 {
            return coordinates_0.clone();
        }
        let v_1p = point - coordinates_1;
        let d3 = &v_01 * &v_1p;
        let d4 = &v_02 * v_1p;
        if d3 >= 0.0 && d4 <= d3 {
            return coordinates_1.clone();
        }
        let v_2p = point - coordinates_2;
        let d5 = &v_01 * &v_2p;
        let d6 = &v_02 * v_2p;
        if d6 >= 0.0 && d5 <= d6 {
            return coordinates_2.clone();
        }
        let vc = d1 * d4 - d3 * d2;
        if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            return coordinates_0 + v_01 * (d1 / (d1 - d3));
        }
        let vb = d5 * d2 - d1 * d6;
        if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            return coordinates_0 + v_02 * (d2 / (d2 - d6));
        }
        let va = d3 * d6 - d5 * d4;
        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
            return coordinates_1
                + (coordinates_2 - coordinates_1) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
        }
        let denom = va + vb + vc;
        coordinates_0 + v_01 * (vb / denom) + v_02 * (vc / denom)
    }
    /// Calculates and returns the Gaussian curvature.
    pub fn curvature(&self) -> Result<Curvatures, String> {
        let mut edge = Vector::zero();
        let mut edge_norm = 0.0;
        let mut edges_weight = 0.0;
        let mut element_index_1 = 0;
        let mut element_index_2 = 0;
        let mut node_c = 0;
        let mut node_d = 0;
        let element_node_connectivity = self.get_element_node_connectivity();
        let node_element_connectivity = self.get_node_element_connectivity();
        let node_node_connectivity = self.get_node_node_connectivity();
        let nodal_coordinates = self.get_nodal_coordinates();
        if !node_node_connectivity.is_empty() {
            Ok(self
                .get_nodal_coordinates()
                .iter()
                .zip(node_node_connectivity.iter().enumerate())
                .map(|(coordinates_a, (node_a, nodes))| {
                    edges_weight = 0.0;
                    nodes
                        .iter()
                        .map(|&node_b| {
                            [element_index_1, element_index_2, node_c, node_d] = edge_info(
                                node_a,
                                node_b,
                                element_node_connectivity,
                                node_element_connectivity,
                            );
                            edge = coordinates_a - &nodal_coordinates[node_b];
                            edge_norm = edge.norm();
                            edges_weight += edge_norm;
                            ((&nodal_coordinates[node_c] - &nodal_coordinates[node_a])
                                .cross(&(&nodal_coordinates[node_b] - &nodal_coordinates[node_c]))
                                .normalized()
                                * (&nodal_coordinates[node_d] - &nodal_coordinates[node_b])
                                    .cross(
                                        &(&nodal_coordinates[node_a] - &nodal_coordinates[node_d]),
                                    )
                                    .normalized())
                            .acos()
                                / PI
                                * edge_norm
                        })
                        .sum::<Scalar>()
                        / edges_weight
                })
                .collect())
        } else {
            Err("Need to calculate the node-to-node connectivity first".to_string())
        }
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
                l0 = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]];
                l1 = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]];
                l2 = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]];
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
    /// Computes and returns the normal vector for a triangle.
    pub fn normal(coordinates: &Coordinates, [node_0, node_1, node_2]: [usize; TRI]) -> Vector {
        (&coordinates[node_1] - &coordinates[node_0])
            .cross(&(&coordinates[node_2] - &coordinates[node_0]))
            .normalized()
    }
    /// Computes and returns the normal vectors for all triangles.
    pub fn normals(&self) -> Vectors {
        let coordinates = self.get_nodal_coordinates();
        self.get_element_node_connectivity()
            .iter()
            .map(|&connectivity| Self::normal(coordinates, connectivity))
            .collect()
    }
    /// Iteratively refine until all edges are smaller than a size.
    pub fn refine(&mut self, size: Scalar) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut edges: Edges = self
            .get_element_node_connectivity()
            .iter()
            .flat_map(|&[node_0, node_1, node_2]| {
                [[node_0, node_1], [node_1, node_2], [node_2, node_0]].into_iter()
            })
            .collect();
        edges.iter_mut().for_each(|edge| edge.sort());
        edges.sort();
        edges.dedup();
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut lengths = Lengths::zero(edges.len());
        edges
            .iter()
            .zip(lengths.iter_mut())
            .for_each(|(&[node_a, node_b], length)| {
                *length = (&nodal_coordinates[node_a] - &nodal_coordinates[node_b]).norm()
            });
        self.boundary_nodes = vec![];
        self.exterior_nodes = vec![];
        self.interface_nodes = vec![];
        self.interior_nodes = vec![];
        loop {
            if lengths.iter().any(|length| length > &size) {
                split_edges(self, &mut edges, &mut lengths, size / FOUR_THIRDS)
                //
                // Would be nice to bake this into remeshing with two options based on input args:
                // (1) just go for some number of iterations like typically do
                // (2) iterate until all edge below size
                //     - would still do edge collapse (get all edges close to size, just no larger than size)
                //
            } else {
                break;
            }
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mSplitting large edges\x1b[0m {:?}",
            time.elapsed()
        );
    }
}

fn remesh(
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
    // edges
    //     .iter()
    //     .zip(lengths.iter_mut())
    //     .for_each(|(&[node_a, node_b], length)| {
    //         *length =
    //             (&fem.get_nodal_coordinates()[node_a] - &fem.get_nodal_coordinates()[node_b]).norm()
    //     });
    fem.boundary_nodes = vec![];
    fem.exterior_nodes = vec![];
    fem.interface_nodes = vec![];
    fem.interior_nodes = vec![];
    (0..iterations).for_each(|_| {
        edges
            .iter()
            .zip(lengths.iter_mut())
            .for_each(|(&[node_a, node_b], length)| {
                *length = (&fem.get_nodal_coordinates()[node_a]
                    - &fem.get_nodal_coordinates()[node_b])
                    .norm()
            });
        average_length = if let Some(size) = size {
            size / FOUR_THIRDS
        } else {
            lengths.iter().sum::<Scalar>() / (lengths.len() as Scalar)
        };
        // split_edges(fem, &mut edges, &mut lengths, average_length);
        collapse_edges(fem, &mut edges, &mut lengths, average_length);
        // flip_edges(fem, &mut edges);
        // fem.nodal_influencers();
        // fem.smooth(smoothing_method).unwrap();
    });
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mIsotropic remesh tris\x1b[0m {:?}",
        time.elapsed()
    );
}

fn split_edges(
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

fn collapse_edges(
    fem: &mut TriangularFiniteElements,
    edges: &mut Edges,
    lengths: &mut Lengths,
    average_length: Scalar,
) {
    let mut element_index_1 = 0;
    let mut element_index_2 = 0;
    // let element_blocks = &mut fem.element_blocks;
    // let element_node_connectivity = &mut fem.element_node_connectivity;
    // let node_element_connectivity = &mut fem.node_element_connectivity;
    // let node_node_connectivity = &mut fem.node_node_connectivity;
    // let nodal_coordinates = &mut fem.nodal_coordinates;
    // let mut removed_edges = vec![];
    // let mut removed_elements = vec![];
    // let mut merged_nodes = vec![None; nodal_coordinates.len()];

    let mut node_a;
    let mut node_b;
    let mut edge_index = 0;
    let mut foo = 0;
    while edge_index < edges.len() {
        edges.iter().for_each(|[a, b]| {
            if a == b {
                panic!()
            }
        });
        [node_a, node_b] = edges[edge_index];
        if lengths[edge_index] < FOUR_FIFTHS * average_length {
            if foo <= 8900000 {
                println!("\nedge {foo} nodes: {:?}", edges[edge_index]);
                [element_index_1, element_index_2, _, _] = edge_info(
                    node_a,
                    node_b,
                    &fem.element_node_connectivity,
                    &fem.node_element_connectivity,
                );
                let foobar = fem.nodal_coordinates.remove(node_b);
                fem.nodal_coordinates[node_a] = (&fem.nodal_coordinates[node_a] + foobar) * 0.5;
                if element_index_1 < element_index_2 {
                    fem.element_blocks.remove(element_index_1);
                    fem.element_blocks.remove(element_index_2 - 1);
                    fem.element_node_connectivity.remove(element_index_1);
                    fem.element_node_connectivity.remove(element_index_2 - 1);
                } else if element_index_2 < element_index_1 {
                    fem.element_blocks.remove(element_index_2);
                    fem.element_blocks.remove(element_index_1 - 1);
                    fem.element_node_connectivity.remove(element_index_2);
                    fem.element_node_connectivity.remove(element_index_1 - 1);
                }
                fem.element_node_connectivity
                    .iter_mut()
                    .for_each(|connectivity| {
                        connectivity.iter_mut().for_each(|node| {
                            if *node == node_b {
                                *node = node_a
                            } else if *node > node_b {
                                *node -= 1
                            }
                        })
                    });
                fem.node_element_connectivity().unwrap(); // when do this later have to -1 or -2 the elements indices
                edges.remove(edge_index);
                lengths.remove(edge_index);
                edges.iter_mut().for_each(|edge| {
                    edge.iter_mut().for_each(|node| {
                        if *node == node_b {
                            *node = node_a
                        } else if *node > node_b {
                            *node -= 1
                        }
                    });
                    edge.sort()
                });
                let mut seen: std::collections::HashSet<[usize; 2]> =
                    std::collections::HashSet::new();
                let mut index = 0;
                edges.retain(|edge| {
                    let keep = seen.insert(edge.clone());
                    if keep {
                        index += 1;
                    }
                    keep
                });
                lengths.retain(|_| {
                    let keep = index > 0;
                    index -= 1;
                    keep
                });
                edges
                    .iter()
                    .zip(lengths.iter_mut())
                    .for_each(|(&[node_a, node_b], length)| {
                        *length = (&fem.get_nodal_coordinates()[node_a]
                            - &fem.get_nodal_coordinates()[node_b])
                            .norm()
                    });
                fem.write_exo("asdf.exo").unwrap();
                //
                // It is possible to create these again (merging 3 into 1 creates another 3 inside 1)
                // so may have to turn this into a loop of some sort.
                // Also, when do one, changes numbering, and causes issues on the second.
                // So maybe want to start right away with "while there are some..."
                // And possible issues with how far to search...
                //
                degenerate_triangles(fem, edges, lengths, &mut edge_index, node_a);

                //                 let elements = &fem.get_node_element_connectivity()[node_a];
                // println!("elements: {:?}", elements);
                //                 let mut nodes: Vec<usize> = elements
                //                     .iter()
                //                     .flat_map(|&element| {
                //                         fem.get_element_node_connectivity()[element]
                //                     })
                //                     .collect();
                //                 nodes.sort();
                //                 nodes.dedup();
                // println!("nodes: {:?}", nodes);
                //                 let mut more_elements: Vec<usize> = nodes
                //                     .iter()
                //                     .flat_map(|&node| {
                //                         fem.get_node_element_connectivity()[node].clone()
                //                     })
                //                     .collect();
                //                 //
                //                 // is pool of more_elements smaller if use node-to-node connectivity for the first step?
                //                 //
                //                 more_elements.sort();
                //                 more_elements.dedup();
                // println!("more_elements: {:?}", more_elements);
                //                 let (center_nodes, triangless): (Vec<usize>, Vec<Vec<usize>>) = nodes
                //                     .iter()
                //                     .map(|node| {
                //                         (node, more_elements
                //                             .iter()
                //                             .filter(|&&element| {
                //                                 fem.get_element_node_connectivity()
                //                                     [element]
                //                                     .contains(node)
                //                             })
                //                             .cloned()
                //                             .collect::<Vec<usize>>())
                //                     })
                //                     .filter(|(_, triangles)| triangles.len() == NUM_NODES_TRI)
                //                     .unzip();
                // println!("center_nodes: {:?}", center_nodes);
                // println!("triangles: {:?}", triangless);
                //                 // if let Some(center_node) = <[usize; 1]>::try_from(center_nodes).ok() {
                //                 //     if let Some(triangles) = <[Vec<usize>; 1]>::try_from(triangles).ok() {
                //                 for (center_node, triangles) in center_nodes.iter().zip(triangless) {
                // println!("center_nodes: {:?}", center_node);
                // println!("triangles: {:?}", triangles);
                //                         // let center_node = center_node[0];
                //                         // let triangles = <[usize; 3]>::try_from(triangles[0].clone())
                //                         //     .expect("Not exactly three triangles");
                //                         assert!(triangles.is_sorted()); // should be true anyway, take out when done
                //                         let mut triangle_nodes = triangles.iter().flat_map(|&triangle| {
                //                             println!("\ttri: {:?}, conn: {:?}", triangle, fem.get_element_node_connectivity()[triangle]);
                //                             fem.get_element_node_connectivity()[triangle]
                //                         }).collect::<Vec<usize>>();
                //                         triangle_nodes.sort();
                //                         triangle_nodes.dedup();
                //                         triangle_nodes.remove(triangle_nodes.iter().position(|node| node == center_node).expect("Center node not found"));
                // println!("triangle_nodes: {:?}", triangle_nodes);
                //                         let triangle_nodes = triangle_nodes.try_into().expect("Not exactly three nodes");
                //                         fem.element_blocks.remove(triangles[1]);
                //                         fem.element_blocks.remove(triangles[2] - 1);
                //                         fem.element_node_connectivity[triangles[0]] = triangle_nodes; // need to get 3 other nodes and ensure normal correct?
                //                         fem.element_node_connectivity.remove(triangles[1]);
                //                         fem.element_node_connectivity.remove(triangles[2] - 1);
                //                         fem.nodal_coordinates.remove(center_node);
                //                         fem.element_node_connectivity
                //                             .iter_mut()
                //                             .for_each(|connectivity| {
                //                                 connectivity.iter_mut().for_each(|node| {
                //                                     if *node > *center_node {
                //                                         *node -= 1
                //                                     }
                //                                 })
                //                             });
                //                         fem.node_element_connectivity().unwrap(); // when do this later have to -1 or -2 the elements indices
                //                         (0..NUM_NODES_TRI).for_each(|_| {
                //                             let triangle_edge_index = edges.iter().position(|[edge_node_a, edge_node_b]|
                //                                 edge_node_a == center_node || edge_node_b == center_node
                //                             ).expect("Edge not found");
                //                             if triangle_edge_index < edge_index {
                //                                 edge_index -= 1
                //                             }
                //                             edges.remove(triangle_edge_index);
                //                             lengths.remove(triangle_edge_index);
                //                         });
                //                         edges.iter_mut().for_each(|edge| {
                //                             edge.iter_mut().for_each(|node| {
                //                                 if *node > *center_node {
                //                                     *node -= 1
                //                                 }
                //                             });
                //                             edge.sort()
                //                         });
                // }
            }
            foo += 1;
        }
        edge_index += 1;
    }

    // edges
    //     .iter()
    //     .enumerate()
    //     .zip(lengths.iter())
    //     .filter(|&(_, &length)| length < FOUR_FIFTHS * average_length)
    //     .for_each(|((edge_index, &[node_a, node_b]), _)| {
    //         if foo {
    //             [element_index_1, element_index_2, _, _] = edge_info(
    //                 node_a,
    //                 node_b,
    //                 element_node_connectivity,
    //                 node_element_connectivity,
    //             );
    //             // println!("{}", nodal_coordinates[node_a]);
    //             // println!("{}", nodal_coordinates[node_b]);
    //             nodal_coordinates[node_a] = (
    //                 &nodal_coordinates[node_a] + nodal_coordinates[node_b].clone()
    //             ) * 0.5;
    //             // nodal_coordinates[node_b] = nodal_coordinates[node_a].clone();
    //             // merged_nodes[node_b] = Some(node_a);
    //             // removed_edges.push(edge_index);

    //             if element_index_1 < element_index_2 {
    //                 element_blocks.remove(element_index_1);
    //                 element_blocks.remove(element_index_2 - 1);
    //                 element_node_connectivity.remove(element_index_1);
    //                 element_node_connectivity.remove(element_index_2 - 1);
    //             } else {
    //                 element_blocks.remove(element_index_2);
    //                 element_blocks.remove(element_index_1 - 1);
    //                 element_node_connectivity.remove(element_index_2);
    //                 element_node_connectivity.remove(element_index_1 - 1);
    //             }
    //             nodal_coordinates.remove(node_b);

    //             element_node_connectivity.iter_mut().for_each(|connectivity|
    //                 connectivity.iter_mut().for_each(|node|
    //                     if *node == node_b {
    //                         *node = node_a
    //                     } else if *node > node_b {
    //                         *node -= 1
    //                     }
    //                 )
    //             );

    //             // let asdf = node_element_connectivity[node_b].clone();
    //             // node_element_connectivity[node_a].extend(asdf);
    //             // node_element_connectivity.remove(node_b);

    //             // node_element_connectivity.iter_mut().for_each(|connectivity| {
    //             //     if let Some(index) = connectivity.iter().position(|&element| element == element_index_1 + 1) {
    //             //         connectivity.remove(index);
    //             //     }
    //             //     if let Some(index) = connectivity.iter().position(|&element| element == element_index_2 + 1) {
    //             //         connectivity.remove(index);
    //             //     }
    //             // });

    //             let mut asdf = vec![vec![]; nodal_coordinates.len()];
    //             element_node_connectivity
    //                 .iter()
    //                 .enumerate()
    //                 .for_each(|(element, connectivity)| {
    //                     connectivity.iter().for_each(|node| {
    //                         asdf[node]
    //                             .push(element + ELEMENT_NUMBERING_OFFSET)
    //                     })
    //                 });
    //             node_element_connectivity.iter_mut().zip(asdf).for_each(|(entry, yrtne)|
    //                 *entry = yrtne
    //             );

    //             // println!("edge {} between {} and {} is too short", edge_index + 1, node_a, node_b);
    //             // println!("would delete elements {} and {}", element_index_1 + 1, element_index_2 + 1);
    //             // foo = false
    //         }
    // });
    // fem.node_element_connectivity().unwrap(); // needs to happen on the fly to not screw up the edge_info
    fem.node_node_connectivity().unwrap();
    //
    // remove edges from edges AND lengths
    //
}

// fn collapse_edges_old(
//     fem: &mut TriangularFiniteElements,
//     edges: &mut Edges,
//     lengths: &Lengths,
//     average_length: Scalar,
// ) {
//     let mut element_index_1;
//     let mut element_index_2;
//     let element_node_connectivity = &mut fem.element_node_connectivity;
//     let node_element_connectivity = &mut fem.node_element_connectivity;
//     let node_node_connectivity = &mut fem.node_node_connectivity;
//     let nodal_coordinates = &mut fem.nodal_coordinates;

//     let mut edge_index = 0;
//     let mut node_a;
//     let mut node_b;
//     while edge_index < edges.len() {
//         node_a = edges[edge_index][0];
//         node_b = edges[edge_index][1];
// println!("\n{:?}", (node_a, node_b));
//         if (&nodal_coordinates[node_a]
//             - &nodal_coordinates[node_b])
//             .norm()
//             < FOUR_FIFTHS * average_length
//         {
//             if node_a != node_b {
//                 if node_a > node_b {
//                     panic!("TEMPORARY BLOCK TO VERIFY ASSUMPTION")
//                 }
// println!("{:?}", node_element_connectivity[node_a - 1]);
// println!("{:?}", node_element_connectivity[node_b - 1]);
// println!("{:?}", element_node_connectivity[1333 - 1]);
// println!("{:?}", element_node_connectivity[1334 - 1]);
//                 [element_index_1, element_index_2, _, _] = edge_info(
//                     node_a,
//                     node_b,
//                     element_node_connectivity,
//                     node_element_connectivity,
//                 );
// println!("{:?}", [[element_index_1 + 1, element_index_2 + 1]]);
//                 if element_index_1 == element_index_2 {
//                     panic!("TEMPORARY BLOCK TO VERIFY ASSUMPTION")
//                 } else if element_index_1 > element_index_2 {
//                     element_node_connectivity.remove(element_index_1);
//                     node_element_connectivity.iter_mut().for_each(|elements| {
//                         elements.retain(|&element| element != element_index_1 + ELEMENT_NUMBERING_OFFSET);
//                         elements.iter_mut().for_each(|element| {
//                             if *element > element_index_1 + ELEMENT_NUMBERING_OFFSET {
//                                 *element -= 1
//                             }
//                         })
//                     });
//                     element_node_connectivity.remove(element_index_2);
//                     node_element_connectivity.iter_mut().for_each(|elements| {
//                         elements.retain(|&element| element != element_index_2 + ELEMENT_NUMBERING_OFFSET);
//                         elements.iter_mut().for_each(|element| {
//                             if *element > element_index_2 + ELEMENT_NUMBERING_OFFSET {
//                                 *element -= 1
//                             }
//                         })
//                     });
//                 } else if element_index_2 > element_index_1 {
//                     element_node_connectivity.remove(element_index_2);
//                     node_element_connectivity.iter_mut().for_each(|elements| {
//                         elements.retain(|&element| element != element_index_2 + ELEMENT_NUMBERING_OFFSET);
//                         elements.iter_mut().for_each(|element| {
//                             if *element > element_index_2 + ELEMENT_NUMBERING_OFFSET {
//                                 *element -= 1
//                             }
//                         })
//                     });
//                     element_node_connectivity.remove(element_index_1);
//                     node_element_connectivity.iter_mut().for_each(|elements| {
//                         elements.retain(|&element| element != element_index_1 + ELEMENT_NUMBERING_OFFSET);
//                         elements.iter_mut().for_each(|element| {
//                             if *element > element_index_1 + ELEMENT_NUMBERING_OFFSET {
//                                 *element -= 1
//                             }
//                         })
//                     });
//                 }
//                 let mut foo = node_element_connectivity[node_b].clone();
//                 node_element_connectivity[node_a].append(&mut foo);
//                 node_element_connectivity[node_a].sort();
//                 node_element_connectivity[node_a].dedup();
//                 node_element_connectivity.remove(node_b);
//                 element_node_connectivity.iter_mut().for_each(|nodes| {
//                     nodes.iter_mut().for_each(|node| {
//                         if node == &node_b {
//                             *node = node_a
//                         } else if *node > node_b {
//                             *node -= 1
//                         }
//                     })
//                 });
//                 edges.iter_mut().for_each(|nodes| {
//                     nodes.iter_mut().for_each(|node| {
//                         if node == &node_b {
//                             *node = node_a
//                         } else if *node > node_b {
//                             *node -= 1
//                         }
//                     });
//                     nodes.sort() // can sort when change from b to a only
//                 });
//                 nodal_coordinates[node_a] = (
//                     nodal_coordinates[node_a].clone() +
//                     &nodal_coordinates[node_b]
//                 ) * 0.5;
//                 nodal_coordinates.remove(node_b);
//             }
//         }
//         edge_index += 1
//     }
//     //
//     // Need to adjust the node-to-node connectivity, keep sorted.
//     //
//     // Could also remove edges that are collapsed (a == b) or duplicated?
//     //
// }

fn flip_edges(fem: &mut TriangularFiniteElements, edges: &mut Edges) {
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
    edges.iter_mut().for_each(|[node_a, node_b]| {
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
        }
    });
}

fn edge_info(
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
        .expect(format!("Nodes {node_a} and {node_b} do not share exactly two elements").as_str());
    let node_c = *element_node_connectivity[element_index_1]
        .iter()
        .find(|node_1| !element_node_connectivity[element_index_2].contains(node_1))
        .expect(
            format!("Elements {element_index_1} and {element_index_2} are overlapping.").as_str(),
        );
    let node_d = *element_node_connectivity[element_index_2]
        .iter()
        .find(|node_2| !element_node_connectivity[element_index_1].contains(node_2))
        .expect(
            format!("Elements {element_index_1} and {element_index_2} are overlapping.").as_str(),
        );
    [element_index_1, element_index_2, node_c, node_d]
}

fn degenerate_triangles(
    fem: &mut TriangularFiniteElements,
    edges: &mut Edges,
    lengths: &mut Lengths,
    edge_index: &mut usize,
    node_a: usize,
) {
    let mut complete = false;
    while !complete {
        complete = degenerate_triangle(fem, edges, lengths, edge_index, node_a)
    }
}

fn degenerate_triangle(
    fem: &mut TriangularFiniteElements,
    edges: &mut Edges,
    lengths: &mut Lengths,
    edge_index: &mut usize,
    node_a: usize,
) -> bool {
    //
    // How large to create the stencil of nodes to search from node_a?
    //
    let elements = &fem.get_node_element_connectivity()[node_a];
    println!("elements: {:?}", elements);
    let mut nodes: Vec<usize> = elements
        .iter()
        .flat_map(|&element| fem.get_element_node_connectivity()[element])
        .collect();
    nodes.sort();
    nodes.dedup();
    println!("nodes: {:?}", nodes);
    let mut more_elements: Vec<usize> = nodes
        .iter()
        .flat_map(|&node| fem.get_node_element_connectivity()[node].clone())
        .collect();
    //
    // is pool of more_elements smaller if use node-to-node connectivity for the first step?
    //
    more_elements.sort();
    more_elements.dedup();
    println!("more_elements: {:?}", more_elements);
    let (mut center_nodes, mut triangless): (Vec<usize>, Vec<Vec<usize>>) = nodes
        .iter()
        .map(|node| {
            (
                node,
                more_elements
                    .iter()
                    .filter(|&&element| fem.get_element_node_connectivity()[element].contains(node))
                    .cloned()
                    .collect::<Vec<usize>>(),
            )
        })
        .filter(|(_, triangles)| triangles.len() == NUM_NODES_TRI)
        .unzip();
    let mut complete = true;
    if let Some(center_node) = center_nodes.pop() {
        if let Some(triangles) = triangless.pop() {
            complete = false;
            println!("center_nodes: {:?}", center_node);
            println!("triangles: {:?}", triangles);
            // let center_node = center_node[0];
            // let triangles = <[usize; 3]>::try_from(triangles[0].clone())
            //     .expect("Not exactly three triangles");
            assert!(triangles.is_sorted()); // should be true anyway, take out when done
            let mut triangle_nodes = triangles
                .iter()
                .flat_map(|&triangle| {
                    println!(
                        "\ttri: {:?}, conn: {:?}",
                        triangle,
                        fem.get_element_node_connectivity()[triangle]
                    );
                    fem.get_element_node_connectivity()[triangle]
                })
                .collect::<Vec<usize>>();
            triangle_nodes.sort();
            triangle_nodes.dedup();
            triangle_nodes.remove(
                triangle_nodes
                    .iter()
                    .position(|node| node == &center_node)
                    .expect("Center node not found"),
            );
            println!("triangle_nodes: {:?}", triangle_nodes);
            let triangle_nodes = triangle_nodes.try_into().expect("Not exactly three nodes");
            fem.element_blocks.remove(triangles[1]);
            fem.element_blocks.remove(triangles[2] - 1);
            fem.element_node_connectivity[triangles[0]] = triangle_nodes; // need to get 3 other nodes and ensure normal correct?
            fem.element_node_connectivity.remove(triangles[1]);
            fem.element_node_connectivity.remove(triangles[2] - 1);
            fem.nodal_coordinates.remove(center_node);
            fem.element_node_connectivity
                .iter_mut()
                .for_each(|connectivity| {
                    connectivity.iter_mut().for_each(|node| {
                        if *node > center_node {
                            *node -= 1
                        }
                    })
                });
            fem.node_element_connectivity().unwrap(); // when do this later have to -1 or -2 the elements indices
            (0..NUM_NODES_TRI).for_each(|_| {
                let triangle_edge_index = edges
                    .iter()
                    .position(|[edge_node_a, edge_node_b]| {
                        edge_node_a == &center_node || edge_node_b == &center_node
                    })
                    .expect("Edge not found");
                if &triangle_edge_index < edge_index {
                    *edge_index -= 1
                }
                edges.remove(triangle_edge_index);
                lengths.remove(triangle_edge_index);
            });
            edges.iter_mut().for_each(|edge| {
                edge.iter_mut().for_each(|node| {
                    if *node > center_node {
                        *node -= 1
                    }
                });
                edge.sort()
            });
        }
    }
    complete
}
