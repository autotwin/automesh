#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    fem::{Blocks, NODE_NUMBERING_OFFSET, NUM_NODES_HEX},
    voxel::{Nel, Scale},
    Coordinate, Coordinates, FiniteElements, HexahedralFiniteElements, Vector, VoxelData, Voxels,
    NSD,
};
use conspire::math::{TensorArray, TensorVec};
use ndarray::{s, Axis};
use std::array::from_fn;

const NUM_FACES: usize = 6;
const NUM_OCTANTS: usize = 8;
const NUM_SUBCELLS_FACE: usize = 4;

type SubcellsOnFace = [usize; NUM_SUBCELLS_FACE];
const SUBCELLS_ON_OWN_FACE_0: SubcellsOnFace = [0, 1, 4, 5];
const SUBCELLS_ON_OWN_FACE_1: SubcellsOnFace = [1, 3, 5, 7];
const SUBCELLS_ON_OWN_FACE_2: SubcellsOnFace = [2, 3, 6, 7];
const SUBCELLS_ON_OWN_FACE_3: SubcellsOnFace = [0, 2, 4, 6];
const SUBCELLS_ON_OWN_FACE_4: SubcellsOnFace = [0, 1, 2, 3];
const SUBCELLS_ON_OWN_FACE_5: SubcellsOnFace = [4, 5, 6, 7];

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
type Faces = [Option<usize>; NUM_FACES];
type Indices = [usize; NUM_OCTANTS];

/// The octree type.
pub type Octree = Vec<Cell>;

type Clusters = Vec<Vec<usize>>;
type SubcellToCellMap = Vec<Option<(usize, usize)>>;

