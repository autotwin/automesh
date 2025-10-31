#[cfg(feature = "python")]
pub mod py;

#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    Coordinate, Coordinates, NSD, Octree, Vector,
    fem::{
        Blocks, Connectivity, FiniteElementMethods, HEX, HexahedralFiniteElements,
        TetrahedralFiniteElements, TriangularFiniteElements,
    },
};
use conspire::math::TensorArray;
use ndarray::{Array3, Axis, parallel::prelude::*, s};
use ndarray_npy::{ReadNpyError, ReadNpyExt, WriteNpyError, WriteNpyExt};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Error, Write},
};

type InitialNodalCoordinates = Vec<Option<Coordinate>>;
type VoxelDataFlattened = Blocks;
type Indices = Vec<[usize; NSD]>;

/// The segmentation data corresponding to voxels.
pub type VoxelData = Array3<u8>;

/// The number of voxels in each direction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Nel {
    x: usize,
    y: usize,
    z: usize,
}

impl Nel {
    pub fn from_input<'a>([nelx, nely, nelz]: [Option<usize>; NSD]) -> Result<Self, &'a str> {
        if let Some(x) = nelx {
            if let Some(y) = nely {
                if let Some(z) = nelz {
                    Ok(Self { x, y, z })
                } else {
                    Err("Argument nelz was required but was not provided")
                }
            } else {
                Err("Argument nely was required but was not provided")
            }
        } else {
            Err("Argument nelx was required but was not provided")
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        [self.x(), self.y(), self.z()].into_iter()
    }
    pub fn x(&self) -> &usize {
        &self.x
    }
    pub fn y(&self) -> &usize {
        &self.y
    }
    pub fn z(&self) -> &usize {
        &self.z
    }
}

impl Default for Nel {
    fn default() -> Self {
        Self { x: 1, y: 1, z: 1 }
    }
}

impl From<[usize; NSD]> for Nel {
    fn from([x, y, z]: [usize; NSD]) -> Self {
        if x < 1 || y < 1 || z < 1 {
            panic!("Need to specify nel > 0.")
        } else {
            Self { x, y, z }
        }
    }
}

impl From<&[usize]> for Nel {
    fn from(nel: &[usize]) -> Self {
        if nel.iter().any(|&entry| entry < 1) {
            panic!("Need to specify nel > 0")
        } else {
            Self {
                x: nel[0],
                y: nel[1],
                z: nel[2],
            }
        }
    }
}

impl From<Nel> for (usize, usize, usize) {
    fn from(nel: Nel) -> Self {
        (nel.x, nel.y, nel.z)
    }
}

impl From<Nel> for VoxelData {
    fn from(nel: Nel) -> Self {
        VoxelData::zeros::<(usize, usize, usize)>(nel.into())
    }
}

impl FromIterator<usize> for Nel {
    fn from_iter<Ii: IntoIterator<Item = usize>>(into_iterator: Ii) -> Self {
        let nel: Vec<usize> = into_iterator.into_iter().collect();
        Self::from(&nel[..])
    }
}

/// The multiplying scale in each direction.
#[derive(Clone, Debug, PartialEq)]
pub struct Scale(Vector);

impl Scale {
    pub fn x(&self) -> &f64 {
        &self.0[0]
    }
    pub fn y(&self) -> &f64 {
        &self.0[1]
    }
    pub fn z(&self) -> &f64 {
        &self.0[2]
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self(Vector::new([1.0; NSD]))
    }
}

impl From<[f64; NSD]> for Scale {
    fn from(scale: [f64; NSD]) -> Self {
        if scale.iter().any(|&entry| entry <= 0.0) {
            panic!("Need to specify scale > 0.")
        } else {
            Self(Vector::new(scale))
        }
    }
}

/// The additive translation in each direction.
#[derive(Clone, Debug, PartialEq)]
pub struct Translate(Vector);

impl Translate {
    pub fn x(&self) -> &f64 {
        &self.0[0]
    }
    pub fn y(&self) -> &f64 {
        &self.0[1]
    }
    pub fn z(&self) -> &f64 {
        &self.0[2]
    }
}

impl Default for Translate {
    fn default() -> Self {
        Self(Vector::new([0.0; NSD]))
    }
}

impl From<Translate> for Vector {
    fn from(translate: Translate) -> Self {
        translate.0
    }
}

impl From<Vector> for Translate {
    fn from(translate: Vector) -> Self {
        Self(translate)
    }
}

impl From<[f64; NSD]> for Translate {
    fn from(translate: [f64; NSD]) -> Self {
        Self(Vector::new(translate))
    }
}

/// The voxels IDs to be removed.
#[derive(Debug, Default)]
pub enum Remove {
    #[default]
    None,
    Some(Blocks),
}

impl From<Remove> for Vec<u8> {
    fn from(remove: Remove) -> Self {
        match remove {
            Remove::Some(blocks) => blocks,
            Remove::None => vec![],
        }
    }
}

impl From<&Remove> for Vec<u8> {
    fn from(remove: &Remove) -> Self {
        match remove {
            Remove::Some(blocks) => blocks.clone(),
            Remove::None => vec![],
        }
    }
}

impl From<Vec<u8>> for Remove {
    fn from(mut blocks: Vec<u8>) -> Self {
        if blocks.is_empty() {
            Self::None
        } else {
            blocks.sort();
            blocks.dedup();
            Self::Some(blocks)
        }
    }
}

impl From<Option<Vec<usize>>> for Remove {
    fn from(option: Option<Vec<usize>>) -> Self {
        if let Some(mut blocks) = option {
            blocks.sort();
            blocks.dedup();
            Self::Some(blocks.into_iter().map(|entry| entry as u8).collect())
        } else {
            Self::None
        }
    }
}

impl From<Option<Blocks>> for Remove {
    fn from(option: Option<Blocks>) -> Self {
        if let Some(mut blocks) = option {
            blocks.sort();
            blocks.dedup();
            Self::Some(blocks)
        } else {
            Self::None
        }
    }
}

/// Extraction ranges for a segmentation.
pub struct Extraction {
    x_min: usize,
    x_max: usize,
    y_min: usize,
    y_max: usize,
    z_min: usize,
    z_max: usize,
}

impl Extraction {
    pub fn from_input<'a>(
        [x_min, x_max, y_min, y_max, z_min, z_max]: [usize; 6],
    ) -> Result<Self, &'a str> {
        if x_min >= x_max {
            Err("Need to specify x_min < x_max")
        } else if y_min >= y_max {
            Err("Need to specify y_min < y_max")
        } else if z_min >= z_max {
            Err("Need to specify z_min < z_max")
        } else {
            Ok(Self {
                x_min,
                x_max,
                y_min,
                y_max,
                z_min,
                z_max,
            })
        }
    }
}

