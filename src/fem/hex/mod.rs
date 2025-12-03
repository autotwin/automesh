#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Connectivity, Coordinates, FiniteElementMethods, FiniteElementSpecifics, FiniteElements,
    Metrics, Size, Smoothing, Tessellation, Vector,
};
use conspire::math::{Tensor, TensorArray, TensorVec};
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    path::Path,
};

/// The number of nodes in a hexahedral finite element.
pub const HEX: usize = 8;

/// The number of nodes per face of a hexahedral finite element.
const NUM_NODES_FACE: usize = 4;

/// The element-to-node connectivity for hexahedral finite elements.
pub type HexConnectivity = Connectivity<HEX>;

/// The hexahedral finite elements type.
pub type HexahedralFiniteElements = FiniteElements<HEX>;

impl FiniteElementSpecifics<NUM_NODES_FACE> for HexahedralFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 3, 4],
            1 => vec![0, 2, 5],
            2 => vec![1, 3, 6],
            3 => vec![0, 2, 7],
            4 => vec![0, 5, 7],
            5 => vec![1, 4, 6],
            6 => vec![2, 5, 7],
            7 => vec![3, 4, 6],
            _ => panic!(),
        }
    }
    fn exterior_faces(&self) -> Connectivity<NUM_NODES_FACE> {
        let mut face_counts = HashMap::new();
        let face_to_original: Vec<_> = self
            .get_element_node_connectivity()
            .iter()
            .flat_map(
                |&[
                    node_0,
                    node_1,
                    node_2,
                    node_3,
                    node_4,
                    node_5,
                    node_6,
                    node_7,
                ]| {
                    [
                        [node_0, node_1, node_5, node_4],
                        [node_1, node_2, node_6, node_5],
                        [node_2, node_3, node_7, node_6],
                        [node_3, node_0, node_4, node_7],
                        [node_3, node_2, node_1, node_0],
                        [node_4, node_5, node_6, node_7],
                    ]
                },
            )
            .map(|face| {
                let mut canonical = face;
                canonical.sort_unstable();
                *face_counts.entry(canonical).or_default() += 1;
                (canonical, face)
            })
            .collect();
        face_to_original
            .into_iter()
            .filter_map(|(canonical, original)| {
                if face_counts.get(&canonical) == Some(&1) {
                    Some(original)
                } else {
                    None
                }
            })
            .collect()
    }
    fn exterior_faces_interior_points(&self, grid_length: usize) -> Coordinates {
        if grid_length == 0 {
            panic!("Grid length must be greater than zero");
        } else if grid_length == 1 {
            self.exterior_faces_centroids()
        } else {
            let nodal_coordinates = self.get_nodal_coordinates();
            let mut points = Coordinates::zero(0);
            let mut shape_functions = [0.0; NUM_NODES_FACE];
            let step = 2.0 / (grid_length as f64);
            let mut xi = 0.0;
            let mut eta = 0.0;
            self.exterior_faces().iter().for_each(|nodes| {
                (0..grid_length).for_each(|i| {
                    xi = -1.0 + (i as f64 + 0.5) * step;
                    (0..grid_length).for_each(|j| {
                        eta = -1.0 + (j as f64 + 0.5) * step;
                        shape_functions = [
                            0.25 * (1.0 - xi) * (1.0 - eta),
                            0.25 * (1.0 + xi) * (1.0 - eta),
                            0.25 * (1.0 + xi) * (1.0 + eta),
                            0.25 * (1.0 - xi) * (1.0 + eta),
                        ];
                        points.push(
                            nodes
                                .iter()
                                .zip(shape_functions.iter())
                                .map(|(&node, shape_function)| {
                                    &nodal_coordinates[node] * shape_function
                                })
                                .sum(),
                        );
                    })
                })
            });
            points
        }
    }
    fn faces(&self) -> Connectivity<NUM_NODES_FACE> {
        let faces: Connectivity<NUM_NODES_FACE> = self
            .get_element_node_connectivity()
            .iter()
            .flat_map(
                |&[
                    node_0,
                    node_1,
                    node_2,
                    node_3,
                    node_4,
                    node_5,
                    node_6,
                    node_7,
                ]| {
                    [
                        [node_0, node_1, node_5, node_4],
                        [node_1, node_2, node_6, node_5],
                        [node_2, node_3, node_7, node_6],
                        [node_3, node_0, node_4, node_7],
                        [node_3, node_2, node_1, node_0],
                        [node_4, node_5, node_6, node_7],
                    ]
                },
            )
            .collect();
        let mut canonical_face = [0; NUM_NODES_FACE];
        let mut unique_faces = HashSet::new();
        let mut deduplicated_faces = Vec::new();
        faces.iter().for_each(|&face| {
            canonical_face = face;
            canonical_face.sort_unstable();
            if unique_faces.insert(canonical_face) {
                deduplicated_faces.push(face);
            }
        });
        deduplicated_faces
    }
    fn interior_points(&self, grid_length: usize) -> Coordinates {
        if grid_length == 0 {
            panic!("Grid length must be greater than zero");
        } else if grid_length == 1 {
            self.centroids()
        } else {
            let nodal_coordinates = self.get_nodal_coordinates();
            let mut points = Coordinates::zero(0);
            let mut shape_functions = [0.0; HEX];
            let step = 2.0 / (grid_length as f64);
            let mut xi = 0.0;
            let mut eta = 0.0;
            let mut zeta = 0.0;
            self.get_element_node_connectivity()
                .iter()
                .for_each(|nodes| {
                    (0..grid_length).for_each(|i| {
                        xi = -1.0 + (i as f64 + 0.5) * step;
                        (0..grid_length).for_each(|j| {
                            eta = -1.0 + (j as f64 + 0.5) * step;
                            (0..grid_length).for_each(|k| {
                                zeta = -1.0 + (k as f64 + 0.5) * step;
                                shape_functions = [
                                    0.125 * (1.0 - xi) * (1.0 - eta) * (1.0 - zeta),
                                    0.125 * (1.0 + xi) * (1.0 - eta) * (1.0 - zeta),
                                    0.125 * (1.0 + xi) * (1.0 + eta) * (1.0 - zeta),
                                    0.125 * (1.0 - xi) * (1.0 + eta) * (1.0 - zeta),
                                    0.125 * (1.0 - xi) * (1.0 - eta) * (1.0 + zeta),
                                    0.125 * (1.0 + xi) * (1.0 - eta) * (1.0 + zeta),
                                    0.125 * (1.0 + xi) * (1.0 + eta) * (1.0 + zeta),
                                    0.125 * (1.0 - xi) * (1.0 + eta) * (1.0 + zeta),
                                ];
                                points.push(
                                    nodes
                                        .iter()
                                        .zip(shape_functions.iter())
                                        .map(|(&node, shape_function)| {
                                            &nodal_coordinates[node] * shape_function
                                        })
                                        .sum(),
                                );
                            })
                        })
                    })
                });
            points
        }
    }
    fn maximum_edge_ratios(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l1 = 0.0;
        let mut l2 = 0.0;
        let mut l3 = 0.0;
        self.get_element_node_connectivity()
            .iter()
            .map(
                |&[
                    node_0,
                    node_1,
                    node_2,
                    node_3,
                    node_4,
                    node_5,
                    node_6,
                    node_7,
                ]| {
                    l1 = (&nodal_coordinates[node_1] - &nodal_coordinates[node_0]
                        + &nodal_coordinates[node_2]
                        - &nodal_coordinates[node_3]
                        + &nodal_coordinates[node_5]
                        - &nodal_coordinates[node_4]
                        + &nodal_coordinates[node_6]
                        - &nodal_coordinates[node_7])
                        .norm();
                    l2 = (&nodal_coordinates[node_3] - &nodal_coordinates[node_0]
                        + &nodal_coordinates[node_2]
                        - &nodal_coordinates[node_1]
                        + &nodal_coordinates[node_7]
                        - &nodal_coordinates[node_4]
                        + &nodal_coordinates[node_6]
                        - &nodal_coordinates[node_5])
                        .norm();
                    l3 = (&nodal_coordinates[node_4] - &nodal_coordinates[node_0]
                        + &nodal_coordinates[node_5]
                        - &nodal_coordinates[node_1]
                        + &nodal_coordinates[node_6]
                        - &nodal_coordinates[node_2]
                        + &nodal_coordinates[node_7]
                        - &nodal_coordinates[node_3])
                        .norm();
                    [l1, l2, l3].into_iter().reduce(f64::max).unwrap()
                        / [l1, l2, l3].into_iter().reduce(f64::min).unwrap()
                },
            )
            .collect()
    }
    fn maximum_skews(&self) -> Metrics {
        let mut x1 = Vector::zero();
        let mut x2 = Vector::zero();
        let mut x3 = Vector::zero();
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                (x1, x2, x3) = self.principal_axes(connectivity);
                x1.normalize();
                x2.normalize();
                x3.normalize();
                [(&x1 * &x2).abs(), (&x1 * &x3).abs(), (&x2 * &x3).abs()]
                    .into_iter()
                    .reduce(f64::max)
                    .unwrap()
            })
            .collect()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut u = Vector::zero();
        let mut v = Vector::zero();
        let mut w = Vector::zero();
        let mut n = Vector::zero();
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                connectivity
                    .iter()
                    .enumerate()
                    .map(|(index, &node)| {
                        match index {
                            0 => {
                                u = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[4]] - &nodal_coordinates[node];
                            }
                            1 => {
                                u = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[5]] - &nodal_coordinates[node];
                            }
                            2 => {
                                u = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[6]] - &nodal_coordinates[node];
                            }
                            3 => {
                                u = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[7]] - &nodal_coordinates[node];
                            }
                            4 => {
                                u = &nodal_coordinates[connectivity[7]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[5]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[node];
                            }
                            5 => {
                                u = &nodal_coordinates[connectivity[4]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[6]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[node];
                            }
                            6 => {
                                u = &nodal_coordinates[connectivity[5]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[7]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[node];
                            }
                            7 => {
                                u = &nodal_coordinates[connectivity[6]] - &nodal_coordinates[node];
                                v = &nodal_coordinates[connectivity[4]] - &nodal_coordinates[node];
                                w = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[node];
                            }
                            _ => panic!(),
                        }
                        n = u.cross(&v);
                        (&n * &w) / u.norm() / v.norm() / w.norm()
                    })
                    .collect::<Vec<f64>>()
                    .into_iter()
                    .reduce(f64::min)
                    .unwrap()
            })
            .collect()
    }
    fn remesh(&mut self, _iterations: usize, _smoothing_method: &Smoothing, _size: Size) {
        todo!()
    }
    fn write_metrics(&self, file_path: &str) -> Result<(), ErrorIO> {
        let maximum_edge_ratios = self.maximum_edge_ratios();
        let minimum_scaled_jacobians = self.minimum_scaled_jacobians();
        let maximum_skews = self.maximum_skews();
        let volumes = self.volumes();
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
                        minimum_scaled_jacobians
                            .iter()
                            .zip(maximum_skews.iter().zip(volumes.iter())),
                    )
                    .try_for_each(
                        |(
                            maximum_edge_ratio,
                            (minimum_scaled_jacobian, (maximum_skew, volume)),
                        )| {
                            file.write_all(
                                format!(
                                    "{maximum_edge_ratio:>10.6e},{minimum_scaled_jacobian:>10.6e},{maximum_skew:>10.6e},{volume:>10.6e}\n",
                                )
                                .as_bytes(),
                            )
                        },
                    )?;
                file.flush()?
            }
            Some("npy") => {
                let n_columns = 4; // total number of hexahedral metrics
                let idx_ratios = 0; // maximum edge ratios
                let idx_jacobians = 1; // minimum scaled jacobians
                let idx_skews = 2; // maximum skews
                let idx_volumes = 3; // areas
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
                metrics_set.slice_mut(s![.., idx_volumes]).assign(&volumes);
                metrics_set.write_npy(file).unwrap();
            }
            _ => panic!(
                "Unsupported file extension for metrics output: {:?}.  Please use 'csv' or 'npy'.",
                input_extension
            ),
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mWriting hexahedral metrics to file\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(())
    }
}

