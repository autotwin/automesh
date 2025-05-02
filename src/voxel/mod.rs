#[cfg(feature = "python")]
pub mod py;

#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    fem::{
        Blocks, Connectivity, FiniteElementMethods, HexahedralFiniteElements, HEX,
        NODE_NUMBERING_OFFSET,
    },
    Coordinate, Coordinates, Octree, Tree, Vector, NSD,
};
use conspire::math::TensorArray;
use ndarray::{s, Array3, Axis};
use ndarray_npy::{ReadNpyError, ReadNpyExt, WriteNpyError, WriteNpyExt};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Error, Write},
};

const NODE_NUMBERING_OFFSET_PLUS_ONE: usize = NODE_NUMBERING_OFFSET + 1;

type InitialNodalCoordinates = Vec<Option<Coordinate>>;
type VoxelDataFlattened = Blocks;
type VoxelDataSized<const N: usize> = Vec<[usize; N]>;

/// The segmentation data corresponding to voxels.
pub type VoxelData = Array3<u8>;

/// The number of voxels in each direction.
#[derive(Copy, Clone, Debug)]
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
#[derive(PartialEq)]
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
#[derive(PartialEq)]
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

impl From<[f64; NSD]> for Translate {
    fn from(scale: [f64; NSD]) -> Self {
        Self(Vector::new(scale))
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
}

impl Voxels {
    /// Defeatures clusters with less than a minimum number of voxels.
    pub fn defeature(self, min_num_voxels: usize) -> Self {
        defeature_voxels(min_num_voxels, self)
    }
    /// Extract a specified range of voxels from the segmentation.
    pub fn extract(&mut self, extraction: Extraction) {
        extract_voxels(self, extraction)
    }
    /// Constructs and returns a new voxels type from an NPY file.
    pub fn from_npy(file_path: &str) -> Result<Self, ReadNpyError> {
        Ok(Self {
            data: voxel_data_from_npy(file_path)?,
        })
    }
    /// Constructs and returns a new voxels type from an Octree.
    pub fn from_octree(nel: Nel, mut tree: Octree) -> Self {
        let mut data = VoxelData::from(nel);
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
        let voxels = Self { data };
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mOctree to voxels\x1b[0m {:?}",
            time.elapsed()
        );
        voxels
    }
    /// Constructs and returns a new voxels type from an SPN file.
    pub fn from_spn(file_path: &str, nel: Nel) -> Result<Self, String> {
        Ok(Self {
            data: voxel_data_from_spn(file_path, nel)?,
        })
    }
    /// Returns a reference to the internal voxels data.
    pub fn get_data(&self) -> &VoxelData {
        &self.data
    }
    /// Converts the voxels type into a finite elements type, consuming the voxels type.
    pub fn into_finite_elements(
        self,
        remove: Option<Blocks>,
        scale: Scale,
        translate: Translate,
    ) -> Result<HexahedralFiniteElements, String> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_data(self.get_data(), remove, scale, translate)?;
        Ok(HexahedralFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ))
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
    let (nel, mut tree) = Octree::from_voxels(voxels);
    tree.balance(true);
    tree.defeature(min_num_voxels);
    let mut voxels = Voxels::from_octree(nel, tree);
    extract_voxels(&mut voxels, Extraction::from(nel_0));
    voxels
}

fn filter_voxel_data(data: &VoxelData, remove: Option<Blocks>) -> (VoxelDataSized<NSD>, Blocks) {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut removed_data = remove.unwrap_or_default();
    removed_data.sort();
    removed_data.dedup();
    let (filtered_voxel_data, element_blocks) = data
        .axis_iter(Axis(2))
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
        "           \x1b[1;93m⤷ Removed voxels\x1b[0m {:?}",
        time.elapsed()
    );
    (filtered_voxel_data, element_blocks)
}