/// Methods for trees such as quadtrees or octrees.
pub trait Tree {
    fn balance(&mut self, strong: bool);
    fn boundaries(&mut self);
    fn clusters(&self, remove: Option<Blocks>) -> (Clusters, SubcellToCellMap);
    fn defeature(&mut self, min_num_voxels: usize, remove: Option<Blocks>);
    fn from_voxels(voxels: Voxels) -> (Nel, Self);
    fn into_finite_elements(
        self,
        remove: Option<Blocks>,
        scale: Scale,
        translate: &Vector,
    ) -> Result<HexahedralFiniteElements, String>;
    fn octree_into_finite_elements(
        self,
        remove: Option<Blocks>,
        scale: Scale,
        translate: &Vector,
    ) -> Result<HexahedralFiniteElements, String>;
    fn pair(&mut self);
    fn prune(&mut self);
    fn subdivide(&mut self, index: usize);
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
            panic!()
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
    pub fn homogeneous(&self, data: &VoxelData) -> Option<u8> {
        let x_min = *self.get_min_x() as usize;
        let y_min = *self.get_min_y() as usize;
        let z_min = *self.get_min_z() as usize;
        let x_max = x_min + *self.get_lngth() as usize;
        let y_max = y_min + *self.get_lngth() as usize;
        let z_max = z_min + *self.get_lngth() as usize;
        let contained = data.slice(s![x_min..x_max, y_min..y_max, z_min..z_max]);
        let mut materials: Blocks = contained.iter().cloned().collect();
        materials.dedup();
        if materials.len() == 1 {
            Some(materials[0])
        } else {
            None
        }
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
                if self[index].get_lngth() > &1 && self[index].cells.is_none() {
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
    fn boundaries(&mut self) {
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
                if cell.get_lngth() > &1 && cell.get_cells().is_none() {
                    block = cell.get_block();
                    if cell
                        .get_faces()
                        .iter()
                        .filter_map(|&face| face)
                        .filter(|&face| self[face].get_cells().is_none())
                        .filter(|&face| self[face].get_block() != block)
                        .count()
                        > 0
                        || cell
                            .get_faces()
                            .iter()
                            .enumerate()
                            .any(|(face, &face_cell_maybe)| {
                                if let Some(face_cell) = face_cell_maybe {
                                    if let Some(subcells) = self[face_cell].get_cells() {
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
                "            \x1b[1;93mBoundaries iteration {}\x1b[0m {:?} ",
                iteration,
                time.elapsed()
            );
            if boundaries {
                break;
            }
        }
    }
    fn clusters(&self, remove: Option<Blocks>) -> (Clusters, SubcellToCellMap) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut removed_data = remove.unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        let mut blocks: Blocks = self
            .iter()
            .filter(|cell| {
                cell.get_cells().is_none() && removed_data.binary_search(&cell.get_block()).is_err()
            })
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
                        if cell.get_cells().is_none() && cell.get_block() == block {
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
        let mut cell_from_subcell_map = vec![None; leaves.iter().flatten().max().unwrap() + 1];
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
                        .filter(|(_, &subcell)| {
                            removed_data
                                .binary_search(&self[subcell].get_block())
                                .is_err()
                        })
                        .for_each(|(subcell_index, &subcell)| {
                            cell_from_subcell_map[subcell] = Some((parent_index, subcell_index))
                        })
                }
            });
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mClusters creation instigation\x1b[0m {:?} ",
            time.elapsed()
        );
        #[allow(unused_variables)]
        let mut cluster_index = 1;
        blocks
            .into_iter()
            .enumerate()
            .for_each(|(block_index, block)| {
                cluster_index = 1;
                let block_leaves = &mut leaves[block_index];
                while let Some(starting_leaf) = block_leaves.pop() {
                    let mut cluster = vec![starting_leaf];
                    #[allow(unused_variables)]
                    for iteration in 1.. {
                        #[cfg(feature = "profile")]
                        let time = Instant::now();
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
                                                            complete = false;
                                                            block_leaves.remove(spot);
                                                            cluster.push(subcells[subcell]);
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
                            if let Some((parent, subcell)) = cell_from_subcell_map[leaf] {
                                self[parent].get_faces().iter().enumerate().for_each(
                                    |(face, face_cell)| {
                                        if let Some(cell) = face_cell {
                                            if subcells_on_own_face(face)
                                                .iter()
                                                .any(|&entry| subcell == entry)
                                            {
                                                if let Ok(spot) = block_leaves.binary_search(cell) {
                                                    if self[*cell].get_block() == block {
                                                        complete = false;
                                                        block_leaves.remove(spot);
                                                        cluster.push(*cell);
                                                    }
                                                }
                                            }
                                        }
                                    },
                                );
                            }
                            index += 1;
                        }
                        #[cfg(feature = "profile")]
                        println!(
                            "             \x1b[1;93mBlock {} cluster {} iteration {}\x1b[0m {:?} ",
                            block,
                            cluster_index,
                            iteration,
                            time.elapsed()
                        );
                        if complete {
                            break;
                        }
                    }
                    clusters.push(cluster);
                    cluster_index += 1;
                }
            });
        (clusters, cell_from_subcell_map)
    }
    fn defeature(&mut self, min_num_voxels: usize, remove: Option<Blocks>) {
        //
        // does Sculpt consider voxels sharing an edge or corner part of the same volume?
        // based on the protrusions thing, seems like it does not
        // seems like one face shared is also not enough ("4 or 5 sides")
        // might have to take care of remaining protrusions in another step
        //
        //
        // should you delete cluster?
        //
        // do you need to add the cells from this cluster to another cluster?
        //
        // do you need to update the volumes?
        // meaning like,
        // do you have to start with the smallest cluster,
        // update all the cluster volumes and restart from the smallest cluster again,
        // in case you made a cluster larger/smaller?
        // like if you had small and medium clusters, both being small enough to eliminate,
        // and a small was within a medium shell, them combined might be large enough to keep
        //
        // if you have to iterate, are there clusters you can lock down once you know they will not change?
        // would be beneficial to remove their leaves from searches and so on
        //
        // what does sculpt do?
        // seems like it does some sort of iteration like this
        //
        //
        //
        // updating the octree somehow would be useful,
        // because then you could defeature an octree before meshing it (dualization, tets, etc.)
        // or at least output it for visualization/testing
        //
        let mut block = 0;
        let mut blocks = vec![];
        let (clusters, cell_from_subcell_map) = self.clusters(remove);
        let mut counts = vec![];
        let mut face_block = 0;
        let mut neighbor_block = 0;
        let mut new_block = 0;
        let mut unique_blocks = vec![];
        let volumes: Vec<usize> = clusters
            .iter()
            .map(|cluster| {
                cluster
                    .iter()
                    .map(|&cell| self[cell].get_lngth().pow(NSD as u32) as usize)
                    .sum()
            })
            .collect();
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
                        if let Some((parent, subcell)) = cell_from_subcell_map[cell] {
                            Some(
                                self[parent]
                                    .get_faces()
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(face, face_cell)| {
                                        if let Some(neighbor_cell) = face_cell {
                                            if self[*neighbor_cell].get_cells().is_none()
                                                && subcells_on_own_face(face)
                                                    .iter()
                                                    .any(|&entry| subcell == entry)
                                            {
                                                neighbor_block = self[*neighbor_cell].get_block();
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
                //
                //
                // how could `blocks` be empty?
                // if a cluster is misidentified as a piece within a block of itself?
                // all these small block 0 (and other blocks) clusters seem suspicious
                //
                //
                if blocks.is_empty() {
                    println!(
                        "{:?}\n{:?}\n{:?}\n{:?}",
                        &cluster, &blocks, &unique_blocks, &counts
                    );
                } else {
                    new_block = unique_blocks[counts
                        .iter()
                        .position(|count| count == counts.iter().max().expect("maximum not found"))
                        .expect("position of maximum not found")];
                    cluster
                        .iter()
                        .for_each(|&cell| self[cell].block = Some(new_block));
                }
            });
    }
    fn from_voxels(voxels: Voxels) -> (Nel, Self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let data_voxels = voxels.get_data();
        let mut neli = 0;
        let nel: Nel = data_voxels
            .shape()
            .iter()
            .map(|nel0| {
                neli = *nel0;
                while (neli & (neli - 1)) != 0 {
                    neli += 1
                }
                neli
            })
            .collect();
        let mut data = VoxelData::from(nel);
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
    fn into_finite_elements(
        self,
        _remove: Option<Blocks>,
        scale: Scale,
        translate: &Vector,
    ) -> Result<HexahedralFiniteElements, String> {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let xtranslate = translate[0];
        let ytranslate = translate[1];
        let ztranslate = translate[2];
        let mut element_node_connectivity = vec![];
        let mut nodal_coordinates = Coordinates::zero(0);
        let mut cells_nodes = vec![0; self.len()];
        let mut node_index = 1;
        self.iter().enumerate().for_each(|(cell_index, cell)| {
            if cell.get_cells().is_none() {
                cells_nodes[cell_index] = node_index;
                nodal_coordinates.append(&mut vec![Coordinate::new([
                    0.5 * (2 * cell.get_min_x() + cell.get_lngth()) as f64 * scale.x() + xtranslate,
                    0.5 * (2 * cell.get_min_y() + cell.get_lngth()) as f64 * scale.y() + ytranslate,
                    0.5 * (2 * cell.get_min_z() + cell.get_lngth()) as f64 * scale.z() + ztranslate,
                ])]);
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
                    .filter(|&&subcell| self[subcell].get_cells().is_none())
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
                                        .filter(|&&subcell| self[subcell].get_cells().is_none())
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
                                    .filter(|&&subcell| self[subcell].get_cells().is_none())
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
                                    .filter(|&&subcell| self[subcell].get_cells().is_none())
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
                                    .filter(|&&subcell| self[subcell].get_cells().is_none())
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
                                                .filter(|&&subcell| {
                                                    self[subcell].get_cells().is_none()
                                                })
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
    fn octree_into_finite_elements(
        self,
        remove: Option<Blocks>,
        scale: Scale,
        translate: &Vector,
    ) -> Result<HexahedralFiniteElements, String> {
        let xtranslate = translate[0];
        let ytranslate = translate[1];
        let ztranslate = translate[2];
        let mut x_min = 0.0;
        let mut y_min = 0.0;
        let mut z_min = 0.0;
        let mut x_val = 0.0;
        let mut y_val = 0.0;
        let mut z_val = 0.0;
        let mut removed_data = remove.unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        let num_elements = self
            .iter()
            .filter(|cell| removed_data.binary_search(&cell.get_block()).is_err())
            .count();
        let mut element_blocks = vec![0; num_elements];
        let mut element_node_connectivity = vec![from_fn(|_| 0); num_elements];
        let mut nodal_coordinates: Coordinates = (0..num_elements * NUM_NODES_HEX)
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
                x_min = *cell.get_min_x() as f64 * scale.x() + xtranslate;
                y_min = *cell.get_min_y() as f64 * scale.y() + ytranslate;
                z_min = *cell.get_min_z() as f64 * scale.z() + ztranslate;
                x_val = (cell.get_min_x() + cell.get_lngth()) as f64 * scale.x() + xtranslate;
                y_val = (cell.get_min_y() + cell.get_lngth()) as f64 * scale.y() + ytranslate;
                z_val = (cell.get_min_z() + cell.get_lngth()) as f64 * scale.z() + ztranslate;
                nodal_coordinates[index] = Coordinate::new([x_min, y_min, z_min]);
                nodal_coordinates[index + 1] = Coordinate::new([x_val, y_min, z_min]);
                nodal_coordinates[index + 2] = Coordinate::new([x_val, y_val, z_min]);
                nodal_coordinates[index + 3] = Coordinate::new([x_min, y_val, z_min]);
                nodal_coordinates[index + 4] = Coordinate::new([x_min, y_min, z_val]);
                nodal_coordinates[index + 5] = Coordinate::new([x_val, y_min, z_val]);
                nodal_coordinates[index + 6] = Coordinate::new([x_val, y_val, z_val]);
                nodal_coordinates[index + 7] = Coordinate::new([x_min, y_val, z_val]);
                index += NUM_NODES_HEX;
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
    fn prune(&mut self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        self.retain(|cell| cell.get_cells().is_none());
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mPruning octree\x1b[0m {:?} ",
            time.elapsed()
        );
    }
    fn subdivide(&mut self, index: usize) {
        assert!(self[index].get_cells().is_none());
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
}