impl From<[usize; 6]> for Extraction {
    fn from([x_min, x_max, y_min, y_max, z_min, z_max]: [usize; 6]) -> Self {
        if x_min >= x_max {
            panic!("Need to specify x_min < x_max.")
        } else if y_min >= y_max {
            panic!("Need to specify y_min < y_max.")
        } else if z_min >= z_max {
            panic!("Need to specify z_min < z_max.")
        } else {
            Self {
                x_min,
                x_max,
                y_min,
                y_max,
                z_min,
                z_max,
            }
        }
    }
}

impl From<[usize; NSD]> for Extraction {
    fn from([x_max, y_max, z_max]: [usize; NSD]) -> Self {
        Self {
            x_min: 0,
            x_max,
            y_min: 0,
            y_max,
            z_min: 0,
            z_max,
        }
    }
}

impl From<Nel> for Extraction {
    fn from(Nel { x, y, z }: Nel) -> Self {
        Self {
            x_min: 0,
            x_max: x,
            y_min: 0,
            y_max: y,
            z_min: 0,
            z_max: z,
        }
    }
}

/// The voxels type.
pub struct Voxels {
    data: VoxelData,
    remove: Remove,
    scale: Scale,
    translate: Translate,
}

impl Voxels {
    /// Returns and moves the data associated with the voxels.
    pub fn data(self) -> (VoxelData, Remove, Scale, Translate) {
        (self.data, self.remove, self.scale, self.translate)
    }
    /// Defeatures clusters with less than a minimum number of voxels.
    pub fn defeature(self, min_num_voxels: usize) -> Self {
        defeature_voxels(min_num_voxels, self)
    }
    /// Shows the difference between two segmentations.
    pub fn diff(&self, voxels: &Self) -> Self {
        Self {
            data: diff_voxels(self.get_data(), voxels.get_data()),
            remove: Remove::default(),
            scale: Scale::default(),
            translate: Translate::default(),
        }
    }
    /// Extends the voxel IDs to be removed.
    pub fn extend_removal(&mut self, remove: Remove) {
        match &mut self.remove {
            Remove::None => self.remove = remove,
            Remove::Some(blocks) => {
                <Vec<u8> as Extend<_>>::extend::<Vec<u8>>(blocks, remove.into())
            }
        }
    }
    /// Extract a specified range of voxels from the segmentation.
    pub fn extract(&mut self, extraction: Extraction) {
        extract_voxels(self, extraction)
    }
    /// Constructs and returns a segmentation from a finite element mesh.
    pub fn from_finite_elements<const M: usize, const N: usize, T>(
        finite_elements: T,
        grid: usize,
        size: f64,
    ) -> Self
    where
        T: FiniteElementMethods<M, N>,
    {
        Octree::from_finite_elements(finite_elements, grid, size).into()
    }
    /// Constructs and returns a new voxels type from an NPY file.
    pub fn from_npy(
        file_path: &str,
        remove: Remove,
        scale: Scale,
        translate: Translate,
    ) -> Result<Self, ReadNpyError> {
        Ok(Self {
            data: voxel_data_from_npy(file_path)?,
            remove,
            scale,
            translate,
        })
    }
    /// Constructs and returns a new voxels type from an SPN file.
    pub fn from_spn(
        file_path: &str,
        nel: Nel,
        remove: Remove,
        scale: Scale,
        translate: Translate,
    ) -> Result<Self, String> {
        Ok(Self {
            data: voxel_data_from_spn(file_path, nel)?,
            remove,
            scale,
            translate,
        })
    }
    /// Returns a reference to the internal voxels data.
    pub fn get_data(&self) -> &VoxelData {
        &self.data
    }
    /// Returns a reference to the voxels removal.
    pub fn get_remove(&self) -> &Remove {
        &self.remove
    }
    /// Returns a reference to the voxels scale.
    pub fn get_scale(&self) -> &Scale {
        &self.scale
    }
    /// Returns a reference to the voxels translation.
    pub fn get_translate(&self) -> &Translate {
        &self.translate
    }
    /// Writes the internal voxels data to an NPY file.
    pub fn write_npy(&self, file_path: &str) -> Result<(), WriteNpyError> {
        write_voxels_to_npy(self.get_data(), file_path)
    }
    /// Writes the internal voxels data to an SPN file.
    pub fn write_spn(&self, file_path: &str) -> Result<(), Error> {
        write_voxels_to_spn(self.get_data(), file_path)
    }
}

