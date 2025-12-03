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
// const FOUR_FIFTHS: Scalar = 4.0 / 5.0;
const J_EQUILATERAL: Scalar = 0.8660254037844387;
const REGULAR_DEGREE: i8 = 6;

/// The number of nodes in a triangular finite element.
pub const TRI: usize = 3;

const NUM_NODES_FACE: usize = 1;

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
        split_edges(fem, &mut edges, &mut lengths, average_length);
        // collapse_edges(fem, &mut edges, &lengths, average_length);
        flip_edges(fem, &mut edges);
        fem.nodal_influencers();
        fem.smooth(smoothing_method).unwrap();
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
