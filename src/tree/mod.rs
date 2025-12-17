#[cfg(feature = "profile")]
use std::time::Instant;

mod hex;
mod tessellation;
mod tri;

pub use hex::HexesAndCoords;
pub use tessellation::{Samples, octree_from_surface};

use super::{
    Coordinate, Coordinates, NSD, Vector,
    fem::{Blocks, FiniteElementMethods, HEX, HexahedralFiniteElements, hex::HexConnectivity},
    voxel::{Nel, Remove, Scale, Translate, VoxelData, Voxels},
};
use conspire::math::{Tensor, TensorArray, TensorVec, tensor_rank_1};
use ndarray::{Axis, parallel::prelude::*, s};
use std::{
    array::from_fn,
    collections::HashMap,
    iter::repeat_n,
    ops::{Deref, DerefMut},
};
use tessellation::octree_from_bounding_cube;

pub const PADDING: u8 = 255;

const NUM_EDGES: usize = 8;
const NUM_FACES: usize = 6;
const NUM_OCTANTS: usize = 8;
const NUM_NODES_CELL: usize = 8;
const NUM_NODES_FACE: usize = 4;
const NUM_SUBCELLS_FACE: usize = 4;
const NUM_SUBSUBCELLS_FACE: usize = 16;

type SubcellsOnFace = [usize; NUM_SUBCELLS_FACE];
const SUBCELLS_ON_OWN_FACE_0: SubcellsOnFace = [0, 1, 4, 5];
const SUBCELLS_ON_OWN_FACE_1: SubcellsOnFace = [1, 3, 5, 7];
const SUBCELLS_ON_OWN_FACE_2: SubcellsOnFace = [2, 3, 6, 7];
const SUBCELLS_ON_OWN_FACE_3: SubcellsOnFace = [0, 2, 4, 6];
const SUBCELLS_ON_OWN_FACE_4: SubcellsOnFace = [0, 1, 2, 3];
const SUBCELLS_ON_OWN_FACE_5: SubcellsOnFace = [4, 5, 6, 7];

const fn face_direction(face_index: usize) -> Vector {
    match face_index {
        0 => tensor_rank_1([0.0, -1.0, 0.0]),
        1 => tensor_rank_1([1.0, 0.0, 0.0]),
        2 => tensor_rank_1([0.0, 1.0, 0.0]),
        3 => tensor_rank_1([-1.0, 0.0, 0.0]),
        4 => tensor_rank_1([0.0, 0.0, -1.0]),
        5 => tensor_rank_1([0.0, 0.0, 1.0]),
        _ => panic!(),
    }
}

const fn mirror_face(face: usize) -> usize {
    match face {
        0 => 2,
        1 => 3,
        2 => 0,
        3 => 1,
        4 => 5,
        5 => 4,
        _ => {
            panic!()
        }
    }
}

const fn subcells_on_own_face(face: usize) -> SubcellsOnFace {
    match face {
        0 => SUBCELLS_ON_OWN_FACE_0,
        1 => SUBCELLS_ON_OWN_FACE_1,
        2 => SUBCELLS_ON_OWN_FACE_2,
        3 => SUBCELLS_ON_OWN_FACE_3,
        4 => SUBCELLS_ON_OWN_FACE_4,
        5 => SUBCELLS_ON_OWN_FACE_5,
        _ => {
            panic!()
        }
    }
}

const fn subcells_on_neighbor_face(face: usize) -> SubcellsOnFace {
    match face {
        0 => SUBCELLS_ON_OWN_FACE_2,
        1 => SUBCELLS_ON_OWN_FACE_3,
        2 => SUBCELLS_ON_OWN_FACE_0,
        3 => SUBCELLS_ON_OWN_FACE_1,
        4 => SUBCELLS_ON_OWN_FACE_5,
        5 => SUBCELLS_ON_OWN_FACE_4,
        _ => {
            panic!()
        }
    }
}

const fn subcells_on_own_face_contains(face: usize, subcell: usize) -> bool {
    match face {
        0 => matches!(subcell, 0 | 1 | 4 | 5),
        1 => matches!(subcell, 1 | 3 | 5 | 7),
        2 => matches!(subcell, 2 | 3 | 6 | 7),
        3 => matches!(subcell, 0 | 2 | 4 | 6),
        4 => matches!(subcell, 0..=3),
        5 => matches!(subcell, 4..=7),
        _ => {
            panic!()
        }
    }
}

type Cells = [Cell; NUM_OCTANTS];
pub type Edge = [usize; 2];
pub type Edges = Vec<Edge>;
type Faces = [Option<usize>; NUM_FACES];
type Indices = [usize; NUM_OCTANTS];
type NodeMap = HashMap<[usize; NSD], usize>;
type SubSubCellsFace = [usize; NUM_SUBSUBCELLS_FACE];

/// The octree type.
#[derive(Debug)]
pub struct Octree {
    nel: Nel,
    octree: Vec<Cell>,
    remove: Remove,
    scale: Scale,
    translate: Translate,
}

impl Deref for Octree {
    type Target = Vec<Cell>;
    fn deref(&self) -> &Self::Target {
        &self.octree
    }
}

impl DerefMut for Octree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.octree
    }
}

type Cluster = Vec<usize>;
type Clusters = Vec<Cluster>;
type Supercells = Vec<Option<[usize; 2]>>;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub block: Option<u8>,
    cells: Option<Indices>,
    faces: Faces,
    lngth: u16,
    min_x: u16,
    min_y: u16,
    min_z: u16,
}

