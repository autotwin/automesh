#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Connectivity, Coordinates, FiniteElementMethods, FiniteElementSpecifics, FiniteElements, 
    HEX, HexahedralFiniteElements, Metrics, Size, Smoothing, Tessellation, Vector, 
};
use conspire::math::Tensor;
// use conspire::math::{Tensor, TensorArray, TensorVec};
use ndarray::parallel::prelude::*;
use std::{io::Error as ErrorIO, iter::repeat_n};

/// The number of nodes in a tetrahedral finite element.
pub const TET: usize = 4;

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
            .par_iter()
            .map(|connectivity| {
                let edge_vectors = self.edge_vectors(connectivity);
                let lengths: Vec<f64> = edge_vectors.iter().map(|v| v.norm()).collect();
                let min_length = lengths.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_length = lengths.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                max_length / min_length
            })
            .collect::<Vec<f64>>()
            .into()
    }
    fn maximum_skews(&self) -> Metrics {
        todo!()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                // The element Jacobian j is 6.0 times the signed element volume
                let j = self.signed_element_volume(connectivity) * 6.0;

                // Get all six edge lengths
                let edge_vectors = self.edge_vectors(connectivity);
                let els: Vec<f64> = edge_vectors.iter().map(|v| v.norm()).collect();

                // Compute the four nodal Jacobians
                let lambda_0 = els[0] * els[2] * els[3];
                let lambda_1 = els[0] * els[1] * els[4];
                let lambda_2 = els[1] * els[2] * els[5];
                let lambda_3 = els[3] * els[4] * els[5];

                // Find the maximum of the nodal Jacobians (including the element Jacobian)
                let lambda_max = [j, lambda_0, lambda_1, lambda_2, lambda_3]
                    .into_iter()
                    .reduce(f64::max)
                    .unwrap_or(f64::INFINITY); // use unwrap_or for safety

                // Calculate the final quality metric
                if lambda_max == 0.0 {
                    0.0 // Avoid division by zero for collapsed elements
                } else {
                    j * 2.0_f64.sqrt() / lambda_max
                }
            })
            .collect::<Vec<f64>>()
            .into()
    }
    fn remesh(&mut self, _iterations: usize, _smoothing_method: &Smoothing, _size: Size) {
        todo!()
    }
    fn write_metrics(&self, _file_path: &str) -> Result<(), ErrorIO> {
        todo!()
    }
}

impl TetrahedralFiniteElements {
    fn edge_vectors(&self, connectivity: &[usize; TET]) -> Vec<Vector> {
        // TODO: Ask Michael about the differences here.
        // let nodal_coordinates = self.get_nodal_coordinates();
        let nodal_coordinates = &self.nodal_coordinates;
        // Base edges (in a cycle 0 -> 1 -> 2 -> 0])
        let e0 = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]];
        let e1 = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]];
        let e2 = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]];
        
        // Edges connecting the apex (node 3)
        let e3 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[0]];
        let e4 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[1]];
        let e5 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[2]];
        
        // Return all six edge vectors
        vec![e0, e1, e2, e3, e4, e5]
    }

    // Helper function to calculate the signed volume of a single tetrahedron.
    fn signed_element_volume(&self, connectivity: &[usize; TET]) -> f64 {
        let nodal_coordinates = self.get_nodal_coordinates();
        let v0 = &nodal_coordinates[connectivity[0]];
        let v1 = &nodal_coordinates[connectivity[1]];
        let v2 = &nodal_coordinates[connectivity[2]];
        let v3 = &nodal_coordinates[connectivity[3]];
        ((v1 - v0).cross(&(v2 - v0)) * &(v3 - v0)) / 6.0
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
            .collect::<Vec<f64>>()
            .into()
    }

    pub fn hex_to_tet(connectivity: &[usize; HEX]) -> [[usize; TET]; NUM_TETS_PER_HEX] {
        [
            [
                connectivity[0],
                connectivity[1],
                connectivity[3],
                connectivity[4],
            ],
            [
                connectivity[4],
                connectivity[5],
                connectivity[1],
                connectivity[7],
            ],
            [
                connectivity[7],
                connectivity[4],
                connectivity[3],
                connectivity[1],
            ],
            [
                connectivity[1],
                connectivity[5],
                connectivity[2],
                connectivity[7],
            ],
            [
                connectivity[5],
                connectivity[6],
                connectivity[2],
                connectivity[7],
            ],
            [
                connectivity[7],
                connectivity[3],
                connectivity[2],
                connectivity[1],
            ],
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