#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Coordinate, Coordinates, Element, FiniteElements, Points, Vector, VoxelData, Voxels,
    ELEMENT_NUM_NODES, NODE_NUMBERING_OFFSET,
};
use flavio::math::Tensor;
use ndarray::{s, Axis};
use std::array::from_fn;

const NUM_OCTANTS: usize = 8;

type Cells = [Cell; NUM_OCTANTS];
type Faces = [Option<usize>; 6];
type Indices = [usize; NUM_OCTANTS];
pub type OcTree = Vec<Cell>;

pub trait Tree {
    fn balance(&mut self);
    fn from_points(levels: &usize, points: &Points) -> Self;
    fn from_voxels(voxels: Voxels) -> Self;
    fn into_finite_elements(
        self,
        element: Element,
        remove: Option<Vec<u8>>,
        scale: &Vector,
        translate: &Vector,
    ) -> Result<FiniteElements, String>;
    fn into_hexahedral_finite_elements(
        self,
        remove: Option<Vec<u8>>,
        scale: &Vector,
        translate: &Vector,
    ) -> Result<FiniteElements, String>;
    fn octree_into_finite_elements(
        self,
        remove: Option<Vec<u8>>,
        scale: &Vector,
        translate: &Vector,
    ) -> Result<FiniteElements, String>;
    fn prune(&mut self);
    fn subdivide(&mut self, index: usize);
}

type HexahedralConnectivity<const D: usize> = [[usize; ELEMENT_NUM_NODES]; D];
// type TetrahedralConnectivity<const D: usize> = [[usize; 4]; D];

#[derive(Debug)]
enum Template {
    Hex00000000(HexahedralConnectivity<1>),
    // Tet0000(TetrahedralConnectivity<12>),
}

#[derive(Debug)]
pub struct Cell {
    block: Option<u8>,
    cells: Option<Indices>,
    level: usize,
    faces: Faces,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    min_z: f64,
    max_z: f64,
    template: Option<Template>,
}

impl Cell {
    fn contains(&self, points: &Points) -> bool {
        for point in points.iter() {
            if &point[0] >= self.get_min_x()
                && &point[0] <= self.get_max_x()
                && &point[1] >= self.get_min_y()
                && &point[1] <= self.get_max_y()
                && &point[2] >= self.get_min_z()
                && &point[2] <= self.get_max_z()
            {
                return true;
            }
        }
        false
    }
    fn get_block(&self) -> u8 {
        if let Some(block) = self.block {
            block
        } else {
            panic!()
        }
    }
    fn get_cells(&self) -> &Option<Indices> {
        &self.cells
    }
    fn get_connectivity(&self) -> &[[usize; ELEMENT_NUM_NODES]] {
        if let Some(template) = self.get_template() {
            match template {
                Template::Hex00000000(connectivity) => connectivity,
            }
        } else {
            panic!()
        }
    }
    fn get_faces(&self) -> &Faces {
        &self.faces
    }
    fn get_level(&self) -> &usize {
        &self.level
    }
    fn get_min_x(&self) -> &f64 {
        &self.min_x
    }
    fn get_max_x(&self) -> &f64 {
        &self.max_x
    }
    fn get_min_y(&self) -> &f64 {
        &self.min_y
    }
    fn get_max_y(&self) -> &f64 {
        &self.max_y
    }
    fn get_min_z(&self) -> &f64 {
        &self.min_z
    }
    fn get_max_z(&self) -> &f64 {
        &self.max_z
    }
    fn get_template(&self) -> &Option<Template> {
        &self.template
    }
    fn homogeneous(&self, data: &VoxelData) -> Option<u8> {
        let x_min = self.get_min_x().round() as u8 as usize;
        let x_max = self.get_max_x().round() as u8 as usize;
        let y_min = self.get_min_y().round() as u8 as usize;
        let y_max = self.get_max_y().round() as u8 as usize;
        let z_min = self.get_min_z().round() as u8 as usize;
        let z_max = self.get_max_z().round() as u8 as usize;
        let contained = data.slice(s![x_min..x_max, y_min..y_max, z_min..z_max]);
        let mut materials: Vec<u8> = contained.iter().cloned().collect();
        materials.dedup();
        if materials.len() == 1 {
            Some(materials[0])
        } else {
            None
        }
    }
    fn subdivide(&mut self, indices: Indices) -> Cells {
        self.cells = Some(indices);
        let level = self.get_level() + 1;
        let min_x = self.get_min_x();
        let max_x = self.get_max_x();
        let min_y = self.get_min_y();
        let max_y = self.get_max_y();
        let min_z = self.get_min_z();
        let max_z = self.get_max_z();
        let val_x = 0.5 * (min_x + max_x);
        let val_y = 0.5 * (min_y + max_y);
        let val_z = 0.5 * (min_z + max_z);
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
                level,
                min_x: *min_x,
                max_x: val_x,
                min_y: *min_y,
                max_y: val_y,
                min_z: *min_z,
                max_z: val_z,
                template: None,
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
                level,
                min_x: val_x,
                max_x: *max_x,
                min_y: *min_y,
                max_y: val_y,
                min_z: *min_z,
                max_z: val_z,
                template: None,
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
                level,
                min_x: *min_x,
                max_x: val_x,
                min_y: val_y,
                max_y: *max_y,
                min_z: *min_z,
                max_z: val_z,
                template: None,
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
                level,
                min_x: val_x,
                max_x: *max_x,
                min_y: val_y,
                max_y: *max_y,
                min_z: *min_z,
                max_z: val_z,
                template: None,
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
                level,
                min_x: *min_x,
                max_x: val_x,
                min_y: *min_y,
                max_y: val_y,
                min_z: val_z,
                max_z: *max_z,
                template: None,
            },
            Cell {
                block: None,
                cells: None,
                faces: [
                    None,
                    None,
                    Some(indices[7]),
                    Some(indices[6]),
                    Some(indices[1]),
                    None,
                ],
                level,
                min_x: val_x,
                max_x: *max_x,
                min_y: *min_y,
                max_y: val_y,
                min_z: val_z,
                max_z: *max_z,
                template: None,
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
                level,
                min_x: *min_x,
                max_x: val_x,
                min_y: val_y,
                max_y: *max_y,
                min_z: val_z,
                max_z: *max_z,
                template: None,
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
                level,
                min_x: val_x,
                max_x: *max_x,
                min_y: val_y,
                max_y: *max_y,
                min_z: val_z,
                max_z: *max_z,
                template: None,
            },
        ]
    }
}

