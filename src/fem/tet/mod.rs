#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Connectivity, Coordinates, FiniteElementMethods, FiniteElementSpecifics, FiniteElements, HEX,
    HexahedralFiniteElements, Metrics, Size, Smoothing, Tessellation, Vector,
};
use conspire::math::{Tensor, TensorRank1};
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    f64::consts::PI,
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    iter::repeat_n,
    path::Path,
};

const TOLERANCE: f64 = 1e-9;

/// The number of nodes in a tetrahedral finite element.
pub const TET: usize = 4;

/// The number of edges in a tetrahedral finite element.
const NUM_EDGES: usize = 6;

/// The number of nodes per face of a tetrahedral finite element.
const NUM_NODES_FACE: usize = 3;

/// The tetrahedral finite elements type.
pub type TetrahedralFiniteElements = FiniteElements<TET>;

/// The number of tetrahedral elements created from a single hexahedral element.
pub const NUM_TETS_PER_HEX: usize = 6;

impl FiniteElementSpecifics<NUM_NODES_FACE> for TetrahedralFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 2, 3],
            1 => vec![0, 2, 3],
            2 => vec![0, 1, 3],
            3 => vec![0, 1, 2],
            _ => panic!(),
        }
    }
    fn exterior_faces(&self) -> Connectivity<NUM_NODES_FACE> {
        todo!()
    }
    fn exterior_faces_interior_points(&self, _grid_length: usize) -> Coordinates {
        todo!()
    }
    fn faces(&self) -> Connectivity<NUM_NODES_FACE> {
        todo!()
    }
    fn interior_points(&self, _grid_length: usize) -> Coordinates {
        todo!()
    }
    fn maximum_edge_ratios(&self) -> Metrics {
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                let (min_length, max_length) = self.edge_vectors(connectivity).into_iter().fold(
                    (f64::INFINITY, f64::NEG_INFINITY),
                    |(mut minimum, mut maximum), edge_vector| {
                        let length = edge_vector.norm();
                        minimum = minimum.min(length);
                        maximum = maximum.max(length);
                        (minimum, maximum)
                    },
                );
                max_length / min_length
            })
            .collect()
    }
    fn maximum_skews(&self) -> Metrics {
        self.get_element_node_connectivity()
            .iter()
            .map(|&[n0, n1, n2, n3]| {
                // A tetrahedron has four faces, so calculate the skew for each and
                // then take the maximum
                [
                    self.face_maximum_skew(n0, n1, n2),
                    self.face_maximum_skew(n0, n1, n3),
                    self.face_maximum_skew(n0, n2, n3),
                    self.face_maximum_skew(n1, n2, n3),
                ]
                .into_iter()
                .reduce(f64::max)
                .unwrap()
            })
            .collect()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                // The element Jacobian j is 6.0 times the signed element volume
                let jac = self.signed_element_volume(connectivity) * 6.0;

                // Get all six edge lengths
                let [len_0, len_1, len_2, len_3, len_4, len_5]: [_; NUM_EDGES] = self
                    .edge_vectors(connectivity)
                    .into_iter()
                    .map(|v| v.norm())
                    .collect::<TensorRank1<_, 1>>()
                    .into();

                // Compute the four nodal Jacobians
                let lambda_0 = len_0 * len_2 * len_3;
                let lambda_1 = len_0 * len_1 * len_4;
                let lambda_2 = len_1 * len_2 * len_5;
                let lambda_3 = len_3 * len_4 * len_5;

                // Find the maximum of the nodal Jacobians (including the element Jacobian)
                let lambda_max = [jac, lambda_0, lambda_1, lambda_2, lambda_3]
                    .into_iter()
                    .reduce(f64::max)
                    .unwrap();

                // Calculate the final quality metric
                if lambda_max == 0.0 {
                    0.0 // Avoid division by zero for collapsed elements
                } else {
                    jac * 2.0_f64.sqrt() / lambda_max
                }
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
                let n_columns = 4; // total number of tetrahedral metrics
                let idx_ratios = 0; // maximum edge ratios
                let idx_jacobians = 1; // minimum scaled jacobians
                let idx_skews = 2; // maximum skews
                let idx_volumes = 3; // areas
                let mut metrics_set =
                    Array2::from_elem((minimum_scaled_jacobians.len(), n_columns), 0.0);
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
            "             \x1b[1;93mWriting tetrahedral metrics to file\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(())
    }
}