impl<const I: usize, const J: usize, const K: usize> From<[[[u8; K]; J]; I]> for Voxels {
    fn from(input: [[[u8; K]; J]; I]) -> Self {
        let nel = Nel::from([I, J, K]);
        let mut data = VoxelData::from(nel);
        data.axis_iter_mut(Axis(2))
            .zip(input)
            .for_each(|(mut data_i, input_i)| {
                data_i
                    .axis_iter_mut(Axis(1))
                    .zip(input_i)
                    .for_each(|(mut data_ij, input_ij)| {
                        data_ij
                            .iter_mut()
                            .zip(input_ij)
                            .for_each(|(data_ijk, input_ijk)| *data_ijk = input_ijk)
                    })
            });
        Self {
            data,
            remove: Remove::default(),
            scale: Scale::default(),
            translate: Translate::default(),
        }
    }
}

impl From<Voxels> for HexahedralFiniteElements {
    fn from(voxels: Voxels) -> Self {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_data(
                voxels.data,
                voxels.remove,
                voxels.scale,
                voxels.translate,
            );
        Self::from((element_blocks, element_node_connectivity, nodal_coordinates))
    }
}

impl From<Voxels> for TetrahedralFiniteElements {
    fn from(voxels: Voxels) -> Self {
        HexahedralFiniteElements::from(voxels).into()
    }
}

