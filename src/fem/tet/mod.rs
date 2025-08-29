use crate::FiniteElementMethods;

use super::{
    FiniteElementSpecifics, FiniteElements, HEX, HexahedralFiniteElements, Metrics, Smoothing,
    Tessellation,
};
use std::{io::Error as ErrorIO, iter::repeat_n};

/// The number of nodes in a tetrahedral finite element.
pub const TET: usize = 4;

/// The tetrahedral finite elements type.
pub type TetrahedralFiniteElements = FiniteElements<TET>;

pub const NUM_TETS_PER_HEX: usize = 6;

impl FiniteElementSpecifics for TetrahedralFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 2, 3],
            1 => vec![0, 2, 3],
            2 => vec![0, 1, 3],
            3 => vec![0, 1, 2],
            _ => panic!(),
        }
    }
    fn maximum_edge_ratios(&self) -> Metrics {
        todo!()
    }
    fn maximum_skews(&self) -> Metrics {
        todo!()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        todo!()
    }
    fn remesh(&mut self, _iterations: usize, _smoothing_method: &Smoothing) {
        todo!()
    }
    fn write_metrics(&self, _file_path: &str) -> Result<(), ErrorIO> {
        todo!()
    }
}

impl TetrahedralFiniteElements {
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
        let (hex_blocks, hex_connectivity, nodal_coordinates) = hexes.data();
        let blocks = hex_blocks
            .into_iter()
            .flat_map(|hex_block| repeat_n(hex_block, NUM_TETS_PER_HEX))
            .collect();
        let element_node_connectivity =
            hex_connectivity.iter().flat_map(Self::hex_to_tet).collect();
        Self::from_data(blocks, element_node_connectivity, nodal_coordinates)
    }
}

impl From<Tessellation> for TetrahedralFiniteElements {
    fn from(_tessellation: Tessellation) -> Self {
        unimplemented!()
    }
}
