#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    fem::{
        Blocks, FiniteElementMethods, HexahedralFiniteElements, TriangularFiniteElements, HEX,
        NODE_NUMBERING_OFFSET,
    },
    voxel::{Nel, Scale, Translate, VoxelData, Voxels},
    Coordinate, Coordinates, NSD,
};
use conspire::math::{TensorArray, TensorRank1Vec, TensorVec};
use ndarray::{s, Axis};
use std::array::from_fn;

pub const PADDING: u8 = 255;

const NUM_FACES: usize = 6;
const NUM_OCTANTS: usize = 8;
const NUM_NODES_FACE: usize = 4;
const NUM_SUBCELLS_FACE: usize = 4;

type SubcellsOnFace = [usize; NUM_SUBCELLS_FACE];
const SUBCELLS_ON_OWN_FACE_0: SubcellsOnFace = [0, 1, 4, 5];
const SUBCELLS_ON_OWN_FACE_1: SubcellsOnFace = [1, 3, 5, 7];
const SUBCELLS_ON_OWN_FACE_2: SubcellsOnFace = [2, 3, 6, 7];
const SUBCELLS_ON_OWN_FACE_3: SubcellsOnFace = [0, 2, 4, 6];
const SUBCELLS_ON_OWN_FACE_4: SubcellsOnFace = [0, 1, 2, 3];
const SUBCELLS_ON_OWN_FACE_5: SubcellsOnFace = [4, 5, 6, 7];

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

type Cells = [Cell; NUM_OCTANTS];
type Edge = [usize; 2];
type Edges = Vec<Edge>;
type Faces = [Option<usize>; NUM_FACES];
type Indices = [usize; NUM_OCTANTS];

/// The octree type.
pub type Octree = Vec<Cell>;

type Cluster = Vec<usize>;
type Clusters = Vec<Cluster>;
type Supercells = Vec<Option<[usize; 2]>>;

/// Methods for trees such as quadtrees or octrees.
pub trait Tree {
    fn balance(&mut self, strong: bool);
    fn boundaries(&mut self, nel_padded: &Nel);
    fn clusters(&self, remove: &Option<Blocks>, supercells: Option<&Supercells>) -> Clusters;
    fn defeature(&mut self, min_num_voxels: usize);
    fn from_voxels(voxels: Voxels) -> (Nel, Self);
    fn octree_into_finite_elements(
        self,
        remove: Option<Blocks>,
        scale: Scale,
        translate: Translate,
    ) -> Result<HexahedralFiniteElements, String>;
    fn pair(&mut self);
    fn protrusions(&mut self, supercells: &Supercells) -> bool;
    fn prune(&mut self);
    fn subdivide(&mut self, index: usize);
    fn supercells(&self) -> Supercells;
}