impl From<Voxels> for TriangularFiniteElements {
    fn from(_voxels: Voxels) -> Self {
        unimplemented!()
    }
}

impl From<Octree> for Voxels {
    fn from(mut tree: Octree) -> Voxels {
        let mut data = VoxelData::from(tree.nel());
        let mut length = 0;
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;
        tree.prune();
        #[cfg(feature = "profile")]
        let time = Instant::now();
        tree.iter().for_each(|cell| {
            x = *cell.get_min_x() as usize;
            y = *cell.get_min_y() as usize;
            z = *cell.get_min_z() as usize;
            length = *cell.get_lngth() as usize;
            (0..length).for_each(|i| {
                (0..length).for_each(|j| {
                    (0..length).for_each(|k| data[[x + i, y + j, z + k]] = cell.get_block())
                })
            })
        });
        let (remove, scale, translate) = tree.parameters();
        let voxels = Self {
            data,
            remove,
            scale,
            translate,
        };
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mOctree to voxels\x1b[0m {:?}",
            time.elapsed()
        );
        voxels
    }
}

fn extract_voxels(
    voxels: &mut Voxels,
    Extraction {
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    }: Extraction,
) {
    voxels.data = voxels
        .data
        .slice(s![x_min..x_max, y_min..y_max, z_min..z_max])
        .to_owned()
}

fn defeature_voxels(min_num_voxels: usize, voxels: Voxels) -> Voxels {
    let nel_0 = Nel::from(voxels.data.shape());
    let mut tree = Octree::from(voxels);
    tree.balance(true);
    tree.defeature(min_num_voxels);
    let mut voxels = Voxels::from(tree);
    extract_voxels(&mut voxels, Extraction::from(nel_0));
    voxels
}

fn diff_voxels(data_1: &VoxelData, data_2: &VoxelData) -> VoxelData {
    let nel = Nel::from(data_1.shape());
    assert_eq!(
        nel,
        Nel::from(data_2.shape()),
        "Segmentations do not have the same dimensions"
    );
    let mut data = VoxelData::from(nel);
    data.axis_iter_mut(Axis(2))
        .zip(data_1.axis_iter(Axis(2)).zip(data_2.axis_iter(Axis(2))))
        .for_each(|(mut data_i, (data_1_i, data_2_i))| {
            data_i
                .axis_iter_mut(Axis(1))
                .zip(data_1_i.axis_iter(Axis(1)).zip(data_2_i.axis_iter(Axis(1))))
                .for_each(|(mut data_ij, (data_1_ij, data_2_ij))| {
                    data_ij
                        .iter_mut()
                        .zip(data_1_ij.iter().zip(data_2_ij.iter()))
                        .for_each(|(data_ijk, (data_1_ijk, data_2_ijk))| {
                            *data_ijk = (data_1_ijk != data_2_ijk) as u8
                        })
                })
        });
    data
}