fn initial_element_node_connectivity(
    filtered_voxel_data: &VoxelDataSized<NSD>,
    nelxplus1: &usize,
    nelyplus1: &usize,
) -> Connectivity<HEX> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let nelxplus1_mul_nelyplus1 = nelxplus1 * nelyplus1;
    let element_node_connectivity: Connectivity<HEX> = filtered_voxel_data
        .iter()
        .map(|&[i, j, k]| {
            [
                i + j * nelxplus1 + k * nelxplus1_mul_nelyplus1 + NODE_NUMBERING_OFFSET,
                i + j * nelxplus1 + k * nelxplus1_mul_nelyplus1 + NODE_NUMBERING_OFFSET_PLUS_ONE,
                i + (j + 1) * nelxplus1
                    + k * nelxplus1_mul_nelyplus1
                    + NODE_NUMBERING_OFFSET_PLUS_ONE,
                i + (j + 1) * nelxplus1 + k * nelxplus1_mul_nelyplus1 + NODE_NUMBERING_OFFSET,
                i + j * nelxplus1 + (k + 1) * nelxplus1_mul_nelyplus1 + NODE_NUMBERING_OFFSET,
                i + j * nelxplus1
                    + (k + 1) * nelxplus1_mul_nelyplus1
                    + NODE_NUMBERING_OFFSET_PLUS_ONE,
                i + (j + 1) * nelxplus1
                    + (k + 1) * nelxplus1_mul_nelyplus1
                    + NODE_NUMBERING_OFFSET_PLUS_ONE,
                i + (j + 1) * nelxplus1 + (k + 1) * nelxplus1_mul_nelyplus1 + NODE_NUMBERING_OFFSET,
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
    filtered_voxel_data: &VoxelDataSized<NSD>,
    number_of_nodes_unfiltered: usize,
    scale: Scale,
    translate: Translate,
) -> Result<InitialNodalCoordinates, String> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut nodal_coordinates: InitialNodalCoordinates =
        (0..number_of_nodes_unfiltered).map(|_| None).collect();
    filtered_voxel_data
        .iter()
        .zip(element_node_connectivity.iter())
        .for_each(|(&[x, y, z], connectivity)| {
            nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                x as f64 * scale.x() + translate.x(),
                y as f64 * scale.y() + translate.y(),
                z as f64 * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                (x as f64 + 1.0) * scale.x() + translate.x(),
                y as f64 * scale.y() + translate.y(),
                z as f64 * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                (x as f64 + 1.0) * scale.x() + translate.x(),
                (y as f64 + 1.0) * scale.y() + translate.y(),
                z as f64 * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                x as f64 * scale.x() + translate.x(),
                (y as f64 + 1.0) * scale.y() + translate.y(),
                z as f64 * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                x as f64 * scale.x() + translate.x(),
                y as f64 * scale.y() + translate.y(),
                (z as f64 + 1.0) * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                (x as f64 + 1.0) * scale.x() + translate.x(),
                y as f64 * scale.y() + translate.y(),
                (z as f64 + 1.0) * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                (x as f64 + 1.0) * scale.x() + translate.x(),
                (y as f64 + 1.0) * scale.y() + translate.y(),
                (z as f64 + 1.0) * scale.z() + translate.z(),
            ]));
            nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET] = Some(Coordinate::new([
                x as f64 * scale.x() + translate.x(),
                (y as f64 + 1.0) * scale.y() + translate.y(),
                (z as f64 + 1.0) * scale.z() + translate.z(),
            ]));
        });
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mNodal coordinates\x1b[0m {:?}",
        time.elapsed()
    );
    Ok(nodal_coordinates)
}

fn renumber_nodes(
    element_node_connectivity: &mut Connectivity<HEX>,
    mut initial_nodal_coordinates: InitialNodalCoordinates,
    number_of_nodes_unfiltered: usize,
) -> Coordinates {
    #[cfg(feature = "profile")]
    let time = std::time::Instant::now();
    let mut mapping = vec![0; number_of_nodes_unfiltered];
    let mut nodes = 1..=number_of_nodes_unfiltered;
    initial_nodal_coordinates
        .iter()
        .enumerate()
        .filter(|&(_, coordinate)| coordinate.is_some())
        .for_each(|(index, _)| {
            if let Some(node) = nodes.next() {
                mapping[index] = node;
            }
        });
    element_node_connectivity
        .iter_mut()
        .for_each(|connectivity| {
            connectivity
                .iter_mut()
                .for_each(|node| *node = mapping[*node - NODE_NUMBERING_OFFSET])
        });
    initial_nodal_coordinates.retain(|coordinate| coordinate.is_some());
    let nodal_coordinates = initial_nodal_coordinates
        .into_iter()
        .map(|entry| entry.unwrap())
        .collect();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mRenumbered nodes\x1b[0m {:?}",
        time.elapsed()
    );
    nodal_coordinates
}

fn finite_element_data_from_data(
    data: &VoxelData,
    remove: Option<Blocks>,
    scale: Scale,
    translate: Translate,
) -> Result<(Blocks, Connectivity<HEX>, Coordinates), String> {
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
    )?;
    let nodal_coordinates = renumber_nodes(
        &mut element_node_connectivity,
        initial_nodal_coordinates,
        number_of_nodes_unfiltered,
    );
    Ok((element_blocks, element_node_connectivity, nodal_coordinates))
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
            .try_for_each(|entry| writeln!(file, "{}", entry))
    })
}