impl Cell {
    pub fn any_samples_inside(&self, samples: &[Vec<Vec<bool>>]) -> bool {
        let min_x = self.min_x as usize;
        let min_y = self.min_y as usize;
        let min_z = self.min_z as usize;
        let max_x = (self.min_x + self.lngth) as usize;
        let max_y = (self.min_y + self.lngth) as usize;
        let max_z = (self.min_z + self.lngth) as usize;
        (min_x..max_x).any(|i| (min_y..max_y).any(|j| (min_z..max_z).any(|k| samples[i][j][k])))
    }
    pub const fn get_block(&self) -> u8 {
        if let Some(block) = self.block {
            block
        } else {
            panic!("Called get_block() on a non-leaf cell.")
        }
    }
    pub const fn get_cells(&self) -> &Option<Indices> {
        &self.cells
    }
    pub const fn get_center(&self) -> [f64; 3] {
        let half_length: f64 = 0.5 * self.lngth as f64;
        [
            self.min_x as f64 + half_length,
            self.min_y as f64 + half_length,
            self.min_z as f64 + half_length,
        ]
    }
    pub const fn get_faces(&self) -> &Faces {
        &self.faces
    }
    pub const fn get_lngth(&self) -> &u16 {
        &self.lngth
    }
    pub const fn get_min_x(&self) -> &u16 {
        &self.min_x
    }
    pub const fn get_min_y(&self) -> &u16 {
        &self.min_y
    }
    pub const fn get_min_z(&self) -> &u16 {
        &self.min_z
    }
    pub const fn get_max_x(&self) -> u16 {
        self.min_x + self.lngth
    }
    pub const fn get_max_y(&self) -> u16 {
        self.min_y + self.lngth
    }
    pub const fn get_max_z(&self) -> u16 {
        self.min_z + self.lngth
    }
    pub const fn get_nodal_indices_cell(&self) -> [[usize; NSD]; NUM_NODES_CELL] {
        [
            [
                *self.get_min_x() as usize,
                *self.get_min_y() as usize,
                *self.get_min_z() as usize,
            ],
            [
                self.get_max_x() as usize,
                *self.get_min_y() as usize,
                *self.get_min_z() as usize,
            ],
            [
                self.get_max_x() as usize,
                self.get_max_y() as usize,
                *self.get_min_z() as usize,
            ],
            [
                *self.get_min_x() as usize,
                self.get_max_y() as usize,
                *self.get_min_z() as usize,
            ],
            [
                *self.get_min_x() as usize,
                *self.get_min_y() as usize,
                self.get_max_z() as usize,
            ],
            [
                self.get_max_x() as usize,
                *self.get_min_y() as usize,
                self.get_max_z() as usize,
            ],
            [
                self.get_max_x() as usize,
                self.get_max_y() as usize,
                self.get_max_z() as usize,
            ],
            [
                *self.get_min_x() as usize,
                self.get_max_y() as usize,
                self.get_max_z() as usize,
            ],
        ]
    }
    pub const fn get_nodal_indices_face(&self, face_index: &usize) -> [[u16; NSD]; NUM_NODES_FACE] {
        match face_index {
            0 => [
                [*self.get_min_x(), *self.get_min_y(), *self.get_min_z()],
                [self.get_max_x(), *self.get_min_y(), *self.get_min_z()],
                [self.get_max_x(), *self.get_min_y(), self.get_max_z()],
                [*self.get_min_x(), *self.get_min_y(), self.get_max_z()],
            ],
            1 => [
                [self.get_max_x(), *self.get_min_y(), *self.get_min_z()],
                [self.get_max_x(), self.get_max_y(), *self.get_min_z()],
                [self.get_max_x(), self.get_max_y(), self.get_max_z()],
                [self.get_max_x(), *self.get_min_y(), self.get_max_z()],
            ],
            2 => [
                [self.get_max_x(), self.get_max_y(), *self.get_min_z()],
                [*self.get_min_x(), self.get_max_y(), *self.get_min_z()],
                [*self.get_min_x(), self.get_max_y(), self.get_max_z()],
                [self.get_max_x(), self.get_max_y(), self.get_max_z()],
            ],
            3 => [
                [*self.get_min_x(), self.get_max_y(), *self.get_min_z()],
                [*self.get_min_x(), *self.get_min_y(), *self.get_min_z()],
                [*self.get_min_x(), *self.get_min_y(), self.get_max_z()],
                [*self.get_min_x(), self.get_max_y(), self.get_max_z()],
            ],
            4 => [
                [*self.get_min_x(), *self.get_min_y(), *self.get_min_z()],
                [*self.get_min_x(), self.get_max_y(), *self.get_min_z()],
                [self.get_max_x(), self.get_max_y(), *self.get_min_z()],
                [self.get_max_x(), *self.get_min_y(), *self.get_min_z()],
            ],
            5 => [
                [*self.get_min_x(), *self.get_min_y(), self.get_max_z()],
                [self.get_max_x(), *self.get_min_y(), self.get_max_z()],
                [self.get_max_x(), self.get_max_y(), self.get_max_z()],
                [*self.get_min_x(), self.get_max_y(), self.get_max_z()],
            ],
            _ => {
                panic!()
            }
        }
    }
    pub fn homogeneous(&self, data: &VoxelData) -> Option<u8> {
        let x_min = *self.get_min_x() as usize;
        let y_min = *self.get_min_y() as usize;
        let z_min = *self.get_min_z() as usize;
        let x_max = self.get_max_x() as usize;
        let y_max = self.get_max_y() as usize;
        let z_max = self.get_max_z() as usize;
        let contained = data.slice(s![x_min..x_max, y_min..y_max, z_min..z_max]);
        let block_0 = contained[(0, 0, 0)];
        if contained.iter().all(|&block| block == block_0) {
            Some(block_0)
        } else {
            None
        }
    }
    pub fn homogeneous_coordinates(
        &self,
        blocks: &Blocks,
        coordinates: &Coordinates,
    ) -> Option<u8> {
        let x_min = *self.get_min_x() as f64;
        let y_min = *self.get_min_y() as f64;
        let z_min = *self.get_min_z() as f64;
        let x_max = self.get_max_x() as f64;
        let y_max = self.get_max_y() as f64;
        let z_max = self.get_max_z() as f64;
        let insides = coordinates
            .iter()
            .enumerate()
            .filter(|(_, coordinate)| {
                coordinate[0] >= x_min
                    && coordinate[0] < x_max
                    && coordinate[1] >= y_min
                    && coordinate[1] < y_max
                    && coordinate[2] >= z_min
                    && coordinate[2] < z_max
            })
            .map(|(index, _)| index)
            .collect::<Vec<usize>>();
        if insides.is_empty() {
            Some(PADDING)
        } else {
            let block_0 = blocks[insides[0]];
            if insides
                .iter()
                .map(|&index| blocks[index])
                .all(|block| block == block_0)
            {
                Some(block_0)
            } else if self.is_voxel() {
                let center = Coordinate::new(self.get_center());
                let min_index = insides
                    .into_iter()
                    .reduce(|min_index, index| {
                        if (&coordinates[min_index] - &center).norm()
                            > (&coordinates[index] - &center).norm()
                        {
                            index
                        } else {
                            min_index
                        }
                    })
                    .unwrap();
                Some(blocks[min_index])
            } else {
                None
            }
        }
    }
    pub fn is_face_on_octree_boundary(&self, face_index: &usize, nel: Nel) -> bool {
        match face_index {
            0 => self.get_min_y() == &0,
            1 => self.get_max_x() == *nel.x() as u16,
            2 => self.get_max_y() == *nel.y() as u16,
            3 => self.get_min_x() == &0,
            4 => self.get_min_z() == &0,
            5 => self.get_max_z() == *nel.z() as u16,
            _ => panic!(),
        }
    }
    pub const fn is_leaf(&self) -> bool {
        self.get_cells().is_none()
    }
    pub const fn is_not_leaf(&self) -> bool {
        self.get_cells().is_some()
    }
    pub const fn is_voxel(&self) -> bool {
        self.lngth == 1
    }
    pub const fn is_not_voxel(&self) -> bool {
        self.lngth != 1
    }
    pub fn subdivide(&mut self, indices: Indices) -> Cells {
        self.cells = Some(indices);
        let lngth = self.get_lngth() / 2;
        let min_x = self.get_min_x();
        let min_y = self.get_min_y();
        let min_z = self.get_min_z();
        let val_x = min_x + lngth;
        let val_y = min_y + lngth;
        let val_z = min_z + lngth;
        [
            Cell {
                block: None,
                cells: None,
                faces: [
                    None,
                    Some(indices[1]),
                    Some(indices[2]),
                    None,
                    None,
                    Some(indices[4]),
                ],
                lngth,
                min_x: *min_x,
                min_y: *min_y,
                min_z: *min_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    None,
                    None,
                    Some(indices[3]),
                    Some(indices[0]),
                    None,
                    Some(indices[5]),
                ],
                lngth,
                min_x: val_x,
                min_y: *min_y,
                min_z: *min_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    Some(indices[0]),
                    Some(indices[3]),
                    None,
                    None,
                    None,
                    Some(indices[6]),
                ],
                lngth,
                min_x: *min_x,
                min_y: val_y,
                min_z: *min_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    Some(indices[1]),
                    None,
                    None,
                    Some(indices[2]),
                    None,
                    Some(indices[7]),
                ],
                lngth,
                min_x: val_x,
                min_y: val_y,
                min_z: *min_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    None,
                    Some(indices[5]),
                    Some(indices[6]),
                    None,
                    Some(indices[0]),
                    None,
                ],
                lngth,
                min_x: *min_x,
                min_y: *min_y,
                min_z: val_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    None,
                    None,
                    Some(indices[7]),
                    Some(indices[4]),
                    Some(indices[1]),
                    None,
                ],
                lngth,
                min_x: val_x,
                min_y: *min_y,
                min_z: val_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    Some(indices[4]),
                    Some(indices[7]),
                    None,
                    None,
                    Some(indices[2]),
                    None,
                ],
                lngth,
                min_x: *min_x,
                min_y: val_y,
                min_z: val_z,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    Some(indices[5]),
                    None,
                    None,
                    Some(indices[6]),
                    Some(indices[3]),
                    None,
                ],
                lngth,
                min_x: val_x,
                min_y: val_y,
                min_z: val_z,
            },
        ]
    }
}