fn filter_voxel_data(data: VoxelData, remove: Remove) -> (Indices, Blocks) {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut removed_data = match remove {
        Remove::Some(blocks) => blocks,
        Remove::None => Vec::new(),
    };
    removed_data.sort();
    removed_data.dedup();
    let (filtered_voxel_data, element_blocks) = data
        .axis_iter(Axis(2))
        .into_par_iter()
        .enumerate()
        .flat_map(|(k, data_k)| {
            data_k
                .axis_iter(Axis(1))
                .enumerate()
                .flat_map(|(j, data_kj)| {
                    data_kj
                        .iter()
                        .enumerate()
                        .filter(|&(_, &data_kji)| removed_data.binary_search(&data_kji).is_err())
                        .map(|(i, data_kji)| ([i, j, k], *data_kji))
                        .collect::<Vec<([usize; NSD], u8)>>()
                })
                .collect::<Vec<([usize; NSD], u8)>>()
        })
        .unzip();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mRemoved voxels\x1b[0m {:?}",
        time.elapsed()
    );
    (filtered_voxel_data, element_blocks)
}

fn initial_element_node_connectivity(
    filtered_voxel_data: &Indices,
    nelxplus1: &usize,
    nelyplus1: &usize,
) -> Connectivity<HEX> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let nelxplus1_mul_nelyplus1 = nelxplus1 * nelyplus1;
    let element_node_connectivity: Connectivity<HEX> = filtered_voxel_data
        .par_iter()
        .map(|&[i, j, k]| {
            [
                i + j * nelxplus1 + k * nelxplus1_mul_nelyplus1,
                i + j * nelxplus1 + k * nelxplus1_mul_nelyplus1 + 1,
                i + (j + 1) * nelxplus1 + k * nelxplus1_mul_nelyplus1 + 1,
                i + (j + 1) * nelxplus1 + k * nelxplus1_mul_nelyplus1,
                i + j * nelxplus1 + (k + 1) * nelxplus1_mul_nelyplus1,
                i + j * nelxplus1 + (k + 1) * nelxplus1_mul_nelyplus1 + 1,
                i + (j + 1) * nelxplus1 + (k + 1) * nelxplus1_mul_nelyplus1 + 1,
                i + (j + 1) * nelxplus1 + (k + 1) * nelxplus1_mul_nelyplus1,
            ]
        })
        .collect();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mElement-to-node connectivity\x1b[0m {:?}",
        time.elapsed()
    );
    element_node_connectivity
}

fn initial_nodal_coordinates(
    element_node_connectivity: &Connectivity<HEX>,
    filtered_voxel_data: &Indices,
    number_of_nodes_unfiltered: usize,
    scale: Scale,
    translate: Translate,
) -> InitialNodalCoordinates {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut nodal_coordinates: InitialNodalCoordinates = vec![None; number_of_nodes_unfiltered];
    let offsets = [
        [0, 0, 0],
        [1, 0, 0],
        [1, 1, 0],
        [0, 1, 0],
        [0, 0, 1],
        [1, 0, 1],
        [1, 1, 1],
        [0, 1, 1],
    ];
    filtered_voxel_data
        .iter()
        .zip(element_node_connectivity.iter())
        .for_each(|(&[x, y, z], connectivity)| {
            offsets.iter().enumerate().for_each(|(node, [cx, cy, cz])| {
                if nodal_coordinates[connectivity[node]].is_none() {
                    nodal_coordinates[connectivity[node]] = Some(Coordinate::new([
                        (x + cx) as f64 * scale.x() + translate.x(),
                        (y + cy) as f64 * scale.y() + translate.y(),
                        (z + cz) as f64 * scale.z() + translate.z(),
                    ]))
                }
            })
        });
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mNodal coordinates\x1b[0m {:?}",
        time.elapsed()
    );
    nodal_coordinates
}