impl HexahedralFiniteElements {
    fn principal_axes(&self, connectivity: &[usize; HEX]) -> (Vector, Vector, Vector) {
        let nodal_coordinates = self.get_nodal_coordinates();
        let x1 = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]]
            + &nodal_coordinates[connectivity[2]]
            - &nodal_coordinates[connectivity[3]]
            + &nodal_coordinates[connectivity[5]]
            - &nodal_coordinates[connectivity[4]]
            + &nodal_coordinates[connectivity[6]]
            - &nodal_coordinates[connectivity[7]];
        let x2 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[0]]
            + &nodal_coordinates[connectivity[2]]
            - &nodal_coordinates[connectivity[1]]
            + &nodal_coordinates[connectivity[7]]
            - &nodal_coordinates[connectivity[4]]
            + &nodal_coordinates[connectivity[6]]
            - &nodal_coordinates[connectivity[5]];
        let x3 = &nodal_coordinates[connectivity[4]] - &nodal_coordinates[connectivity[0]]
            + &nodal_coordinates[connectivity[5]]
            - &nodal_coordinates[connectivity[1]]
            + &nodal_coordinates[connectivity[6]]
            - &nodal_coordinates[connectivity[2]]
            + &nodal_coordinates[connectivity[7]]
            - &nodal_coordinates[connectivity[3]];
        (x1, x2, x3)
    }
    fn volumes(&self) -> Metrics {
        let mut x1 = Vector::zero();
        let mut x2 = Vector::zero();
        let mut x3 = Vector::zero();
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                (x1, x2, x3) = self.principal_axes(connectivity);
                &x2.cross(&x3) * &x1 / 64.0
            })
            .collect()
    }
}

impl From<Tessellation> for HexahedralFiniteElements {
    fn from(_tessellation: Tessellation) -> Self {
        unimplemented!()
    }
}
