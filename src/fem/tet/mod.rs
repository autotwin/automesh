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
                let (e0, e1, e2, e3, e4, e5) = self.edge_vectors(connectivity);
                let lengths = [
                    e0.norm(),
                    e1.norm(),
                    e2.norm(),
                    e3.norm(),
                    e4.norm(),
                    e5.norm(),
                ];
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
        todo!()
    }
    fn remesh(&mut self, _iterations: usize, _smoothing_method: &Smoothing, _size: Size) {
        todo!()
    }
    fn write_metrics(&self, _file_path: &str) -> Result<(), ErrorIO> {
        todo!()
    }
}

impl TetrahedralFiniteElements {
    fn edge_vectors(&self, connectivity: &[usize; TET]) -> (Vector, Vector, Vector, Vector, Vector, Vector) {
        let nodal_coordinates = self.get_nodal_coordinates();
        // Base edges (in a cycle 0 -> 1 -> 2 -> 0])
        let e0 = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]];
        let e1 = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]];
        let e2 = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]];
        
        // Edges connecting the apex (node 3)
        let e3 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[0]];
        let e4 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[1]];
        let e5 = &nodal_coordinates[connectivity[3]] - &nodal_coordinates[connectivity[2]];
        
        // Return all six edge vectors
        (e0, e1, e2, e3, e4, e5)
    }
    // fn volumes(&self) -> Metrics {
    //     let nodal_coordinates = &self.nodal_coordinates;
    //     self.element_node_connectivity
    //         .iter()
    //         .map(|&[n0, n1, n2, n3]| {
    //             let v0 = &nodal_coordinates[n0];
    //             let v1 = &nodal_coordinates[n1];
    //             let v2 = &nodal_coordinates[n2];
    //             let v3 = &nodal_coordinates[n3];
    //             ((v0 - v2).cross(&(v1 - v0)) * (v3 - v0)).abs() / 6.0
    //         })
    //         .collect()
    
    // Parallel version of the volumes function.
    fn volumes(&self) -> Metrics {
        let nodal_coordinates = &self.nodal_coordinates;

        // Create a parallel implementation of volumes that is better because:
        // 1. `part_iter()`, from the rayon crate, replaces `iter()` and will
        // create a parallel iterator that will automatically spread the work
        // of the .map() operation across available CPU codes.
        // 2. `.collect::<Vec<f64>>().into()` is needed because par_iter()
        // collects into a standard Vec.  we first collect the results into a
        // Vec<f64> and then use .into() to convert it to the Metrics type
        // (ndarray::Array1<f64>), which is an inexpensive operation.

        self.element_node_connectivity
            .par_iter()
            .map(|&[n0, n1, n2, n3]| {
                let v0 = &nodal_coordinates[n0];
                let v1 = &nodal_coordinates[n1];
                let v2 = &nodal_coordinates[n2];
                let v3 = &nodal_coordinates[n3];
                // ((v0 - v2).cross(&(v1 - v0)) * (v3 - v0)).abs() / 6.0
                ((v1 - v0).cross(&(v2 - v0)) * &(v3 - v0)).abs() / 6.0
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