fn renumber_nodes(
    element_node_connectivity: &mut Connectivity<HEX>,
    mut initial_nodal_coordinates: InitialNodalCoordinates,
    number_of_nodes_unfiltered: usize,
) -> Coordinates {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut mapping = vec![0; number_of_nodes_unfiltered];
    initial_nodal_coordinates
        .iter()
        .enumerate()
        .filter(|&(_, coordinate)| coordinate.is_some())
        .enumerate()
        .for_each(|(node, (index, _))| mapping[index] = node);
    element_node_connectivity
        .par_iter_mut()
        .for_each(|connectivity| {
            connectivity
                .iter_mut()
                .for_each(|node| *node = mapping[*node])
        });
    initial_nodal_coordinates.retain(|coordinate| coordinate.is_some());
    #[allow(clippy::filter_map_identity)]
    let nodal_coordinates = initial_nodal_coordinates
        .into_iter()
        .filter_map(|entry| entry)
        .collect();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mRenumbered nodes\x1b[0m {:?}",
        time.elapsed()
    );
    nodal_coordinates
}

fn finite_element_data_from_data(
    data: VoxelData,
    remove: Remove,
    scale: Scale,
    translate: Translate,
) -> (Blocks, Connectivity<HEX>, Coordinates) {
    let shape = data.shape();
    let nelxplus1 = shape[0] + 1;
    let nelyplus1 = shape[1] + 1;
    let nelzplus1 = shape[2] + 1;
    let number_of_nodes_unfiltered = nelxplus1 * nelyplus1 * nelzplus1;
    let (filtered_voxel_data, element_blocks) = filter_voxel_data(data, remove);
    let mut element_node_connectivity =
        initial_element_node_connectivity(&filtered_voxel_data, &nelxplus1, &nelyplus1);
    let initial_nodal_coordinates = initial_nodal_coordinates(
        &element_node_connectivity,
        &filtered_voxel_data,
        number_of_nodes_unfiltered,
        scale,
        translate,
    );
    let nodal_coordinates = renumber_nodes(
        &mut element_node_connectivity,
        initial_nodal_coordinates,
        number_of_nodes_unfiltered,
    );
    (element_blocks, element_node_connectivity, nodal_coordinates)
}

pub struct IntermediateError {
    pub message: String,
}

impl From<Error> for IntermediateError {
    fn from(error: Error) -> IntermediateError {
        IntermediateError {
            message: error.to_string(),
        }
    }
}

impl From<IntermediateError> for String {
    fn from(err: IntermediateError) -> String {
        err.message
    }
}

fn voxel_data_from_npy(file_path: &str) -> Result<VoxelData, ReadNpyError> {
    VoxelData::read_npy(File::open(file_path)?)
}

fn voxel_data_from_spn(file_path: &str, nel: Nel) -> Result<VoxelData, IntermediateError> {
    let data_flattened = BufReader::new(File::open(file_path)?)
        .lines()
        .map(|line| line.unwrap().parse().unwrap())
        .collect::<VoxelDataFlattened>();
    let mut data = VoxelData::from(nel);
    data.axis_iter_mut(Axis(2))
        .enumerate()
        .for_each(|(k, mut data_k)| {
            data_k
                .axis_iter_mut(Axis(1))
                .enumerate()
                .for_each(|(j, mut data_jk)| {
                    data_jk.iter_mut().enumerate().for_each(|(i, data_ijk)| {
                        *data_ijk = data_flattened[i + nel.x() * (j + nel.y() * k)]
                    })
                })
        });
    Ok(data)
}

fn write_voxels_to_npy(data: &VoxelData, file_path: &str) -> Result<(), WriteNpyError> {
    data.write_npy(BufWriter::new(File::create(file_path)?))
}

fn write_voxels_to_spn(data: &VoxelData, file_path: &str) -> Result<(), Error> {
    let mut file = BufWriter::new(File::create(file_path)?);
    data.axis_iter(Axis(2)).try_for_each(|entry_2d| {
        entry_2d
            .axis_iter(Axis(1))
            .flatten()
            .try_for_each(|entry| writeln!(file, "{entry}"))
    })
}