impl Octree {
    pub fn balance_and_pair(&mut self, strong: bool) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut balanced = false;
        let mut paired = false;
        while !balanced || !paired {
            balanced = self.balance(strong);
            paired = self.pair();
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mBalancing and pairing\x1b[0m {:?}",
            time.elapsed()
        );
    }
    pub fn balance(&mut self, strong: bool) -> bool {
        let mut balanced;
        let mut balanced_already = true;
        let mut block;
        let mut edges = [false; NUM_EDGES];
        let mut index;
        let mut subdivide;
        let mut vertices = [false; 2];
        #[allow(unused_variables)]
        for iteration in 1.. {
            balanced = true;
            index = 0;
            subdivide = false;
            // #[cfg(feature = "profile")]
            // let time = Instant::now();
            while index < self.len() {
                if !self[index].is_voxel() && self[index].is_leaf() {
                    'faces: for (face, face_cell) in self[index].get_faces().iter().enumerate() {
                        if let Some(neighbor) = face_cell
                            && let Some(kids) = self[*neighbor].get_cells()
                        {
                            if strong {
                                edges = from_fn(|_| false);
                                vertices = from_fn(|_| false);
                            }
                            if match face {
                                0 => {
                                    if strong {
                                        if let Some(edge_cell) = self[kids[3]].get_faces()[1] {
                                            edges[0] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[7]].get_faces()[1] {
                                            edges[1] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[6]].get_faces()[5] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[3]
                                            {
                                                vertices[0] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[2] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[7]].get_faces()[5] {
                                            edges[3] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[2]].get_faces()[3] {
                                            edges[4] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[6]].get_faces()[3] {
                                            edges[5] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[2]].get_faces()[4] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[3]
                                            {
                                                vertices[1] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[6] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[3]].get_faces()[4] {
                                            edges[7] = self[edge_cell].is_not_leaf()
                                        }
                                    }
                                    self[kids[2]].is_not_leaf()
                                        || self[kids[3]].is_not_leaf()
                                        || self[kids[6]].is_not_leaf()
                                        || self[kids[7]].is_not_leaf()
                                        || edges.into_iter().any(|edge: bool| edge)
                                        || vertices.into_iter().any(|vertex| vertex)
                                }
                                1 => {
                                    if strong {
                                        if let Some(edge_cell) = self[kids[2]].get_faces()[2] {
                                            edges[0] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[6]].get_faces()[2] {
                                            edges[1] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[4]].get_faces()[5] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[0]
                                            {
                                                vertices[0] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[2] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[6]].get_faces()[5] {
                                            edges[3] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[0]].get_faces()[0] {
                                            edges[4] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[4]].get_faces()[0] {
                                            edges[5] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[0]].get_faces()[4] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[0]
                                            {
                                                vertices[1] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[6] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[2]].get_faces()[4] {
                                            edges[7] = self[edge_cell].is_not_leaf()
                                        }
                                    }
                                    self[kids[0]].is_not_leaf()
                                        || self[kids[2]].is_not_leaf()
                                        || self[kids[4]].is_not_leaf()
                                        || self[kids[6]].is_not_leaf()
                                        || edges.into_iter().any(|edge| edge)
                                        || vertices.into_iter().any(|vertex| vertex)
                                }
                                2 => {
                                    if strong {
                                        if let Some(edge_cell) = self[kids[0]].get_faces()[3] {
                                            edges[0] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[4]].get_faces()[3] {
                                            edges[1] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[4]].get_faces()[5] {
                                            edges[2] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[5]].get_faces()[5] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[1]
                                            {
                                                vertices[0] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[3] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[1]].get_faces()[1] {
                                            edges[4] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[5]].get_faces()[1] {
                                            edges[5] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[0]].get_faces()[4] {
                                            edges[6] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[1]].get_faces()[4] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[1]
                                            {
                                                vertices[1] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[7] = self[edge_cell].is_not_leaf()
                                        }
                                    }
                                    self[kids[0]].is_not_leaf()
                                        || self[kids[1]].is_not_leaf()
                                        || self[kids[4]].is_not_leaf()
                                        || self[kids[5]].is_not_leaf()
                                        || edges.into_iter().any(|edge| edge)
                                        || vertices.into_iter().any(|vertex| vertex)
                                }
                                3 => {
                                    if strong {
                                        if let Some(edge_cell) = self[kids[1]].get_faces()[0] {
                                            edges[0] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[5]].get_faces()[0] {
                                            edges[1] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[5]].get_faces()[5] {
                                            edges[2] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[7]].get_faces()[5] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[2]
                                            {
                                                vertices[0] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[3] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[3]].get_faces()[2] {
                                            edges[4] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[7]].get_faces()[2] {
                                            edges[5] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[1]].get_faces()[4] {
                                            edges[6] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[3]].get_faces()[4] {
                                            if let Some(vertex_cell) =
                                                self[edge_cell].get_faces()[2]
                                            {
                                                vertices[1] = self[vertex_cell].is_not_leaf()
                                            }
                                            edges[7] = self[edge_cell].is_not_leaf()
                                        }
                                    }
                                    self[kids[1]].is_not_leaf()
                                        || self[kids[3]].is_not_leaf()
                                        || self[kids[5]].is_not_leaf()
                                        || self[kids[7]].is_not_leaf()
                                        || edges.into_iter().any(|edge| edge)
                                        || vertices.into_iter().any(|vertex| vertex)
                                }
                                4 => {
                                    if strong {
                                        if let Some(edge_cell) = self[kids[5]].get_faces()[1] {
                                            edges[0] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[7]].get_faces()[1] {
                                            edges[1] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[6]].get_faces()[2] {
                                            edges[2] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[7]].get_faces()[2] {
                                            edges[3] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[4]].get_faces()[3] {
                                            edges[4] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[6]].get_faces()[3] {
                                            edges[5] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[4]].get_faces()[0] {
                                            edges[6] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[5]].get_faces()[0] {
                                            edges[7] = self[edge_cell].is_not_leaf()
                                        }
                                    }
                                    edges.into_iter().any(|edge| edge)
                                        || self[kids[4]].is_not_leaf()
                                        || self[kids[5]].is_not_leaf()
                                        || self[kids[6]].is_not_leaf()
                                        || self[kids[7]].is_not_leaf()
                                }
                                5 => {
                                    if strong {
                                        if let Some(edge_cell) = self[kids[1]].get_faces()[1] {
                                            edges[0] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[3]].get_faces()[1] {
                                            edges[1] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[2]].get_faces()[2] {
                                            edges[2] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[3]].get_faces()[2] {
                                            edges[3] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[0]].get_faces()[3] {
                                            edges[4] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[2]].get_faces()[3] {
                                            edges[5] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[0]].get_faces()[0] {
                                            edges[6] = self[edge_cell].is_not_leaf()
                                        }
                                        if let Some(edge_cell) = self[kids[1]].get_faces()[0] {
                                            edges[7] = self[edge_cell].is_not_leaf()
                                        }
                                    }
                                    edges.into_iter().any(|edge| edge)
                                        || self[kids[0]].is_not_leaf()
                                        || self[kids[1]].is_not_leaf()
                                        || self[kids[2]].is_not_leaf()
                                        || self[kids[3]].is_not_leaf()
                                }
                                _ => panic!(),
                            } {
                                subdivide = true;
                                break 'faces;
                            }
                        }
                    }
                    if subdivide {
                        block = self[index].get_block();
                        self.subdivide(index);
                        self.iter_mut()
                            .rev()
                            .take(NUM_OCTANTS)
                            .for_each(|cell| cell.block = Some(block));
                        balanced = false;
                        balanced_already = false;
                        subdivide = false;
                    }
                }
                index += 1;
            }
            // #[cfg(feature = "profile")]
            // println!(
            //     "             \x1b[1;93mBalancing iteration {}\x1b[0m {:?} ",
            //     iteration,
            //     time.elapsed()
            // );
            if balanced {
                break;
            }
        }
        balanced_already
    }
    fn boundaries(&mut self) {
        //
        // Consider having this skip blocks that will be removed.
        // Also, for this and other places, should you always remove the padding?
        //
        let mut block;
        let mut boundaries;
        let mut cell;
        let mut index;
        #[allow(unused_variables)]
        for iteration in 1.. {
            boundaries = true;
            index = 0;
            #[cfg(feature = "profile")]
            let time = Instant::now();
            while index < self.len() {
                cell = self[index];
                if cell.get_lngth() > &1 && cell.is_leaf() {
                    block = cell.get_block();
                    if cell
                        .get_faces()
                        .iter()
                        .flatten()
                        .filter(|&face| self[*face].is_leaf())
                        .any(|face| self[*face].get_block() != block)
                        || cell
                            .get_faces()
                            .iter()
                            .enumerate()
                            .any(|(face, &face_cell_maybe)| {
                                if let Some(face_cell) = face_cell_maybe {
                                    if let Some(subcells) = self[face_cell].get_cells() {
                                        //
                                        // Since subdivision here can create unbalancing,
                                        // balancing is called at the end,
                                        // but balancing is still needed beforehand,
                                        // otherwise a leaf can face grand kids here.
                                        // Unknown whether unbalancing here can reintroduce that,
                                        // which would require rebalancing for every subdivision.
                                        //
                                        subcells_on_neighbor_face(face).iter().any(|&subcell| {
                                            self[subcells[subcell]].get_block() != block
                                        })
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            })
                        || cell
                            .get_faces()
                            .iter()
                            .enumerate()
                            .filter(|(_, face)| face.is_none())
                            .any(|(face_index, _)| {
                                cell.is_face_on_octree_boundary(&face_index, self.nel())
                            })
                    {
                        self.subdivide(index);
                        self.iter_mut()
                            .rev()
                            .take(NUM_OCTANTS)
                            .for_each(|cell| cell.block = Some(block));
                        boundaries = false;
                    }
                }
                index += 1;
            }
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mBoundaries iteration {}\x1b[0m {:?} ",
                iteration,
                time.elapsed()
            );
            if boundaries {
                break;
            }
        }
        self.balance(true);
    }
    fn clusters(&self, remove: Option<&Blocks>, supercells_opt: Option<&Supercells>) -> Clusters {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let removed_data = remove.unwrap();
        let supercells = if let Some(supercells) = supercells_opt {
            supercells
        } else {
            &self.supercells()
        };
        let mut blocks: Blocks = self
            .iter()
            .filter(|cell| cell.is_leaf() && removed_data.binary_search(&cell.get_block()).is_err())
            .map(|cell| cell.get_block())
            .collect();
        blocks.sort();
        blocks.dedup();
        let mut leaves: Vec<Vec<usize>> = blocks
            .iter()
            .map(|&block| {
                self.iter()
                    .enumerate()
                    .filter_map(|(index, cell)| {
                        if cell.is_leaf() && cell.get_block() == block {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();
        leaves
            .iter_mut()
            .for_each(|block_leaves| block_leaves.sort());
        let clusters = blocks
            .into_par_iter()
            .zip(leaves.par_iter_mut())
            .flat_map(|(block, block_leaves)| {
                let mut clusters = vec![];
                while let Some(starting_leaf) = block_leaves.pop() {
                    let mut cluster = vec![starting_leaf];
                    loop {
                        let mut index = 0;
                        let initial_cluster_len = cluster.len();
                        while index < cluster.len() {
                            self[cluster[index]]
                                .get_faces()
                                .iter()
                                .enumerate()
                                .for_each(|(face, &face_cell)| {
                                    if let Some(cell) = face_cell {
                                        if let Ok(spot) = block_leaves.binary_search(&cell) {
                                            if self[cell].get_block() == block {
                                                cluster.push(block_leaves.remove(spot));
                                            }
                                        } else if let Some(subcells) = self[cell].get_cells() {
                                            subcells_on_neighbor_face(face).into_iter().for_each(
                                                |subcell| {
                                                    if let Ok(spot) = block_leaves
                                                        .binary_search(&subcells[subcell])
                                                        && self[subcells[subcell]].get_block()
                                                            == block
                                                    {
                                                        cluster.push(block_leaves.remove(spot));
                                                    }
                                                },
                                            )
                                        }
                                    }
                                });
                            index += 1;
                        }
                        index = 0;
                        while index < cluster.len() {
                            if let Some([parent, subcell]) = supercells[cluster[index]] {
                                self[parent].get_faces().iter().enumerate().for_each(
                                    |(face, &face_cell)| {
                                        if let Some(cell) = face_cell
                                            && subcells_on_own_face_contains(face, subcell)
                                            && let Ok(spot) = block_leaves.binary_search(&cell)
                                            && self[cell].get_block() == block
                                        {
                                            cluster.push(block_leaves.remove(spot));
                                        }
                                    },
                                );
                            }
                            index += 1;
                        }
                        if cluster.len() == initial_cluster_len {
                            break;
                        }
                    }
                    clusters.push(cluster);
                }
                clusters
            })
            .collect();
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mClusters creation\x1b[0m {:?} ",
            time.elapsed()
        );
        clusters
    }
    fn cell_contains_leaves<'a>(&self, cell: &'a Cell) -> Option<(&'a Indices, &'a Faces)> {
        if let Some(cell_subcells) = cell.get_cells() {
            if self.just_leaves(cell_subcells) {
                Some((cell_subcells, cell.get_faces()))
            } else {
                None
            }
        } else {
            None
        }
    }
    fn cell_subcells_contain_cells(
        &self,
        cell: &Cell,
        face_index: usize,
    ) -> Option<SubSubCellsFace> {
        if let Some(cell_subcells) = cell.get_cells() {
            let subcell_indices = subcells_on_neighbor_face(face_index);
            if subcell_indices
                .iter()
                .all(|&subcell_index| self[cell_subcells[subcell_index]].is_not_leaf())
            {
                let subsubcells: SubSubCellsFace = subcell_indices
                    .iter()
                    .flat_map(|subcell_a| {
                        subcell_indices.iter().map(|&subcell_b| {
                            self[cell_subcells[*subcell_a]].get_cells().unwrap()[subcell_b]
                        })
                    })
                    .collect::<Vec<usize>>()
                    .try_into()
                    .unwrap();
                Some(subsubcells)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn cell_subcells_contain_leaves(
        &self,
        cell: &Cell,
        face_index: usize,
    ) -> Option<SubSubCellsFace> {
        self.cell_subcells_contain_cells(cell, face_index)
            .filter(|&subsubcells| {
                subsubcells
                    .iter()
                    .all(|&subsubcell| self[subsubcell].is_leaf())
            })
    }
    fn cell_subcell_contains_leaves(
        &self,
        cell: &Cell,
        face_index: usize,
        subsubcell_index: usize,
    ) -> Option<SubSubCellsFace> {
        self.cell_subcells_contain_cells(cell, face_index)
            .filter(|&subsubcells| self[subsubcells[subsubcell_index]].is_leaf())
    }
    pub fn defeature(&mut self, min_num_voxels: usize) {
        //
        // Should cells of a reassigned cluster be reassigned one at a time instead?
        //
        // Do the clusters need to be updated each time another changes?
        // In case a cluster inherits the reassigned cluster and becomes large enough?
        //
        // Still may not understand why `blocks` could be empty below.
        //
        let mut block = 0;
        let mut blocks = vec![];
        let mut clusters;
        let mut counts: Vec<usize> = vec![];
        let mut defeatured;
        let mut face_block = 0;
        let mut neighbor_block = 0;
        let mut new_block = 0;
        let mut protruded;
        let mut unique_blocks = vec![];
        let mut volumes: Vec<usize>;
        let supercells = self.supercells();
        #[allow(unused_variables)]
        for iteration in 1.. {
            clusters = self.clusters(Some(&vec![PADDING]), Some(&supercells));
            #[cfg(feature = "profile")]
            let time = Instant::now();
            volumes = clusters
                .iter()
                .map(|cluster| {
                    cluster
                        .iter()
                        .map(|&cell| self[cell].get_lngth().pow(NSD as u32) as usize)
                        .sum()
                })
                .collect();
            defeatured = volumes.iter().all(|volume| volume >= &min_num_voxels);
            if !defeatured {
                clusters
                    .iter()
                    .zip(volumes)
                    .filter(|(_, volume)| volume < &min_num_voxels)
                    .for_each(|(cluster, _)| {
                        block = self[cluster[0]].get_block();
                        blocks = cluster
                            .iter()
                            .flat_map(|&cell| {
                                self[cell]
                                    .get_faces()
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(face, &face_cell)| {
                                        if let Some(neighbor) = face_cell {
                                            if let Some(subcells) = self[neighbor].get_cells() {
                                                Some(
                                                    subcells_on_neighbor_face(face)
                                                        .into_iter()
                                                        .filter_map(|subcell| {
                                                            face_block =
                                                                self[subcells[subcell]].get_block();
                                                            if face_block != block {
                                                                Some(face_block)
                                                            } else {
                                                                None
                                                            }
                                                        })
                                                        .collect(),
                                                )
                                            } else {
                                                face_block = self[neighbor].get_block();
                                                if face_block != block {
                                                    Some(vec![face_block])
                                                } else {
                                                    None
                                                }
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<Blocks>>()
                            })
                            .chain(cluster.iter().filter_map(|&cell| {
                                if let Some([parent, subcell]) = supercells[cell] {
                                    Some(
                                        self[parent]
                                            .get_faces()
                                            .iter()
                                            .enumerate()
                                            .filter_map(|(face, face_cell)| {
                                                if let Some(neighbor_cell) = face_cell {
                                                    if self[*neighbor_cell].is_leaf()
                                                        && subcells_on_own_face_contains(
                                                            face, subcell,
                                                        )
                                                    {
                                                        neighbor_block =
                                                            self[*neighbor_cell].get_block();
                                                        if neighbor_block != block {
                                                            Some(neighbor_block)
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect(),
                                    )
                                } else {
                                    None
                                }
                            }))
                            .collect::<Vec<Blocks>>()
                            .into_iter()
                            .flatten()
                            .collect();
                        unique_blocks = blocks.to_vec();
                        unique_blocks.sort();
                        unique_blocks.dedup();
                        counts = unique_blocks
                            .iter()
                            .map(|unique_block| {
                                blocks.iter().filter(|&block| block == unique_block).count()
                            })
                            .collect();
                        if !blocks.is_empty() {
                            new_block = unique_blocks[counts
                                .iter()
                                .position(|count| {
                                    count == counts.iter().max().expect("maximum not found")
                                })
                                .expect("position of maximum not found")];
                            cluster
                                .iter()
                                .for_each(|&cell| self[cell].block = Some(new_block));
                        }
                    });
            }
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mDefeaturing iteration {}\x1b[0m {:?} ",
                iteration,
                time.elapsed()
            );
            protruded = self.protrusions(&supercells);
            if defeatured && protruded {
                return;
            }
        }
    }
    pub fn from_finite_elements<const M: usize, const N: usize, T>(
        finite_elements: T,
        grid: usize,
        size: f64,
    ) -> Self
    where
        T: FiniteElementMethods<M, N>,
    {
        let mut blocks: Blocks = finite_elements
            .get_element_blocks()
            .iter()
            .flat_map(|&block| repeat_n(block, grid.pow(3)))
            .collect();
        let mut samples = finite_elements.interior_points(grid);
        let mut exterior_face_samples = finite_elements.exterior_faces_interior_points(grid);
        blocks.extend(vec![PADDING; exterior_face_samples.len()]);
        samples.append(&mut exterior_face_samples);
        let mut tree = octree_from_bounding_cube(&mut samples, size);
        let mut index = 0;
        while index < tree.len() {
            if let Some(block) = tree[index].homogeneous_coordinates(&blocks, &samples) {
                tree[index].block = Some(block);
            } else {
                tree.subdivide(index)
            }
            index += 1;
        }
        tree
    }
    fn just_leaves(&self, cells: &[usize]) -> bool {
        cells.iter().all(|&subcell| self[subcell].is_leaf())
    }
    pub const fn nel(&self) -> Nel {
        self.nel
    }
    pub fn octree_into_finite_elements(
        // will eventually delete this method
        self,
        remove: Option<Blocks>,
        scale: Scale,
        translate: Translate,
    ) -> Result<HexahedralFiniteElements, String> {
        let mut x_min = 0.0;
        let mut y_min = 0.0;
        let mut z_min = 0.0;
        let mut x_val = 0.0;
        let mut y_val = 0.0;
        let mut z_val = 0.0;
        let mut removed_data = remove.unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        // removed_data.push(PADDING);
        let num_elements = self
            .iter()
            .filter(|cell| removed_data.binary_search(&cell.get_block()).is_err())
            .count();
        let mut element_blocks = vec![0; num_elements];
        let mut element_node_connectivity = vec![from_fn(|_| 0); num_elements];
        let mut nodal_coordinates: Coordinates = (0..num_elements * HEX)
            .map(|_| Coordinate::zero())
            .collect();
        let mut index = 0;
        self.iter()
            .filter(|cell| removed_data.binary_search(&cell.get_block()).is_err())
            .zip(
                element_blocks
                    .iter_mut()
                    .zip(element_node_connectivity.iter_mut()),
            )
            .for_each(|(cell, (block, connectivity))| {
                *block = cell.get_block();
                *connectivity = from_fn(|n| n + index);
                x_min = *cell.get_min_x() as f64 * scale.x() + translate.x();
                y_min = *cell.get_min_y() as f64 * scale.y() + translate.y();
                z_min = *cell.get_min_z() as f64 * scale.z() + translate.z();
                x_val = (cell.get_min_x() + cell.get_lngth()) as f64 * scale.x() + translate.x();
                y_val = (cell.get_min_y() + cell.get_lngth()) as f64 * scale.y() + translate.y();
                z_val = (cell.get_min_z() + cell.get_lngth()) as f64 * scale.z() + translate.z();
                nodal_coordinates[index] = Coordinate::new([x_min, y_min, z_min]);
                nodal_coordinates[index + 1] = Coordinate::new([x_val, y_min, z_min]);
                nodal_coordinates[index + 2] = Coordinate::new([x_val, y_val, z_min]);
                nodal_coordinates[index + 3] = Coordinate::new([x_min, y_val, z_min]);
                nodal_coordinates[index + 4] = Coordinate::new([x_min, y_min, z_val]);
                nodal_coordinates[index + 5] = Coordinate::new([x_val, y_min, z_val]);
                nodal_coordinates[index + 6] = Coordinate::new([x_val, y_val, z_val]);
                nodal_coordinates[index + 7] = Coordinate::new([x_min, y_val, z_val]);
                index += HEX;
            });
        Ok(HexahedralFiniteElements::from((
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        )))
    }
    pub fn pair(&mut self) -> bool {
        // #[cfg(feature = "profile")]
        // let time = Instant::now();
        let mut block = 0;
        let mut index = 0;
        let mut paired_already = true;
        let mut subsubcells: Vec<bool>;
        while index < self.len() {
            if let Some(subcells) = self[index].cells {
                subsubcells = subcells
                    .into_iter()
                    .map(|subcell| self[subcell].is_not_leaf())
                    .collect();
                if subsubcells.iter().any(|&subsubcell| subsubcell)
                    && !subsubcells.iter().all(|&subsubcell| subsubcell)
                {
                    subcells
                        .into_iter()
                        .filter(|&subcell| self[subcell].cells.is_none())
                        .collect::<Vec<usize>>()
                        .into_iter()
                        .for_each(|subcell| {
                            block = self[subcell].get_block();
                            paired_already = false;
                            self.subdivide(subcell);
                            self.iter_mut()
                                .rev()
                                .take(NUM_OCTANTS)
                                .for_each(|cell| cell.block = Some(block))
                        })
                }
            }
            index += 1;
        }
        // #[cfg(feature = "profile")]
        // println!(
        //     "           \x1b[1;93m  Pairing hanging nodes\x1b[0m {:?} ",
        //     time.elapsed()
        // );
        paired_already
    }
    pub fn parameters(self) -> (Remove, Scale, Translate) {
        (self.remove, self.scale, self.translate)
    }
    fn protrusions(&mut self, supercells: &Supercells) -> bool {
        let mut blocks = vec![];
        let mut complete = true;
        let mut counts: Vec<usize> = vec![];
        let mut new_block = 0;
        let mut protrusions: Vec<(usize, Blocks)>;
        let mut unique_blocks = vec![];
        #[allow(unused_variables)]
        for iteration in 1.. {
            #[cfg(feature = "profile")]
            let time = Instant::now();
            protrusions = self
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.is_voxel())
                .flat_map(|(voxel_cell_index, voxel_cell)| {
                    blocks = voxel_cell
                        .get_faces()
                        .iter()
                        .enumerate()
                        .flat_map(|(face_index, &face)| {
                            if let Some(face_cell_index) = face {
                                Some(self[face_cell_index].get_block())
                            } else if let Some([parent, _]) = supercells[voxel_cell_index] {
                                self[parent].get_faces()[face_index]
                                    .map(|neighbor| self[neighbor].get_block())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if blocks
                        .iter()
                        .filter(|&&face_block| voxel_cell.get_block() != face_block)
                        .count()
                        >= 5
                    {
                        Some((voxel_cell_index, blocks.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            if !protrusions.is_empty() {
                complete = false;
                protrusions.iter().for_each(|(voxel_cell_index, blocks)| {
                    unique_blocks = blocks.to_vec();
                    unique_blocks.sort();
                    unique_blocks.dedup();
                    counts = unique_blocks
                        .iter()
                        .map(|unique_block| {
                            blocks.iter().filter(|&block| block == unique_block).count()
                        })
                        .collect();
                    new_block = unique_blocks[counts
                        .iter()
                        .position(|count| count == counts.iter().max().expect("maximum not found"))
                        .expect("position of maximum not found")];
                    self[*voxel_cell_index].block = Some(new_block)
                })
            }
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mProtrusions iteration {}\x1b[0m {:?} ",
                iteration,
                time.elapsed()
            );
            if protrusions.is_empty() {
                break;
            }
        }
        complete
    }
    pub fn prune(&mut self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        self.retain(|cell| cell.is_leaf());
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mPruning octree\x1b[0m {:?} ",
            time.elapsed()
        );
    }
    pub fn remove(&self) -> &Remove {
        &self.remove
    }
    pub fn scale(&self) -> &Scale {
        &self.scale
    }
    fn subdivide(&mut self, index: usize) {
        assert!(self[index].is_leaf());
        let new_indices = from_fn(|n| self.len() + n);
        let mut new_cells = self[index].subdivide(new_indices);
        self[index]
            .get_faces()
            .clone()
            .iter()
            .enumerate()
            .for_each(|(face, &face_cell)| {
                if let Some(neighbor) = face_cell
                    && let Some(kids) = self[neighbor].cells
                {
                    subcells_on_own_face(face)
                        .iter()
                        .zip(subcells_on_neighbor_face(face).iter())
                        .for_each(|(&subcell, &neighbor_subcell)| {
                            new_cells[subcell].faces[face] = Some(kids[neighbor_subcell]);
                            self[kids[neighbor_subcell]].faces[mirror_face(face)] =
                                Some(new_indices[subcell]);
                        });
                }
            });
        self.extend(new_cells);
    }
    fn supercells(&self) -> Supercells {
        let (max_leaf_id, _) = self
            .iter()
            .enumerate()
            .rfind(|(_, cell)| cell.is_leaf())
            .unwrap();
        let mut supercells = vec![None; max_leaf_id + 1];
        self.iter()
            .enumerate()
            .filter_map(|(parent_index, cell)| {
                cell.get_cells()
                    .as_ref()
                    .map(|subcells| (parent_index, subcells))
            })
            .for_each(|(parent_index, subcells)| {
                if subcells
                    .iter()
                    .filter(|&&subcell| self[subcell].get_cells().is_some())
                    .count()
                    == 0
                {
                    subcells
                        .iter()
                        .enumerate()
                        .for_each(|(subcell_index, &subcell)| {
                            supercells[subcell] = Some([parent_index, subcell_index])
                        })
                }
            });
        supercells
    }
    pub fn translate(&self) -> &Translate {
        &self.translate
    }
}

impl From<Voxels> for Octree {
    fn from(voxels: Voxels) -> Octree {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let (data_voxels, remove, scale, translate) = voxels.data();
        let mut nel_i = 0;
        let nel_padded = data_voxels
            .shape()
            .iter()
            .map(|nel_0| {
                nel_i = *nel_0;
                while (nel_i & (nel_i - 1)) != 0 {
                    nel_i += 1
                }
                nel_i
            })
            .max()
            .unwrap();
        let nel = Nel::from([nel_padded; NSD]);
        let mut data = VoxelData::from(nel);
        data.iter_mut().for_each(|entry| *entry = PADDING);
        if data_voxels.iter().any(|entry| entry == &PADDING) {
            panic!("Segmentation cannot use 255 as an ID with octree padding.")
        }
        data.axis_iter_mut(Axis(2))
            .zip(data_voxels.axis_iter(Axis(2)))
            .for_each(|(mut data_i, data_voxels_i)| {
                data_i
                    .axis_iter_mut(Axis(1))
                    .zip(data_voxels_i.axis_iter(Axis(1)))
                    .for_each(|(mut data_ij, data_voxels_ij)| {
                        data_ij
                            .iter_mut()
                            .zip(data_voxels_ij.iter())
                            .for_each(|(data_ijk, data_voxels_ijk)| *data_ijk = *data_voxels_ijk)
                    })
            });
        let nel_min = nel.iter().min().unwrap();
        let lngth = *nel_min as u16;
        let mut tree = Octree {
            nel,
            octree: vec![],
            remove,
            scale,
            translate,
        };
        (0..(nel.x() / nel_min)).for_each(|i| {
            (0..(nel.y() / nel_min)).for_each(|j| {
                (0..(nel.z() / nel_min)).for_each(|k| {
                    tree.push(Cell {
                        block: None,
                        cells: None,
                        faces: [None; NUM_FACES],
                        lngth,
                        min_x: lngth * i as u16,
                        min_y: lngth * j as u16,
                        min_z: lngth * k as u16,
                    })
                })
            })
        });
        let mut index = 0;
        while index < tree.len() {
            if let Some(block) = tree[index].homogeneous(&data) {
                tree[index].block = Some(block)
            } else {
                tree.subdivide(index)
            }
            index += 1;
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mOctree initialization\x1b[0m {:?} ",
            time.elapsed()
        );
        tree
    }
}