/// Methods for converting trees into finite elements.
pub trait IntoFiniteElements<F> {
    fn into_finite_elements(
        self,
        nel: Nel,
        remove: Option<Blocks>,
        scale: Scale,
        translate: Translate,
    ) -> Result<F, String>;
}

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
    pub fn get_block(&self) -> u8 {
        if let Some(block) = self.block {
            block
        } else {
            panic!("Called get_block() on a non-leaf cell.")
        }
    }
    pub fn get_cells(&self) -> &Option<Indices> {
        &self.cells
    }
    pub fn get_faces(&self) -> &Faces {
        &self.faces
    }
    pub fn get_lngth(&self) -> &u16 {
        &self.lngth
    }
    pub fn get_min_x(&self) -> &u16 {
        &self.min_x
    }
    pub fn get_min_y(&self) -> &u16 {
        &self.min_y
    }
    pub fn get_min_z(&self) -> &u16 {
        &self.min_z
    }
    pub fn get_max_x(&self) -> u16 {
        self.min_x + self.lngth
    }
    pub fn get_max_y(&self) -> u16 {
        self.min_y + self.lngth
    }
    pub fn get_max_z(&self) -> u16 {
        self.min_z + self.lngth
    }
    pub fn get_nodal_indices_face(&self, face_index: &usize) -> [[u16; NSD]; NUM_NODES_FACE] {
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
        let mut materials: Blocks = contained.iter().cloned().collect();
        materials.dedup();
        if materials.len() == 1 {
            Some(materials[0])
        } else {
            None
        }
    }
    pub fn is_face_on_octree_boundary(&self, face_index: &usize, nel: &Nel) -> bool {
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
    pub fn is_leaf(&self) -> bool {
        self.get_cells().is_none()
    }
    pub fn is_voxel(&self) -> bool {
        self.lngth == 1
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

impl Tree for Octree {
    fn balance(&mut self, strong: bool) {
        let mut balanced;
        let mut block;
        let mut edges: [bool; 8];
        let mut index;
        let mut subdivide;
        #[allow(unused_variables)]
        for iteration in 1.. {
            balanced = true;
            index = 0;
            subdivide = false;
            #[cfg(feature = "profile")]
            let time = Instant::now();
            while index < self.len() {
                if !self[index].is_voxel() && self[index].cells.is_none() {
                    'faces: for (face, face_cell) in self[index].get_faces().iter().enumerate() {
                        if let Some(neighbor) = face_cell {
                            if let Some(kids) = self[*neighbor].cells {
                                edges = from_fn(|_| false);
                                if match face {
                                    0 => {
                                        if strong {
                                            if let Some(edge_cell) = self[kids[3]].get_faces()[1] {
                                                edges[0] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[0] = false
                                            }
                                            if let Some(edge_cell) = self[kids[7]].get_faces()[1] {
                                                edges[1] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[1] = false
                                            }
                                            if let Some(edge_cell) = self[kids[6]].get_faces()[5] {
                                                edges[2] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[2] = false
                                            }
                                            if let Some(edge_cell) = self[kids[7]].get_faces()[5] {
                                                edges[3] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[3] = false
                                            }
                                            if let Some(edge_cell) = self[kids[2]].get_faces()[3] {
                                                edges[4] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[4] = false
                                            }
                                            if let Some(edge_cell) = self[kids[6]].get_faces()[3] {
                                                edges[5] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[5] = false
                                            }
                                            if let Some(edge_cell) = self[kids[2]].get_faces()[4] {
                                                edges[6] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[6] = false
                                            }
                                            if let Some(edge_cell) = self[kids[3]].get_faces()[4] {
                                                edges[7] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[7] = false
                                            }
                                        }
                                        edges.into_iter().any(|edge| edge)
                                            || self[kids[2]].cells.is_some()
                                            || self[kids[3]].cells.is_some()
                                            || self[kids[6]].cells.is_some()
                                            || self[kids[7]].cells.is_some()
                                    }
                                    1 => {
                                        if strong {
                                            if let Some(edge_cell) = self[kids[2]].get_faces()[2] {
                                                edges[0] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[0] = false
                                            }
                                            if let Some(edge_cell) = self[kids[6]].get_faces()[2] {
                                                edges[1] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[1] = false
                                            }
                                            if let Some(edge_cell) = self[kids[4]].get_faces()[5] {
                                                edges[2] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[2] = false
                                            }
                                            if let Some(edge_cell) = self[kids[6]].get_faces()[5] {
                                                edges[3] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[3] = false
                                            }
                                            if let Some(edge_cell) = self[kids[0]].get_faces()[0] {
                                                edges[4] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[4] = false
                                            }
                                            if let Some(edge_cell) = self[kids[4]].get_faces()[0] {
                                                edges[5] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[5] = false
                                            }
                                            if let Some(edge_cell) = self[kids[0]].get_faces()[4] {
                                                edges[6] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[6] = false
                                            }
                                            if let Some(edge_cell) = self[kids[2]].get_faces()[4] {
                                                edges[7] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[7] = false
                                            }
                                        }
                                        edges.into_iter().any(|edge| edge)
                                            || self[kids[0]].cells.is_some()
                                            || self[kids[2]].cells.is_some()
                                            || self[kids[4]].cells.is_some()
                                            || self[kids[6]].cells.is_some()
                                    }
                                    2 => {
                                        if strong {
                                            if let Some(edge_cell) = self[kids[0]].get_faces()[3] {
                                                edges[0] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[0] = false
                                            }
                                            if let Some(edge_cell) = self[kids[4]].get_faces()[3] {
                                                edges[1] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[1] = false
                                            }
                                            if let Some(edge_cell) = self[kids[4]].get_faces()[5] {
                                                edges[2] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[2] = false
                                            }
                                            if let Some(edge_cell) = self[kids[5]].get_faces()[5] {
                                                edges[3] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[3] = false
                                            }
                                            if let Some(edge_cell) = self[kids[1]].get_faces()[1] {
                                                edges[4] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[4] = false
                                            }
                                            if let Some(edge_cell) = self[kids[5]].get_faces()[1] {
                                                edges[5] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[5] = false
                                            }
                                            if let Some(edge_cell) = self[kids[0]].get_faces()[4] {
                                                edges[6] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[6] = false
                                            }
                                            if let Some(edge_cell) = self[kids[1]].get_faces()[4] {
                                                edges[7] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[7] = false
                                            }
                                        }
                                        edges.into_iter().any(|edge| edge)
                                            || self[kids[0]].cells.is_some()
                                            || self[kids[1]].cells.is_some()
                                            || self[kids[4]].cells.is_some()
                                            || self[kids[5]].cells.is_some()
                                    }
                                    3 => {
                                        if strong {
                                            if let Some(edge_cell) = self[kids[1]].get_faces()[0] {
                                                edges[0] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[0] = false
                                            }
                                            if let Some(edge_cell) = self[kids[5]].get_faces()[0] {
                                                edges[1] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[1] = false
                                            }
                                            if let Some(edge_cell) = self[kids[5]].get_faces()[5] {
                                                edges[2] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[2] = false
                                            }
                                            if let Some(edge_cell) = self[kids[7]].get_faces()[5] {
                                                edges[3] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[3] = false
                                            }
                                            if let Some(edge_cell) = self[kids[3]].get_faces()[2] {
                                                edges[4] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[4] = false
                                            }
                                            if let Some(edge_cell) = self[kids[7]].get_faces()[2] {
                                                edges[5] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[5] = false
                                            }
                                            if let Some(edge_cell) = self[kids[1]].get_faces()[4] {
                                                edges[6] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[6] = false
                                            }
                                            if let Some(edge_cell) = self[kids[3]].get_faces()[4] {
                                                edges[7] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[7] = false
                                            }
                                        }
                                        edges.into_iter().any(|edge| edge)
                                            || self[kids[1]].cells.is_some()
                                            || self[kids[3]].cells.is_some()
                                            || self[kids[5]].cells.is_some()
                                            || self[kids[7]].cells.is_some()
                                    }
                                    4 => {
                                        if strong {
                                            if let Some(edge_cell) = self[kids[5]].get_faces()[1] {
                                                edges[0] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[0] = false
                                            }
                                            if let Some(edge_cell) = self[kids[7]].get_faces()[1] {
                                                edges[1] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[1] = false
                                            }
                                            if let Some(edge_cell) = self[kids[6]].get_faces()[2] {
                                                edges[2] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[2] = false
                                            }
                                            if let Some(edge_cell) = self[kids[7]].get_faces()[2] {
                                                edges[3] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[3] = false
                                            }
                                            if let Some(edge_cell) = self[kids[4]].get_faces()[3] {
                                                edges[4] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[4] = false
                                            }
                                            if let Some(edge_cell) = self[kids[6]].get_faces()[3] {
                                                edges[5] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[5] = false
                                            }
                                            if let Some(edge_cell) = self[kids[4]].get_faces()[0] {
                                                edges[6] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[6] = false
                                            }
                                            if let Some(edge_cell) = self[kids[5]].get_faces()[0] {
                                                edges[7] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[7] = false
                                            }
                                        }
                                        edges.into_iter().any(|edge| edge)
                                            || self[kids[4]].cells.is_some()
                                            || self[kids[5]].cells.is_some()
                                            || self[kids[6]].cells.is_some()
                                            || self[kids[7]].cells.is_some()
                                    }
                                    5 => {
                                        if strong {
                                            if let Some(edge_cell) = self[kids[1]].get_faces()[1] {
                                                edges[0] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[0] = false
                                            }
                                            if let Some(edge_cell) = self[kids[3]].get_faces()[1] {
                                                edges[1] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[1] = false
                                            }
                                            if let Some(edge_cell) = self[kids[2]].get_faces()[2] {
                                                edges[2] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[2] = false
                                            }
                                            if let Some(edge_cell) = self[kids[3]].get_faces()[2] {
                                                edges[3] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[3] = false
                                            }
                                            if let Some(edge_cell) = self[kids[0]].get_faces()[3] {
                                                edges[4] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[4] = false
                                            }
                                            if let Some(edge_cell) = self[kids[2]].get_faces()[3] {
                                                edges[5] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[5] = false
                                            }
                                            if let Some(edge_cell) = self[kids[0]].get_faces()[0] {
                                                edges[6] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[6] = false
                                            }
                                            if let Some(edge_cell) = self[kids[1]].get_faces()[0] {
                                                edges[7] = self[edge_cell].cells.is_some()
                                            } else {
                                                edges[7] = false
                                            }
                                        }
                                        edges.into_iter().any(|edge| edge)
                                            || self[kids[0]].cells.is_some()
                                            || self[kids[1]].cells.is_some()
                                            || self[kids[2]].cells.is_some()
                                            || self[kids[3]].cells.is_some()
                                    }
                                    _ => panic!(),
                                } {
                                    subdivide = true;
                                    break 'faces;
                                }
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
                        subdivide = false;
                    }
                }
                index += 1;
            }
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mBalancing iteration {}\x1b[0m {:?} ",
                iteration,
                time.elapsed()
            );
            if balanced {
                break;
            }
        }
    }
    fn boundaries(&mut self, nel_padded: &Nel) {
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
                                cell.is_face_on_octree_boundary(&face_index, nel_padded)
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
            if iteration == 1 {
                println!(
                    "           \x1b[1;93m⤷ Boundaries iteration {}\x1b[0m {:?} ",
                    iteration,
                    time.elapsed()
                );
            } else {
                println!(
                    "             \x1b[1;93mBoundaries iteration {}\x1b[0m {:?} ",
                    iteration,
                    time.elapsed()
                );
            }
            if boundaries {
                break;
            }
        }
        self.balance(true);
    }
    fn clusters(&self, remove: &Option<Blocks>, supercells_opt: Option<&Supercells>) -> Clusters {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut removed_data = remove.clone().unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        removed_data.push(PADDING);
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
        let mut clusters = vec![];
        let mut complete = false;
        let mut index = 0;
        let mut leaf = 0;
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
        blocks
            .into_iter()
            .enumerate()
            .for_each(|(block_index, block)| {
                let block_leaves = &mut leaves[block_index];
                while let Some(starting_leaf) = block_leaves.pop() {
                    let mut cluster = vec![starting_leaf];
                    loop {
                        complete = true;
                        index = 0;
                        while index < cluster.len() {
                            leaf = cluster[index];
                            self[leaf].get_faces().iter().enumerate().for_each(
                                |(face, face_cell)| {
                                    if let Some(cell) = face_cell {
                                        if let Ok(spot) = block_leaves.binary_search(cell) {
                                            if self[*cell].get_block() == block {
                                                block_leaves.remove(spot);
                                                cluster.push(*cell);
                                                complete = false;
                                            }
                                        } else if let Some(subcells) = self[*cell].get_cells() {
                                            subcells_on_neighbor_face(face).into_iter().for_each(
                                                |subcell| {
                                                    if let Ok(spot) = block_leaves
                                                        .binary_search(&subcells[subcell])
                                                    {
                                                        if self[subcells[subcell]].get_block()
                                                            == block
                                                        {
                                                            block_leaves.remove(spot);
                                                            cluster.push(subcells[subcell]);
                                                            complete = false;
                                                        }
                                                    }
                                                },
                                            )
                                        }
                                    }
                                },
                            );
                            index += 1;
                        }
                        index = 0;
                        while index < cluster.len() {
                            leaf = cluster[index];
                            if let Some([parent, subcell]) = supercells[leaf] {
                                self[parent].get_faces().iter().enumerate().for_each(
                                    |(face, face_cell)| {
                                        if let Some(cell) = face_cell {
                                            if subcells_on_own_face(face).contains(&subcell) {
                                                if let Ok(spot) = block_leaves.binary_search(cell) {
                                                    if self[*cell].get_block() == block {
                                                        block_leaves.remove(spot);
                                                        cluster.push(*cell);
                                                        complete = false;
                                                    }
                                                }
                                            }
                                        }
                                    },
                                );
                            }
                            index += 1;
                        }
                        if complete {
                            break;
                        }
                    }
                    clusters.push(cluster);
                }
            });
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mClusters creation\x1b[0m {:?} ",
            time.elapsed()
        );
        clusters
    }
    fn defeature(&mut self, min_num_voxels: usize) {
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
            clusters = self.clusters(&None, Some(&supercells));
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
                                                        && subcells_on_own_face(face)
                                                            .contains(&subcell)
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
    fn from_voxels(voxels: Voxels) -> (Nel, Self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let data_voxels = voxels.get_data();
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
        let mut tree = vec![];
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
            "           \x1b[1;93m⤷ Octree initialization\x1b[0m {:?} ",
            time.elapsed()
        );
        (nel, tree)
    }
    fn octree_into_finite_elements(
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
        removed_data.push(PADDING);
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
                *connectivity = from_fn(|n| n + index + NODE_NUMBERING_OFFSET);
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
        Ok(HexahedralFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ))
    }
    fn pair(&mut self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut block = 0;
        let mut index = 0;
        let mut subsubcells: Vec<bool>;
        while index < self.len() {
            if let Some(subcells) = self[index].cells {
                subsubcells = subcells
                    .into_iter()
                    .map(|subcell| self[subcell].cells.is_some())
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
        #[cfg(feature = "profile")]
        println!(
            "           \x1b[1;93m  Pairing hanging nodes\x1b[0m {:?} ",
            time.elapsed()
        );
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
    fn prune(&mut self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        self.retain(|cell| cell.is_leaf());
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mPruning octree\x1b[0m {:?} ",
            time.elapsed()
        );
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
            .for_each(|(face, face_cell)| {
                if let Some(neighbor) = face_cell {
                    if let Some(kids) = self[*neighbor].cells {
                        match face {
                            0 => {
                                new_cells[0].faces[0] = Some(kids[2]);
                                new_cells[1].faces[0] = Some(kids[3]);
                                new_cells[4].faces[0] = Some(kids[6]);
                                new_cells[5].faces[0] = Some(kids[7]);
                                self[kids[2]].faces[2] = Some(new_indices[0]);
                                self[kids[3]].faces[2] = Some(new_indices[1]);
                                self[kids[6]].faces[2] = Some(new_indices[4]);
                                self[kids[7]].faces[2] = Some(new_indices[5]);
                            }
                            1 => {
                                new_cells[1].faces[1] = Some(kids[0]);
                                new_cells[3].faces[1] = Some(kids[2]);
                                new_cells[5].faces[1] = Some(kids[4]);
                                new_cells[7].faces[1] = Some(kids[6]);
                                self[kids[0]].faces[3] = Some(new_indices[1]);
                                self[kids[2]].faces[3] = Some(new_indices[3]);
                                self[kids[4]].faces[3] = Some(new_indices[5]);
                                self[kids[6]].faces[3] = Some(new_indices[7]);
                            }
                            2 => {
                                new_cells[2].faces[2] = Some(kids[0]);
                                new_cells[3].faces[2] = Some(kids[1]);
                                new_cells[6].faces[2] = Some(kids[4]);
                                new_cells[7].faces[2] = Some(kids[5]);
                                self[kids[0]].faces[0] = Some(new_indices[2]);
                                self[kids[1]].faces[0] = Some(new_indices[3]);
                                self[kids[4]].faces[0] = Some(new_indices[6]);
                                self[kids[5]].faces[0] = Some(new_indices[7]);
                            }
                            3 => {
                                new_cells[0].faces[3] = Some(kids[1]);
                                new_cells[2].faces[3] = Some(kids[3]);
                                new_cells[4].faces[3] = Some(kids[5]);
                                new_cells[6].faces[3] = Some(kids[7]);
                                self[kids[1]].faces[1] = Some(new_indices[0]);
                                self[kids[3]].faces[1] = Some(new_indices[2]);
                                self[kids[5]].faces[1] = Some(new_indices[4]);
                                self[kids[7]].faces[1] = Some(new_indices[6]);
                            }
                            4 => {
                                new_cells[0].faces[4] = Some(kids[4]);
                                new_cells[1].faces[4] = Some(kids[5]);
                                new_cells[2].faces[4] = Some(kids[6]);
                                new_cells[3].faces[4] = Some(kids[7]);
                                self[kids[4]].faces[5] = Some(new_indices[0]);
                                self[kids[5]].faces[5] = Some(new_indices[1]);
                                self[kids[6]].faces[5] = Some(new_indices[2]);
                                self[kids[7]].faces[5] = Some(new_indices[3]);
                            }
                            5 => {
                                new_cells[4].faces[5] = Some(kids[0]);
                                new_cells[5].faces[5] = Some(kids[1]);
                                new_cells[6].faces[5] = Some(kids[2]);
                                new_cells[7].faces[5] = Some(kids[3]);
                                self[kids[0]].faces[4] = Some(new_indices[4]);
                                self[kids[1]].faces[4] = Some(new_indices[5]);
                                self[kids[2]].faces[4] = Some(new_indices[6]);
                                self[kids[3]].faces[4] = Some(new_indices[7]);
                            }
                            _ => panic!(),
                        }
                    }
                }
            });
        self.extend(new_cells);
    }
    fn supercells(&self) -> Supercells {
        let (max_leaf_id, _) = self
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_leaf())
            .next_back()
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
}

impl IntoFiniteElements<TriangularFiniteElements> for Octree {
    fn into_finite_elements(
        mut self,
        nel_padded: Nel,
        remove: Option<Blocks>,
        scale: Scale,
        translate: Translate,
    ) -> Result<TriangularFiniteElements, String> {
        let mut removed_data = remove.clone().unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        removed_data.push(PADDING);
        self.boundaries(&nel_padded);
        let clusters = self.clusters(&remove, None);
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let blocks = clusters
            .iter()
            .map(|cluster: &Vec<usize>| self[cluster[0]].get_block())
            .collect::<Blocks>();
        let default_face_info = [None; NUM_FACES];
        let mut faces_info = default_face_info;
        let boundaries_cells_faces = blocks
            .iter()
            .zip(clusters.iter())
            .map(|(&block, cluster)| {
                cluster
                    .iter()
                    .filter(|&&cell| self[cell].is_voxel())
                    .filter_map(|&cell| {
                        faces_info = default_face_info;
                        faces_info
                            .iter_mut()
                            .enumerate()
                            .zip(self[cell].get_faces().iter())
                            .for_each(|((face_index, face_info), &face)| {
                                if let Some(face_cell) = face {
                                    if self[face_cell].get_block() != block {
                                        *face_info = Some(face_cell)
                                    }
                                } else if self[cell]
                                    .is_face_on_octree_boundary(&face_index, &nel_padded)
                                {
                                    *face_info = Some(usize::MAX)
                                }
                            });
                        if faces_info.iter().all(|face_info| face_info.is_none()) {
                            None
                        } else {
                            Some((cell, faces_info))
                        }
                    })
                    .collect()
            })
            .collect::<Vec<Vec<(usize, Faces)>>>();
        let mut max_cell_id = 0;
        let mut boundaries_face_from_cell = boundaries_cells_faces
            .iter()
            .map(|boundary_cells_faces| {
                (max_cell_id, _) = *boundary_cells_faces
                    .iter()
                    .max_by(|(cell_a, _), (cell_b, _)| cell_a.cmp(cell_b))
                    .unwrap();
                vec![[false; NUM_FACES]; max_cell_id + 1]
            })
            .collect::<Vec<Vec<[bool; NUM_FACES]>>>();
        max_cell_id = 0;
        boundaries_cells_faces
            .iter()
            .for_each(|boundary_cells_faces| {
                boundary_cells_faces.iter().for_each(|(cell, _)| {
                    if cell > &max_cell_id {
                        max_cell_id = *cell
                    }
                })
            });
        let mut boundary_from_cell = vec![None; max_cell_id + 1];
        boundaries_cells_faces
            .iter()
            .enumerate()
            .for_each(|(boundary, boundary_cells_faces)| {
                boundary_cells_faces
                    .iter()
                    .for_each(|(cell, _)| boundary_from_cell[*cell] = Some(boundary))
            });
        let mut cell_face_index = 0;
        let mut cell_faces = vec![vec![]; max_cell_id + 1];
        let mut face_blocks: Vec<u8> = vec![];
        let mut face_cells: Vec<usize> = vec![];
        let mut face_connectivity = [0; NUM_NODES_FACE];
        let mut faces_connectivity = vec![];
        let mut nodal_coordinates = Coordinates::zero(0);
        let mut node_new = 1;
        let nodes_len = (self[0].get_lngth() + 1) as usize;
        let mut nodes = vec![vec![vec![None::<usize>; nodes_len]; nodes_len]; nodes_len];
        (0..boundaries_cells_faces.len()).for_each(|boundary| {
            boundaries_cells_faces[boundary]
                .iter()
                .for_each(|(cell, faces)| {
                    faces.iter().enumerate().for_each(|(face_index, face)| {
                        if let Some(face_cell) = face {
                            if !boundaries_face_from_cell[boundary][*cell][face_index] {
                                boundaries_face_from_cell[boundary][*cell][face_index] = true;
                                #[allow(clippy::collapsible_if)]
                                if face_cell != &usize::MAX {
                                    if removed_data
                                        .binary_search(&self[*face_cell].get_block())
                                        .is_err()
                                    {
                                        if let Some(opposing_boundary) =
                                            boundary_from_cell[*face_cell]
                                        {
                                            boundaries_face_from_cell[opposing_boundary]
                                                [*face_cell][mirror_face(face_index)] = true;
                                        }
                                    }
                                }
                                self[*cell]
                                    .get_nodal_indices_face(&face_index)
                                    .iter()
                                    .zip(face_connectivity.iter_mut())
                                    .for_each(|(nodal_indices, face_node)| {
                                        if let Some(node) = nodes[nodal_indices[0] as usize]
                                            [nodal_indices[1] as usize]
                                            [nodal_indices[2] as usize]
                                        {
                                            *face_node = node
                                        } else {
                                            nodal_coordinates.push(Coordinate::new([
                                                nodal_indices[0] as f64 * scale.x() + translate.x(),
                                                nodal_indices[1] as f64 * scale.y() + translate.y(),
                                                nodal_indices[2] as f64 * scale.z() + translate.z(),
                                            ]));
                                            *face_node = node_new;
                                            nodes[nodal_indices[0] as usize]
                                                [nodal_indices[1] as usize]
                                                [nodal_indices[2] as usize] = Some(node_new);
                                            node_new += 1;
                                        }
                                    });
                                cell_faces[*cell].push(cell_face_index);
                                cell_face_index += 1;
                                face_blocks.push(boundary as u8 + 1);
                                face_cells.push(*cell);
                                faces_connectivity.push(face_connectivity)
                            }
                        }
                    })
                })
        });
        let node_face_connectivity =
            invert_connectivity(&faces_connectivity, nodal_coordinates.len());
        let non_manifold_edges = non_manifold(&faces_connectivity, &node_face_connectivity);
        non_manifold_edges.iter().for_each(|non_manifold_edge_a| {
            non_manifold_edges.iter().for_each(|non_manifold_edge_b| {
                if non_manifold_edge_a
                    .iter()
                    .filter(|node_a| non_manifold_edge_b.contains(node_a))
                    .count()
                    == 1
                {
                    panic!("Consecutive non-manifold edges are currently unsupported.")
                }
            })
        });
        non_manifold_edges.iter().for_each(|edge| {
            let non_manifold_faces: Vec<usize> = node_face_connectivity
                [edge[0] - NODE_NUMBERING_OFFSET]
                .iter()
                .filter(|face_a| {
                    node_face_connectivity[edge[1] - NODE_NUMBERING_OFFSET].contains(face_a)
                })
                .copied()
                .collect();
            let mut non_manifold_cells_vec: Vec<usize> = non_manifold_faces
                .iter()
                .map(|&non_manifold_face| face_cells[non_manifold_face])
                .collect();
            non_manifold_cells_vec.sort();
            non_manifold_cells_vec.dedup();
            let non_manifold_cells: [usize; 2] = non_manifold_cells_vec
                .try_into()
                .expect("There should be two non-manifold cells.");
            let non_manifold_cells_non_manifold_faces: [[usize; 2]; 2] = non_manifold_cells
                .iter()
                .map(|&non_manifold_cell| {
                    cell_faces[non_manifold_cell]
                        .iter()
                        .filter(|cell_face| non_manifold_faces.contains(cell_face))
                        .copied()
                        .collect::<Vec<usize>>()
                        .try_into()
                        .expect("There should be two non-manifold faces per non-manifold cell.")
                })
                .collect::<Vec<[usize; 2]>>()
                .try_into()
                .expect("There should be two non-manifold cells.");

            let non_manifold_cells_other_faces: Vec<Vec<usize>> = non_manifold_cells
                .iter()
                .map(|&non_manifold_cell| {
                    cell_faces[non_manifold_cell]
                        .iter()
                        .filter(|cell_face| !non_manifold_faces.contains(cell_face))
                        .copied()
                        .collect()
                })
                .collect();
            let non_manifold_cells_bowtie_faces: Vec<Vec<usize>> =
                non_manifold_cells_non_manifold_faces
                    .iter()
                    .zip(non_manifold_cells_other_faces.iter())
                    .map(|(non_manifold_faces, other_faces)| {
                        other_faces
                            .iter()
                            .filter(|&&other_face| {
                                faces_connectivity[other_face].iter().any(|node| {
                                    non_manifold_faces.iter().all(|&non_manifold_face| {
                                        faces_connectivity[non_manifold_face].contains(node)
                                    })
                                })
                            })
                            .copied()
                            .collect()
                    })
                    .collect();
            let cells_num_bowtie_faces: [usize; 2] = non_manifold_cells_bowtie_faces
                .iter()
                .map(|non_manifold_cell_bowtie_faces| non_manifold_cell_bowtie_faces.len())
                .collect::<Vec<usize>>()
                .try_into()
                .expect("There should be two non-manifold cells.");
            let cell_index = match cells_num_bowtie_faces {
                [0, 0] => unimplemented!("Change below [0] once implemented."),
                [1, 0] | [1, 1] => 0,
                [0, 1] => 1,
                [2, 2] => unimplemented!("Change below [0] once implemented."),
                _ => panic!(),
            };
            let node = edge
                .iter()
                .find(|node| {
                    faces_connectivity[non_manifold_cells_bowtie_faces[cell_index][0]]
                        .contains(node)
                })
                .unwrap();
            let faces: [usize; 3] = node_face_connectivity[node - NODE_NUMBERING_OFFSET]
                .iter()
                .filter(|face| cell_faces[non_manifold_cells[cell_index]].contains(face))
                .copied()
                .collect::<Vec<usize>>()
                .try_into()
                .expect("Should be 3 faces.");
            nodal_coordinates.push(nodal_coordinates[node - NODE_NUMBERING_OFFSET].clone());
            let node_new = nodal_coordinates.len();
            let mut position = 0;
            faces.iter().for_each(|&face| {
                position = faces_connectivity[face]
                    .iter()
                    .position(|face_node| face_node == node)
                    .unwrap();
                faces_connectivity[face][position] = node_new
            });
        });
        let node_face_connectivity =
            invert_connectivity(&faces_connectivity, nodal_coordinates.len());
        let mut faces = [[0; 3]; 2];
        let mut faces_temp;
        for node_index in 0..nodal_coordinates.len() {
            if node_face_connectivity[node_index].len() == 6 {
                faces[0][0] = node_face_connectivity[node_index][0];
                if let Ok(trial_faces) = <[usize; 2]>::try_from(
                    node_face_connectivity[node_index]
                        .iter()
                        .skip(1)
                        .filter(|&&face| {
                            faces_connectivity[faces[0][0]]
                                .iter()
                                .filter(|node| faces_connectivity[face].contains(node))
                                .count()
                                == 2
                        })
                        .copied()
                        .collect::<Vec<usize>>(),
                ) {
                    faces[0][1] = trial_faces[0];
                    faces[0][2] = trial_faces[1];
                    faces_temp = node_face_connectivity[node_index].clone();
                    faces_temp.retain(|face| !faces[0].contains(face));
                    if let Ok(trial_faces_2) = <[usize; 3]>::try_from(faces_temp.clone()) {
                        faces[1] = trial_faces_2;
                        if faces[0].iter().all(|&face_a| {
                            faces[1].iter().all(|&face_b| {
                                faces_connectivity[face_a]
                                    .iter()
                                    .filter(|node_a| faces_connectivity[face_b].contains(node_a))
                                    .count()
                                    == 1
                            })
                        }) {
                            nodal_coordinates.push(nodal_coordinates[node_index].clone());
                            let node = node_index + NODE_NUMBERING_OFFSET;
                            let node_new = nodal_coordinates.len();
                            let mut position = 0;
                            faces[0].iter().for_each(|&face| {
                                position = faces_connectivity[face]
                                    .iter()
                                    .position(|face_node| face_node == &node)
                                    .unwrap();
                                faces_connectivity[face][position] = node_new
                            });
                        }
                    }
                }
            }
        }
        let mut element_blocks = vec![0; 2 * face_blocks.len()];
        let mut element_node_connectivity = vec![[0; 3]; 2 * faces_connectivity.len()];
        let mut face = 0;
        let mut triangle = 0;
        faces_connectivity.iter().for_each(|face_connectivity| {
            element_blocks[triangle] = face_blocks[face];
            element_blocks[triangle + 1] = face_blocks[face];
            element_node_connectivity[triangle] = [
                face_connectivity[0],
                face_connectivity[1],
                face_connectivity[3],
            ];
            element_node_connectivity[triangle + 1] = [
                face_connectivity[1],
                face_connectivity[2],
                face_connectivity[3],
            ];
            face += 1;
            triangle += 2;
        });
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mSurface finite elements\x1b[0m {:?} ",
            time.elapsed()
        );
        Ok(TriangularFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ))
    }
}

fn invert_connectivity(faces_connectivity: &[[usize; 4]], num_nodes: usize) -> Vec<Vec<usize>> {
    let mut node_face_connectivity = vec![vec![]; num_nodes];
    faces_connectivity
        .iter()
        .enumerate()
        .for_each(|(face, connectivity)| {
            connectivity
                .iter()
                .for_each(|node| node_face_connectivity[node - NODE_NUMBERING_OFFSET].push(face))
        });
    node_face_connectivity
}

fn edges(faces_connectivity: &[[usize; 4]]) -> Edges {
    let mut edges: Edges = faces_connectivity
        .iter()
        .flat_map(|connectivity| {
            [
                [connectivity[0], connectivity[1]],
                [connectivity[1], connectivity[2]],
                [connectivity[2], connectivity[3]],
                [connectivity[3], connectivity[0]],
            ]
            .into_iter()
        })
        .collect();
    edges.iter_mut().for_each(|edge| edge.sort());
    edges.sort();
    edges.dedup();
    edges
}

fn non_manifold(faces_connectivity: &[[usize; 4]], node_face_connectivity: &[Vec<usize>]) -> Edges {
    edges(faces_connectivity)
        .iter()
        .flat_map(|&edge| {
            if node_face_connectivity[edge[0] - NODE_NUMBERING_OFFSET]
                .iter()
                .filter(|face_a| {
                    node_face_connectivity[edge[1] - NODE_NUMBERING_OFFSET].contains(face_a)
                })
                .count()
                == 4
            {
                Some(edge)
            } else {
                None
            }
        })
        .collect()
}

impl IntoFiniteElements<HexahedralFiniteElements> for Octree {
    fn into_finite_elements(
        self,
        _nel: Nel,
        _remove: Option<Blocks>,
        scale: Scale,
        translate: Translate,
    ) -> Result<HexahedralFiniteElements, String> {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut element_node_connectivity = vec![];
        let mut nodal_coordinates = Coordinates::zero(0);
        let mut cells_nodes = vec![0; self.len()];
        let mut node_index = 1;
        self.iter().enumerate().for_each(|(cell_index, cell)| {
            if cell.is_leaf() {
                cells_nodes[cell_index] = node_index;
                nodal_coordinates.append(&mut TensorRank1Vec::new(&[[
                    0.5 * (2 * cell.get_min_x() + cell.get_lngth()) as f64 * scale.x()
                        + translate.x(),
                    0.5 * (2 * cell.get_min_y() + cell.get_lngth()) as f64 * scale.y()
                        + translate.y(),
                    0.5 * (2 * cell.get_min_z() + cell.get_lngth()) as f64 * scale.z()
                        + translate.z(),
                ]]));
                node_index += 1;
            }
        });
        let mut connected_faces = [None; NUM_FACES];
        let mut d_01_subcells = None;
        let mut d_04_subcells = None;
        let mut d_14_subcells = None;
        let mut d014_subcells = None;
        let mut fa_0_subcells = [0; NUM_OCTANTS];
        let mut fa_1_subcells = [0; NUM_OCTANTS];
        let mut fa_4_subcells = [0; NUM_OCTANTS];
        let mut face_0_faces = &[None; NUM_FACES];
        self.iter().for_each(|cell| {
            if let Some(cell_subcells) = cell.get_cells() {
                if cell_subcells
                    .iter()
                    .filter(|&&subcell| self[subcell].is_leaf())
                    .count()
                    == NUM_OCTANTS
                {
                    element_node_connectivity.push([
                        cells_nodes[cell_subcells[0]],
                        cells_nodes[cell_subcells[1]],
                        cells_nodes[cell_subcells[3]],
                        cells_nodes[cell_subcells[2]],
                        cells_nodes[cell_subcells[4]],
                        cells_nodes[cell_subcells[5]],
                        cells_nodes[cell_subcells[7]],
                        cells_nodes[cell_subcells[6]],
                    ]);
                    connected_faces = [None; NUM_FACES];
                    d_01_subcells = None;
                    d_04_subcells = None;
                    d_14_subcells = None;
                    d014_subcells = None;
                    cell.get_faces()
                        .iter()
                        .enumerate()
                        .for_each(|(face_index, face_cell)| {
                            if let Some(face_cell_index) = face_cell {
                                if let Some(face_subcells) = self[*face_cell_index].get_cells() {
                                    if face_subcells
                                        .iter()
                                        .filter(|&&subcell| self[subcell].is_leaf())
                                        .count()
                                        == NUM_OCTANTS
                                    {
                                        match face_index {
                                            0 => {
                                                element_node_connectivity.push([
                                                    cells_nodes[face_subcells[2]],
                                                    cells_nodes[face_subcells[3]],
                                                    cells_nodes[cell_subcells[1]],
                                                    cells_nodes[cell_subcells[0]],
                                                    cells_nodes[face_subcells[6]],
                                                    cells_nodes[face_subcells[7]],
                                                    cells_nodes[cell_subcells[5]],
                                                    cells_nodes[cell_subcells[4]],
                                                ]);
                                                connected_faces[0] = Some(face_cell_index)
                                            }
                                            1 => {
                                                element_node_connectivity.push([
                                                    cells_nodes[cell_subcells[1]],
                                                    cells_nodes[face_subcells[0]],
                                                    cells_nodes[face_subcells[2]],
                                                    cells_nodes[cell_subcells[3]],
                                                    cells_nodes[cell_subcells[5]],
                                                    cells_nodes[face_subcells[4]],
                                                    cells_nodes[face_subcells[6]],
                                                    cells_nodes[cell_subcells[7]],
                                                ]);
                                                connected_faces[1] = Some(face_cell_index)
                                            }
                                            4 => {
                                                element_node_connectivity.push([
                                                    cells_nodes[face_subcells[4]],
                                                    cells_nodes[face_subcells[5]],
                                                    cells_nodes[face_subcells[7]],
                                                    cells_nodes[face_subcells[6]],
                                                    cells_nodes[cell_subcells[0]],
                                                    cells_nodes[cell_subcells[1]],
                                                    cells_nodes[cell_subcells[3]],
                                                    cells_nodes[cell_subcells[2]],
                                                ]);
                                                connected_faces[4] = Some(face_cell_index)
                                            }
                                            2 | 3 | 5 => {}
                                            _ => panic!(),
                                        }
                                    }
                                }
                            }
                        });
                    if let Some(face_4) = connected_faces[4] {
                        fa_4_subcells = self[*face_4].get_cells().unwrap();
                    }
                    if let Some(face_1) = connected_faces[1] {
                        fa_1_subcells = self[*face_1].get_cells().unwrap();
                        if connected_faces[4].is_some() {
                            if let Some(diag_subcells) =
                                self[self[*face_1].get_faces()[4].unwrap()].get_cells()
                            {
                                if diag_subcells
                                    .iter()
                                    .filter(|&&subcell| self[subcell].is_leaf())
                                    .count()
                                    == NUM_OCTANTS
                                {
                                    d_14_subcells = Some(diag_subcells);
                                }
                            }
                        }
                    }
                    if let Some(face_0) = connected_faces[0] {
                        fa_0_subcells = self[*face_0].get_cells().unwrap();
                        face_0_faces = self[*face_0].get_faces();
                        if connected_faces[1].is_some() {
                            if let Some(diag_subcells) = self[face_0_faces[1].unwrap()].get_cells()
                            {
                                if diag_subcells
                                    .iter()
                                    .filter(|&&subcell| self[subcell].is_leaf())
                                    .count()
                                    == NUM_OCTANTS
                                {
                                    d_01_subcells = Some(diag_subcells);
                                }
                            }
                        }
                        if connected_faces[4].is_some() {
                            if let Some(diag_subcells) = self[face_0_faces[4].unwrap()].get_cells()
                            {
                                if diag_subcells
                                    .iter()
                                    .filter(|&&subcell| self[subcell].is_leaf())
                                    .count()
                                    == NUM_OCTANTS
                                {
                                    d_04_subcells = Some(diag_subcells);
                                    if d_01_subcells.is_some() && d_01_subcells.is_some() {
                                        if let Some(diag_subcells) = self
                                            [self[face_0_faces[1].unwrap()].get_faces()[4].unwrap()]
                                        .get_cells()
                                        {
                                            if diag_subcells
                                                .iter()
                                                .filter(|&&subcell| self[subcell].is_leaf())
                                                .count()
                                                == NUM_OCTANTS
                                            {
                                                d014_subcells = Some(diag_subcells)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(diag_subcells) = d_01_subcells {
                        element_node_connectivity.push([
                            cells_nodes[fa_0_subcells[3]],
                            cells_nodes[diag_subcells[2]],
                            cells_nodes[fa_1_subcells[0]],
                            cells_nodes[cell_subcells[1]],
                            cells_nodes[fa_0_subcells[7]],
                            cells_nodes[diag_subcells[6]],
                            cells_nodes[fa_1_subcells[4]],
                            cells_nodes[cell_subcells[5]],
                        ]);
                    }
                    if let Some(diag_subcells) = d_04_subcells {
                        element_node_connectivity.push([
                            cells_nodes[diag_subcells[6]],
                            cells_nodes[diag_subcells[7]],
                            cells_nodes[fa_4_subcells[5]],
                            cells_nodes[fa_4_subcells[4]],
                            cells_nodes[fa_0_subcells[2]],
                            cells_nodes[fa_0_subcells[3]],
                            cells_nodes[cell_subcells[1]],
                            cells_nodes[cell_subcells[0]],
                        ]);
                    }
                    if let Some(d_14_subcells) = d_14_subcells {
                        element_node_connectivity.push([
                            cells_nodes[fa_4_subcells[5]],
                            cells_nodes[d_14_subcells[4]],
                            cells_nodes[d_14_subcells[6]],
                            cells_nodes[fa_4_subcells[7]],
                            cells_nodes[cell_subcells[1]],
                            cells_nodes[fa_1_subcells[0]],
                            cells_nodes[fa_1_subcells[2]],
                            cells_nodes[cell_subcells[3]],
                        ]);
                        if let Some(diag_subcells) = d014_subcells {
                            element_node_connectivity.push([
                                cells_nodes[d_04_subcells.unwrap()[7]],
                                cells_nodes[diag_subcells[6]],
                                cells_nodes[d_14_subcells[4]],
                                cells_nodes[fa_4_subcells[5]],
                                cells_nodes[fa_0_subcells[3]],
                                cells_nodes[d_01_subcells.unwrap()[2]],
                                cells_nodes[fa_1_subcells[0]],
                                cells_nodes[cell_subcells[1]],
                            ]);
                        }
                    }
                }
            }
        });
        let fem = Ok(HexahedralFiniteElements::from_data(
            vec![1; element_node_connectivity.len()],
            element_node_connectivity,
            nodal_coordinates,
        ));
        #[cfg(feature = "profile")]
        println!(
            "           \x1b[1;93m  Dualization of primal\x1b[0m {:?} ",
            time.elapsed()
        );
        fem
    }
}