impl TetrahedralFiniteElements {
    fn edge_vectors(
        &self,
        &[node_0, node_1, node_2, node_3]: &[usize; TET],
    ) -> [Vector; NUM_EDGES] {
        let nodal_coordinates = self.get_nodal_coordinates();

        // Base edges (in a cycle 0 -> 1 -> 2 -> 0])
        let e0 = &nodal_coordinates[node_1] - &nodal_coordinates[node_0];
        let e1 = &nodal_coordinates[node_2] - &nodal_coordinates[node_1];
        let e2 = &nodal_coordinates[node_0] - &nodal_coordinates[node_2];

        // Edges connecting the apex (node 3)
        let e3 = &nodal_coordinates[node_3] - &nodal_coordinates[node_0];
        let e4 = &nodal_coordinates[node_3] - &nodal_coordinates[node_1];
        let e5 = &nodal_coordinates[node_3] - &nodal_coordinates[node_2];

        // Return all six edge vectors
        [e0, e1, e2, e3, e4, e5]
    }

    // Helper function to calculate the signed volume of a single tetrahedron.
    fn signed_element_volume(&self, &[node_0, node_1, node_2, node_3]: &[usize; TET]) -> f64 {
        let nodal_coordinates = self.get_nodal_coordinates();
        let v0 = &nodal_coordinates[node_0];
        let v1 = &nodal_coordinates[node_1];
        let v2 = &nodal_coordinates[node_2];
        let v3 = &nodal_coordinates[node_3];
        ((v1 - v0).cross(&(v2 - v0)) * (v3 - v0)) / 6.0
    }

    // Calculates the volumes for all tetrahedral elements in the mesh.
    // This is the public method that iterates over all elements.
    pub fn volumes(&self) -> Metrics {
        self.element_node_connectivity
            .iter()
            .map(|connectivity| {
                // Calls the private helper for each element.
                self.signed_element_volume(connectivity)
            })
            .collect()
    }

    /// Calculates the minimum angle of a triangular face defined by three node indices.
    fn face_minimum_angle(&self, n0_idx: usize, n1_idx: usize, n2_idx: usize) -> f64 {
        let nodal_coordinates = self.get_nodal_coordinates();
        let v0 = &nodal_coordinates[n0_idx];
        let v1 = &nodal_coordinates[n1_idx];
        let v2 = &nodal_coordinates[n2_idx];

        let l0 = (v2 - v1).normalized();
        let l1 = (v0 - v2).normalized();
        let l2 = (v1 - v0).normalized();

        [
            (-(&l0 * &l1)).acos(),
            (-(&l1 * &l2)).acos(),
            (-(&l2 * &l0)).acos(),
        ]
        .into_iter()
        .reduce(f64::min)
        .unwrap()
    }

    /// Calculates the maximum skew for a single triangular face.
    fn face_maximum_skew(&self, n0_idx: usize, n1_idx: usize, n2_idx: usize) -> f64 {
        let deg_to_rad = PI / 180.0;
        let equilateral_rad = 60.0 * deg_to_rad;
        let minimum_angle = self.face_minimum_angle(n0_idx, n1_idx, n2_idx);

        if (equilateral_rad - minimum_angle).abs() < TOLERANCE {
            0.0
        } else {
            (equilateral_rad - minimum_angle) / equilateral_rad
        }
    }

    pub fn hex_to_tet(
        &[
            node_0,
            node_1,
            node_2,
            node_3,
            node_4,
            node_5,
            node_6,
            node_7,
        ]: &[usize; HEX],
    ) -> [[usize; TET]; NUM_TETS_PER_HEX] {
        [
            [node_0, node_1, node_3, node_4],
            [node_4, node_5, node_1, node_7],
            [node_7, node_4, node_3, node_1],
            [node_1, node_5, node_2, node_7],
            [node_5, node_6, node_2, node_7],
            [node_7, node_3, node_2, node_1],
        ]
    }
}

impl From<HexahedralFiniteElements> for TetrahedralFiniteElements {
    fn from(hexes: HexahedralFiniteElements) -> Self {
        let (hex_blocks, hex_connectivity, nodal_coordinates) = hexes.into();
        let element_blocks = hex_blocks
            .into_iter()
            .flat_map(|hex_block| repeat_n(hex_block, NUM_TETS_PER_HEX))
            .collect();
        let element_node_connectivity =
            hex_connectivity.iter().flat_map(Self::hex_to_tet).collect();
        Self::from((element_blocks, element_node_connectivity, nodal_coordinates))
    }
}

impl From<Tessellation> for TetrahedralFiniteElements {
    fn from(_tessellation: Tessellation) -> Self {
        unimplemented!()
    }
}
