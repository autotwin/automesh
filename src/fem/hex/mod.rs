#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Connectivity, Coordinates, FiniteElementMethods, FiniteElementSpecifics, FiniteElements,
    Metrics, Size, Smoothing, Tessellation,
};
use conspire::{
    geometry::mesh::{Connectivity as MeshConnectivity, Mesh, Verdict},
    math::{Tensor, TensorVec},
};
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

const O: usize = 3;
const NUM_NODES_FACE: usize = 4;

/// The element-to-node connectivity for hexahedral finite elements.
pub type HexConnectivity = Connectivity<HEX>;

/// The hexahedral finite elements type.d
pub type HexahedralFiniteElements = FiniteElements<HEX>;

impl FiniteElementSpecifics<NUM_NODES_FACE, O> for HexahedralFiniteElements {
    fn connected_nodes(node: &usize) -> [usize; O] {
        match node {
            0 => [1, 3, 4],
            1 => [0, 2, 5],
            2 => [1, 3, 6],
            3 => [0, 2, 7],
            4 => [0, 5, 7],
            5 => [1, 4, 6],
            6 => [2, 5, 7],
            7 => [3, 4, 6],
            _ => panic!(),
        }
    }
    fn connected_nodes_face(node: &usize) -> [usize; 2] {
        match node {
            0 => [1, 3],
            1 => [0, 2],
            2 => [1, 3],
            3 => [0, 2],
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
        self.as_mesh()
            .maximum_edge_ratios()
            .into_iter()
            .flatten()
            .collect()
    }
    fn maximum_skews(&self) -> Metrics {
        self.as_mesh().maximum_skews().into_iter().flatten().collect()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        self.as_mesh()
            .minimum_scaled_jacobians()
            .into_iter()
            .flatten()
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
    fn as_mesh(&self) -> Mesh<3> {
        let connectivity = self
            .get_element_node_connectivity()
            .clone()
            .into_iter()
            .collect::<Vec<[usize; HEX]>>();
        let coordinates = self
            .get_nodal_coordinates()
            .iter()
            .map(|coordinate| [coordinate[0], coordinate[1], coordinate[2]])
            .collect::<Vec<[f64; 3]>>();
        Mesh::<3>::from((
            vec![MeshConnectivity::Hexahedral(connectivity.into())],
            coordinates.into(),
        ))
    }
    fn volumes(&self) -> Metrics {
        self.as_mesh().volumes().into_iter().flatten().collect()
    }
}

impl From<Tessellation> for HexahedralFiniteElements {
    fn from(_tessellation: Tessellation) -> Self {
        unimplemented!()
    }
}
