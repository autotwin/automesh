#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Connectivity, Coordinates, FiniteElementMethods, FiniteElementSpecifics, FiniteElements, HEX,
    HexahedralFiniteElements, Metrics, Size, Smoothing, Tessellation,
};
#[cfg(test)]
use super::Vector;
use conspire::{
    geometry::mesh::{Connectivity as MeshConnectivity, Mesh, Verdict},
    math::Tensor,
};
#[cfg(test)]
use conspire::math::CrossProduct;
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    iter::repeat_n,
    path::Path,
};

/// The number of nodes in a tetrahedral finite element.
pub const TET: usize = 4;

const O: usize = 3;
#[cfg(test)]
const NUM_EDGES: usize = 6;
const NUM_NODES_FACE: usize = 3;

/// The tetrahedral finite elements type.
pub type TetrahedralFiniteElements = FiniteElements<TET>;

/// The number of tetrahedral elements created from a single hexahedral element.
pub const NUM_TETS_PER_HEX: usize = 6;

impl FiniteElementSpecifics<NUM_NODES_FACE, O> for TetrahedralFiniteElements {
    fn connected_nodes(node: &usize) -> [usize; O] {
        match node {
            0 => [1, 2, 3],
            1 => [0, 2, 3],
            2 => [0, 1, 3],
            3 => [0, 1, 2],
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
    #[cfg(test)]
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
    #[cfg(test)]
    fn signed_element_volume(&self, &[node_0, node_1, node_2, node_3]: &[usize; TET]) -> f64 {
        let nodal_coordinates = self.get_nodal_coordinates();
        let v0 = &nodal_coordinates[node_0];
        let v1 = &nodal_coordinates[node_1];
        let v2 = &nodal_coordinates[node_2];
        let v3 = &nodal_coordinates[node_3];
        ((v1 - v0).cross(v2 - v0) * (v3 - v0)) / 6.0
    }

    fn as_mesh(&self) -> Mesh<3> {
        let connectivity = self
            .get_element_node_connectivity()
            .clone()
            .into_iter()
            .collect::<Vec<[usize; TET]>>();
        let coordinates = self
            .get_nodal_coordinates()
            .iter()
            .map(|coordinate| [coordinate[0], coordinate[1], coordinate[2]])
            .collect::<Vec<[f64; 3]>>();
        Mesh::<3>::from((
            vec![MeshConnectivity::Tetrahedral(connectivity.into())],
            coordinates.into(),
        ))
    }

    // Calculates the signed volumes for all tetrahedral elements in the mesh.
    pub fn volumes(&self) -> Metrics {
        self.as_mesh().volumes().into_iter().flatten().collect()
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
