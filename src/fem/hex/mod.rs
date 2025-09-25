#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Connectivity, Coordinates, FiniteElementMethods, FiniteElementSpecifics, FiniteElements,
    Metrics, NODE_NUMBERING_OFFSET, Smoothing, Tessellation, Vector,
};
use conspire::math::{Tensor, TensorArray};
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    path::Path,
};

/// The number of nodes in a hexahedral finite element.
pub const HEX: usize = 8;

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
        let mut faces: Connectivity<NUM_NODES_FACE> = self
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
        faces.iter_mut().for_each(|face| face.sort());
        let mut face_counts = HashMap::new();
        faces
            .iter()
            .for_each(|&face| *face_counts.entry(face).or_insert(0) += 1);
        let exterior_faces: Connectivity<NUM_NODES_FACE> = face_counts
            .into_iter()
            .filter_map(|(face, count)| if count == 1 { Some(face) } else { None })
            .collect();
        exterior_faces
    }
    fn exterior_faces_integration_points(&self) -> Coordinates {
        let sqrt3_inv = 1.0 / 3.0_f64.sqrt();
        let shape_functions_integration_points: [[f64; NUM_NODES_FACE]; NUM_NODES_FACE] = [
            [-sqrt3_inv, -sqrt3_inv],
            [-sqrt3_inv, sqrt3_inv],
            [sqrt3_inv, -sqrt3_inv],
            [sqrt3_inv, sqrt3_inv],
        ]
        .into_iter()
        .map(|[xi, eta]| {
            [
                0.25 * (1.0 - xi) * (1.0 - eta),
                0.25 * (1.0 + xi) * (1.0 - eta),
                0.25 * (1.0 + xi) * (1.0 + eta),
                0.25 * (1.0 - xi) * (1.0 + eta),
            ]
        })
        .collect::<Vec<[f64; NUM_NODES_FACE]>>()
        .try_into()
        .unwrap();
        let nodal_coordinates = self.get_nodal_coordinates();
        self.exterior_faces()
            .iter()
            .flat_map(|nodes| {
                shape_functions_integration_points
                    .iter()
                    .map(|shape_functions| {
                        nodes
                            .iter()
                            .zip(shape_functions.iter())
                            .map(|(&node, shape_function)| {
                                &nodal_coordinates[node - NODE_NUMBERING_OFFSET] * shape_function
                            })
                            .sum()
                    })
            })
            .collect()
    }
    fn faces(&self) -> Connectivity<NUM_NODES_FACE> {
        let mut faces: Connectivity<NUM_NODES_FACE> = self
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
        faces.iter_mut().for_each(|face| face.sort());
        faces.sort();
        faces.dedup();
        faces
    }
    fn integration_points(&self) -> Coordinates {
        let one_eighth = 1.0 / 8.0;
        let sqrt3_inv = 1.0 / 3.0_f64.sqrt();
        let shape_functions_integration_points: [[f64; HEX]; HEX] = [
            [-sqrt3_inv, -sqrt3_inv, -sqrt3_inv],
            [-sqrt3_inv, -sqrt3_inv, sqrt3_inv],
            [-sqrt3_inv, sqrt3_inv, -sqrt3_inv],
            [-sqrt3_inv, sqrt3_inv, sqrt3_inv],
            [sqrt3_inv, -sqrt3_inv, -sqrt3_inv],
            [sqrt3_inv, -sqrt3_inv, sqrt3_inv],
            [sqrt3_inv, sqrt3_inv, -sqrt3_inv],
            [sqrt3_inv, sqrt3_inv, sqrt3_inv],
        ]
        .into_iter()
        .map(|[xi, eta, zeta]| {
            [
                one_eighth * (1.0 - xi) * (1.0 - eta) * (1.0 - zeta),
                one_eighth * (1.0 + xi) * (1.0 - eta) * (1.0 - zeta),
                one_eighth * (1.0 + xi) * (1.0 + eta) * (1.0 - zeta),
                one_eighth * (1.0 - xi) * (1.0 + eta) * (1.0 - zeta),
                one_eighth * (1.0 - xi) * (1.0 - eta) * (1.0 + zeta),
                one_eighth * (1.0 + xi) * (1.0 - eta) * (1.0 + zeta),
                one_eighth * (1.0 + xi) * (1.0 + eta) * (1.0 + zeta),
                one_eighth * (1.0 - xi) * (1.0 + eta) * (1.0 + zeta),
            ]
        })
        .collect::<Vec<[f64; HEX]>>()
        .try_into()
        .unwrap();
        let nodal_coordinates = self.get_nodal_coordinates();
        self.get_element_node_connectivity()
            .iter()
            .flat_map(|nodes| {
                shape_functions_integration_points
                    .iter()
                    .map(|shape_functions| {
                        nodes
                            .iter()
                            .zip(shape_functions.iter())
                            .map(|(&node, shape_function)| {
                                &nodal_coordinates[node - NODE_NUMBERING_OFFSET] * shape_function
                            })
                            .sum()
                    })
            })
            .collect()
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
                    l1 = (&nodal_coordinates[node_1 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_0 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_2 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_3 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_5 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_4 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_6 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_7 - NODE_NUMBERING_OFFSET])
                        .norm();
                    l2 = (&nodal_coordinates[node_3 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_0 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_2 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_1 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_7 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_4 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_6 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_5 - NODE_NUMBERING_OFFSET])
                        .norm();
                    l3 = (&nodal_coordinates[node_4 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_0 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_5 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_1 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_6 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_2 - NODE_NUMBERING_OFFSET]
                        + &nodal_coordinates[node_7 - NODE_NUMBERING_OFFSET]
                        - &nodal_coordinates[node_3 - NODE_NUMBERING_OFFSET])
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
                    .map(|(index, node)| {
                        match index {
                            0 => {
                                u = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            1 => {
                                u = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            2 => {
                                u = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            3 => {
                                u = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            4 => {
                                u = &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            5 => {
                                u = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            6 => {
                                u = &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            }
                            7 => {
                                u = &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                v = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                                w = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                                    - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
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
    fn remesh(&mut self, _iterations: usize, _smoothing_method: &Smoothing) {
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
            _ => panic!("print error message with input and extension"),
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mWriting hexahedron metrics to file\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(())
    }
}

impl HexahedralFiniteElements {
    fn principal_axes(&self, connectivity: &[usize; HEX]) -> (Vector, Vector, Vector) {
        let nodal_coordinates = self.get_nodal_coordinates();
        let x1 = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET];
        let x2 = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET];
        let x3 = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
            + &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
            - &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET];
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