impl Tree for OcTree {
    fn balance(&mut self) {
        let mut balanced;
        let mut block;
        let mut index;
        let mut subdivide;
        let levels = *self[self.len() - 1].get_level();
        #[allow(unused_variables)]
        for iteration in 1.. {
            balanced = true;
            index = 0;
            subdivide = false;
            #[cfg(feature = "profile")]
            let time = Instant::now();
            while index < self.len() {
                if self[index].get_level() < &(levels - 1) && self[index].cells.is_none() {
                    'faces: for (face, face_cell) in self[index].get_faces().iter().enumerate() {
                        if let Some(neighbor) = face_cell {
                            if let Some(kids) = self[*neighbor].cells {
                                if match face {
                                    0 => {
                                        self[kids[2]].cells.is_some()
                                            || self[kids[3]].cells.is_some()
                                            || self[kids[6]].cells.is_some()
                                            || self[kids[7]].cells.is_some()
                                    }
                                    1 => {
                                        self[kids[0]].cells.is_some()
                                            || self[kids[2]].cells.is_some()
                                            || self[kids[4]].cells.is_some()
                                            || self[kids[6]].cells.is_some()
                                    }
                                    2 => {
                                        self[kids[0]].cells.is_some()
                                            || self[kids[1]].cells.is_some()
                                            || self[kids[4]].cells.is_some()
                                            || self[kids[5]].cells.is_some()
                                    }
                                    3 => {
                                        self[kids[1]].cells.is_some()
                                            || self[kids[3]].cells.is_some()
                                            || self[kids[5]].cells.is_some()
                                            || self[kids[7]].cells.is_some()
                                    }
                                    4 => {
                                        self[kids[4]].cells.is_some()
                                            || self[kids[5]].cells.is_some()
                                            || self[kids[6]].cells.is_some()
                                            || self[kids[7]].cells.is_some()
                                    }
                                    5 => {
                                        self[kids[0]].cells.is_some()
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
            if iteration == 1 {
                println!(
                    "           \x1b[1;93m⤷ Balancing iteration {}\x1b[0m {:?} ",
                    iteration,
                    time.elapsed()
                );
            } else {
                println!(
                    "             \x1b[1;93mBalancing iteration {}\x1b[0m {:?} ",
                    iteration,
                    time.elapsed()
                );
            }
            if balanced {
                break;
            }
        }
    }
    fn from_points(levels: &usize, points: &Points) -> Self {
        let x_vals: Vec<f64> = points.iter().map(|point| point[0]).collect();
        let y_vals: Vec<f64> = points.iter().map(|point| point[1]).collect();
        let z_vals: Vec<f64> = points.iter().map(|point| point[2]).collect();
        let min_x = x_vals.iter().cloned().reduce(f64::min).unwrap();
        let max_x = x_vals.iter().cloned().fold(f64::NAN, f64::max);
        let min_y = y_vals.iter().cloned().reduce(f64::min).unwrap();
        let max_y = y_vals.iter().cloned().fold(f64::NAN, f64::max);
        let min_z = z_vals.iter().cloned().reduce(f64::min).unwrap();
        let max_z = z_vals.iter().cloned().fold(f64::NAN, f64::max);
        let mut tree = vec![Cell {
            block: None,
            cells: None,
            faces: [None; 6],
            level: 0,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
            template: None,
        }];
        let mut index = 0;
        while index < tree.len() {
            if tree[index].get_level() < levels && tree[index].contains(points) {
                tree.subdivide(index);
            }
            index += 1;
        }
        tree
    }
    fn from_voxels(voxels: Voxels) -> Self {
        let data_voxels = voxels.get_data();
        let mut nels = [0; 3];
        nels.iter_mut()
            .zip(data_voxels.shape().iter())
            .for_each(|(nel, nel_0)| {
                *nel = *nel_0;
                while (*nel & (*nel - 1)) != 0 {
                    *nel += 1
                }
            });
        let mut data = VoxelData::zeros((nels[0], nels[1], nels[2]));
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
        let nel_min = nels.iter().min().unwrap();
        let length = *nel_min as f64;
        let mut tree = vec![];
        (0..(nels[0] / nel_min)).for_each(|i| {
            (0..(nels[1] / nel_min)).for_each(|j| {
                (0..(nels[2] / nel_min)).for_each(|k| {
                    tree.push(Cell {
                        block: None,
                        cells: None,
                        faces: [None; 6],
                        level: 0,
                        min_x: length * i as f64,
                        max_x: length * (i + 1) as f64,
                        min_y: length * j as f64,
                        max_y: length * (j + 1) as f64,
                        min_z: length * k as f64,
                        max_z: length * (k + 1) as f64,
                        template: None,
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
        tree
    }
    fn into_finite_elements(
        self,
        element: Element,
        remove: Option<Vec<u8>>,
        scale: &Vector,
        translate: &Vector,
    ) -> Result<FiniteElements, String> {
        match element {
            Element::Hexahedron => self.into_hexahedral_finite_elements(remove, scale, translate),
            // Element::Tetrahedron => self.into_tetrahedral_finite_elements(remove, scale, translate),
        }
    }
    fn into_hexahedral_finite_elements(
        mut self,
        remove: Option<Vec<u8>>,
        scale: &Vector,
        translate: &Vector,
    ) -> Result<FiniteElements, String> {
        let xscale = scale[0];
        let yscale = scale[1];
        let zscale = scale[2];
        let xtranslate = translate[0];
        let ytranslate = translate[1];
        let ztranslate = translate[2];
        if xscale <= 0.0 {
            return Err("Need to specify xscale > 0.0".to_string());
        } else if yscale <= 0.0 {
            return Err("Need to specify yscale > 0.0".to_string());
        } else if zscale <= 0.0 {
            return Err("Need to specify zscale > 0.0".to_string());
        }
        let mut node = 1;
        let mut connectivity: HexahedralConnectivity<1> = [from_fn(|_| 0)];
        let mut element_blocks = vec![];
        let mut element_node_connectivity = vec![];
        let mut nodal_coordinates = Coordinates::zero_vec(0);
        let levels = *self
            .iter()
            .max_by(|a, b| a.get_level().cmp(b.get_level()))
            .unwrap()
            .get_level();
        let levels_map: Vec<Vec<usize>> = (0..levels)
            .rev()
            .map(|level| {
                self.iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.get_level() == &level)
                    .map(|(index, _)| index)
                    .collect()
            })
            .collect();
        //
        // how are you going to stitch these together for real?
        // is T11111111 an actual template? or should you step around instead
        // i.e. T11111111 does not create new nodes (only stitches them together), and is not independent from T00000000
        //
        // rather than "templates", should you focus on "transitions" from finer to coarser cells?
        // and even transitions across similar cells?
        // are the "atomic schemes" in the paper the meshes, or the part where the hanging nodes are connected?
        //
        // how does Marco do it? by actually making the intermediate geometry?
        //
        let mut removed_data = remove.unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        let mut quarter_cell_length = 0.0;
        levels_map.iter().for_each(|level_map| {
            level_map.iter().for_each(|index| {
                if let Some(subcells) = self[*index].cells {
                    if subcells
                        .into_iter()
                        .filter(|&subcell| self[subcell].get_cells().is_none())
                        .count()
                        == NUM_OCTANTS
                    {
                        if subcells
                            .into_iter()
                            .filter(|&subcell| {
                                removed_data
                                    .binary_search(&self[subcell].get_block())
                                    .is_err()
                            })
                            .count()
                            == NUM_OCTANTS
                        {
                            //
                            // putting all element in same block for now since hard to handle:
                            // - cells with some subcells marked for removal
                            // - cells with inhomogeneous subcells
                            //
                            element_blocks.push(1);
                            connectivity = [from_fn(|n| node + n)];
                            self[*index].template = Some(Template::Hex00000000(connectivity));
                            node += ELEMENT_NUM_NODES * connectivity.len();
                            element_node_connectivity
                                .push(connectivity[0].into_iter().collect::<Vec<usize>>());
                            quarter_cell_length =
                                0.25 * (self[*index].get_max_x() - self[*index].get_min_x());
                            nodal_coordinates.0.append(&mut vec![
                                Vector::new([
                                    (self[*index].get_min_x() + quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_min_y() + quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_min_z() + quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_max_x() - quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_min_y() + quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_min_z() + quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_max_x() - quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_max_y() - quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_min_z() + quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_min_x() + quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_max_y() - quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_min_z() + quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_min_x() + quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_min_y() + quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_max_z() - quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_max_x() - quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_min_y() + quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_max_z() - quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_max_x() - quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_max_y() - quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_max_z() - quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                                Vector::new([
                                    (self[*index].get_min_x() + quarter_cell_length) * xscale
                                        + xtranslate,
                                    (self[*index].get_max_y() - quarter_cell_length) * yscale
                                        + ytranslate,
                                    (self[*index].get_max_z() - quarter_cell_length) * zscale
                                        + ztranslate,
                                ]),
                            ]);
                        }
                    }
                    // else if subcells
                    //     .into_iter()
                    //     .filter(|&subcell| {
                    //         matches!(self[subcell].get_template(), Some(Template::Hex00000000(_)))
                    //     })
                    //     .count()
                    //     == NUM_OCTANTS
                    // {
                    // //
                    // // are you going to assign the connectivity and template to this cell?
                    // //
                    //     element_blocks.append(&mut vec![1; 19]);
                    //     element_node_connectivity.append(&mut vec![
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][1],
                    //             self[subcells[1]].get_connectivity()[0][0],
                    //             self[subcells[1]].get_connectivity()[0][3],
                    //             self[subcells[0]].get_connectivity()[0][2],
                    //             self[subcells[0]].get_connectivity()[0][5],
                    //             self[subcells[1]].get_connectivity()[0][4],
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //         ],
                    //         vec![
                    //             self[subcells[1]].get_connectivity()[0][3],
                    //             self[subcells[1]].get_connectivity()[0][2],
                    //             self[subcells[3]].get_connectivity()[0][1],
                    //             self[subcells[3]].get_connectivity()[0][0],
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[1]].get_connectivity()[0][6],
                    //             self[subcells[3]].get_connectivity()[0][5],
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //         ],
                    //         vec![
                    //             self[subcells[2]].get_connectivity()[0][1],
                    //             self[subcells[3]].get_connectivity()[0][0],
                    //             self[subcells[3]].get_connectivity()[0][3],
                    //             self[subcells[2]].get_connectivity()[0][2],
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //             self[subcells[3]].get_connectivity()[0][7],
                    //             self[subcells[2]].get_connectivity()[0][6],
                    //         ],
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][3],
                    //             self[subcells[0]].get_connectivity()[0][2],
                    //             self[subcells[2]].get_connectivity()[0][1],
                    //             self[subcells[2]].get_connectivity()[0][0],
                    //             self[subcells[0]].get_connectivity()[0][7],
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //             self[subcells[2]].get_connectivity()[0][4],
                    //         ],
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][2],
                    //             self[subcells[1]].get_connectivity()[0][3],
                    //             self[subcells[3]].get_connectivity()[0][0],
                    //             self[subcells[2]].get_connectivity()[0][1],
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //         ],
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][4],
                    //             self[subcells[0]].get_connectivity()[0][5],
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //             self[subcells[0]].get_connectivity()[0][7],
                    //             self[subcells[4]].get_connectivity()[0][0],
                    //             self[subcells[4]].get_connectivity()[0][1],
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //             self[subcells[4]].get_connectivity()[0][3],
                    //         ],
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][5],
                    //             self[subcells[1]].get_connectivity()[0][4],
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //             self[subcells[4]].get_connectivity()[0][1],
                    //             self[subcells[5]].get_connectivity()[0][0],
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //         ],
                    //         vec![
                    //             self[subcells[1]].get_connectivity()[0][4],
                    //             self[subcells[1]].get_connectivity()[0][5],
                    //             self[subcells[1]].get_connectivity()[0][6],
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[5]].get_connectivity()[0][0],
                    //             self[subcells[5]].get_connectivity()[0][1],
                    //             self[subcells[5]].get_connectivity()[0][2],
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //         ],
                    //         vec![
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[1]].get_connectivity()[0][6],
                    //             self[subcells[3]].get_connectivity()[0][5],
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //             self[subcells[5]].get_connectivity()[0][2],
                    //             self[subcells[7]].get_connectivity()[0][1],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //         ],
                    //         vec![
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //             self[subcells[3]].get_connectivity()[0][5],
                    //             self[subcells[3]].get_connectivity()[0][6],
                    //             self[subcells[3]].get_connectivity()[0][7],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //             self[subcells[7]].get_connectivity()[0][1],
                    //             self[subcells[7]].get_connectivity()[0][2],
                    //             self[subcells[7]].get_connectivity()[0][3],
                    //         ],
                    //         vec![
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //             self[subcells[3]].get_connectivity()[0][7],
                    //             self[subcells[2]].get_connectivity()[0][6],
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //             self[subcells[7]].get_connectivity()[0][3],
                    //             self[subcells[6]].get_connectivity()[0][2],
                    //         ],
                    //         vec![
                    //             self[subcells[2]].get_connectivity()[0][4],
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //             self[subcells[2]].get_connectivity()[0][6],
                    //             self[subcells[2]].get_connectivity()[0][7],
                    //             self[subcells[6]].get_connectivity()[0][0],
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //             self[subcells[6]].get_connectivity()[0][2],
                    //             self[subcells[6]].get_connectivity()[0][3],
                    //         ],
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][7],
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //             self[subcells[2]].get_connectivity()[0][4],
                    //             self[subcells[4]].get_connectivity()[0][3],
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //             self[subcells[6]].get_connectivity()[0][0],
                    //         ],
                    //         vec![
                    //             self[subcells[0]].get_connectivity()[0][6],
                    //             self[subcells[1]].get_connectivity()[0][7],
                    //             self[subcells[3]].get_connectivity()[0][4],
                    //             self[subcells[2]].get_connectivity()[0][5],
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //         ],
                    //         vec![
                    //             self[subcells[4]].get_connectivity()[0][1],
                    //             self[subcells[5]].get_connectivity()[0][0],
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //             self[subcells[4]].get_connectivity()[0][5],
                    //             self[subcells[5]].get_connectivity()[0][4],
                    //             self[subcells[5]].get_connectivity()[0][7],
                    //             self[subcells[4]].get_connectivity()[0][6],
                    //         ],
                    //         vec![
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //             self[subcells[5]].get_connectivity()[0][2],
                    //             self[subcells[7]].get_connectivity()[0][1],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //             self[subcells[5]].get_connectivity()[0][7],
                    //             self[subcells[5]].get_connectivity()[0][6],
                    //             self[subcells[7]].get_connectivity()[0][5],
                    //             self[subcells[7]].get_connectivity()[0][4],
                    //         ],
                    //         vec![
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //             self[subcells[7]].get_connectivity()[0][3],
                    //             self[subcells[6]].get_connectivity()[0][2],
                    //             self[subcells[6]].get_connectivity()[0][5],
                    //             self[subcells[7]].get_connectivity()[0][4],
                    //             self[subcells[7]].get_connectivity()[0][7],
                    //             self[subcells[6]].get_connectivity()[0][6],
                    //         ],
                    //         vec![
                    //             self[subcells[4]].get_connectivity()[0][3],
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //             self[subcells[6]].get_connectivity()[0][0],
                    //             self[subcells[4]].get_connectivity()[0][7],
                    //             self[subcells[4]].get_connectivity()[0][6],
                    //             self[subcells[6]].get_connectivity()[0][5],
                    //             self[subcells[6]].get_connectivity()[0][4],
                    //         ],
                    //         vec![
                    //             self[subcells[4]].get_connectivity()[0][2],
                    //             self[subcells[5]].get_connectivity()[0][3],
                    //             self[subcells[7]].get_connectivity()[0][0],
                    //             self[subcells[6]].get_connectivity()[0][1],
                    //             self[subcells[4]].get_connectivity()[0][6],
                    //             self[subcells[5]].get_connectivity()[0][7],
                    //             self[subcells[7]].get_connectivity()[0][4],
                    //             self[subcells[6]].get_connectivity()[0][5],
                    //         ],
                    //     ]);
                    // }
                }
            })
        });
        Ok(FiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ))
    }
    fn octree_into_finite_elements(
        self,
        remove: Option<Vec<u8>>,
        scale: &Vector,
        translate: &Vector,
    ) -> Result<FiniteElements, String> {
        let xscale = scale[0];
        let yscale = scale[1];
        let zscale = scale[2];
        let xtranslate = translate[0];
        let ytranslate = translate[1];
        let ztranslate = translate[2];
        if xscale <= 0.0 {
            return Err("Need to specify xscale > 0.0".to_string());
        } else if yscale <= 0.0 {
            return Err("Need to specify yscale > 0.0".to_string());
        } else if zscale <= 0.0 {
            return Err("Need to specify zscale > 0.0".to_string());
        }
        let mut removed_data = remove.unwrap_or_default();
        removed_data.sort();
        removed_data.dedup();
        let num_elements = self
            .iter()
            .filter(|cell| removed_data.binary_search(&cell.get_block()).is_err())
            .count();
        let mut element_blocks = vec![0; num_elements];
        let mut element_node_connectivity = vec![vec![0; ELEMENT_NUM_NODES]; num_elements];
        let mut nodal_coordinates: Coordinates = (0..num_elements * ELEMENT_NUM_NODES)
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
                *block = cell.get_block() as usize;
                *connectivity = (index + NODE_NUMBERING_OFFSET
                    ..index + ELEMENT_NUM_NODES + NODE_NUMBERING_OFFSET)
                    .collect();
                nodal_coordinates[index] = Coordinate::new([
                    cell.get_min_x().copy() * xscale + xtranslate,
                    cell.get_min_y().copy() * yscale + ytranslate,
                    cell.get_min_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 1] = Coordinate::new([
                    cell.get_max_x().copy() * xscale + xtranslate,
                    cell.get_min_y().copy() * yscale + ytranslate,
                    cell.get_min_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 2] = Coordinate::new([
                    cell.get_max_x().copy() * xscale + xtranslate,
                    cell.get_max_y().copy() * yscale + ytranslate,
                    cell.get_min_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 3] = Coordinate::new([
                    cell.get_min_x().copy() * xscale + xtranslate,
                    cell.get_max_y().copy() * yscale + ytranslate,
                    cell.get_min_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 4] = Coordinate::new([
                    cell.get_min_x().copy() * xscale + xtranslate,
                    cell.get_min_y().copy() * yscale + ytranslate,
                    cell.get_max_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 5] = Coordinate::new([
                    cell.get_max_x().copy() * xscale + xtranslate,
                    cell.get_min_y().copy() * yscale + ytranslate,
                    cell.get_max_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 6] = Coordinate::new([
                    cell.get_max_x().copy() * xscale + xtranslate,
                    cell.get_max_y().copy() * yscale + ytranslate,
                    cell.get_max_z().copy() * zscale + ztranslate,
                ]);
                nodal_coordinates[index + 7] = Coordinate::new([
                    cell.get_min_x().copy() * xscale + xtranslate,
                    cell.get_max_y().copy() * yscale + ytranslate,
                    cell.get_max_z().copy() * zscale + ztranslate,
                ]);
                index += ELEMENT_NUM_NODES;
            });
        Ok(FiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ))
    }
    fn prune(&mut self) {
        self.retain(|cell| cell.get_cells().is_none())
    }
    fn subdivide(&mut self, index: usize) {
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
