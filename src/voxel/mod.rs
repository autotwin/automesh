#[cfg(feature = "python")]
pub mod py;

#[cfg(test)]
pub mod test;

use super::{
    fem::{Blocks, Connectivity, Coordinates, FiniteElements},
    NODE_NUMBERING_OFFSET,
};
use itertools::Itertools;
use ndarray::{Array3, Axis};
use ndarray_npy::{ReadNpyError, ReadNpyExt, WriteNpyError, WriteNpyExt};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Error, Write},
};

type Nel = [usize; 3];
type Scale = [f64; 3];
type VoxelData = Array3<u8>;
type Translate = [f64; 3];
type VoxelDataSized<const N: usize> = Vec<[usize; N]>;

/// The voxels type.
pub struct Voxels {
    data: VoxelData,
}

/// Inherent implementation of the voxels type.
impl Voxels {
    /// Constructs and returns a new voxels type from an NPY file.
    pub fn from_npy(file_path: &str) -> Result<Self, ReadNpyError> {
        Ok(Self {
            data: voxel_data_from_npy(file_path)?,
        })
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
    #[doc = svgbobdoc::transform!(
    /// Converts the voxels type into a finite elements type, consuming the voxels type.
    ///
    /// The voxel data can be scaled and translated (in that order).
    ///
    /// ```math
    /// x \mapsto s_x x + t_x\qquad y \mapsto s_y y + t_y\qquad z \mapsto s_z z + t_z
    /// ```
    ///
    /// ```svgbob
    ///                     8       7
    ///                      *-------*
    ///                   5 /|    6 /|
    ///  z                 *-+-----* |
    ///   ^  y             | |4    | |3
    ///   | ^              | *-----|-*
    ///   |/               |/      |/
    ///   +-----> x        *-------*
    ///                    1       2
    /// ```
    )]
    pub fn into_finite_elements(
        self,
        remove: Option<Vec<u8>>,
        scale: &Scale,
        translate: &Translate,
    ) -> Result<FiniteElements, String> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_npy_data(self.get_data(), remove, scale, translate)?;
        Ok(FiniteElements::from_data(
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

fn element_node_connectivity_node_renumbering(element_node_connectivity: &mut Connectivity) {
    element_node_connectivity
        .clone()
        .into_iter()
        .flatten()
        .unique()
        .sorted()
        .enumerate()
        .filter(|(index, id)| &(index + 1) != id)
        .for_each(|(index, id)| {
            element_node_connectivity
                .iter_mut()
                .flatten()
                .filter(|entry| *entry == &id)
                .for_each(|entry| *entry = index + 1)
        });
}

fn filter_voxel_data(data: &VoxelData, remove: Option<Vec<u8>>) -> (VoxelDataSized<3>, Blocks) {
    let removed_data = remove.unwrap_or(vec![0]);
    let filtered_voxel_data_combo: VoxelDataSized<4> = data
        .axis_iter(Axis(2))
        .enumerate()
        .map(|(k, data_k)| {
            data_k
                .axis_iter(Axis(1))
                .enumerate()
                .map(|(j, data_kj)| {
                    data_kj
                        .iter()
                        .enumerate()
                        .filter(|(_, &data_kji)| !removed_data.contains(&data_kji))
                        .map(|(i, data_kji)| [i, j, k, *data_kji as usize])
                        .collect()
                })
                .collect()
        })
        .collect::<Vec<Vec<Vec<[usize; 4]>>>>()
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    let element_blocks = filtered_voxel_data_combo
        .iter()
        .map(|entry| entry[3])
        .collect();
    let filtered_voxel_data = filtered_voxel_data_combo
        .into_iter()
        .map(|entry| [entry[0], entry[1], entry[2]])
        .collect();
    (filtered_voxel_data, element_blocks)
}

fn finite_element_data_from_npy_data(
    data: &VoxelData,
    remove: Option<Vec<u8>>,
    scale: &Scale,
    translate: &Translate,
) -> Result<(Blocks, Connectivity, Coordinates), String> {
    let shape = data.shape();
    let nelxplus1 = shape[0] + 1;
    let nelyplus1 = shape[1] + 1;
    let xscale = scale[0];
    let yscale = scale[1];
    let zscale = scale[2];
    let xtranslate = translate[0];
    let ytranslate = translate[1];
    let ztranslate = translate[2];
    if xscale <= 0.0 {
        Err("Need to specify xscale > 0.0".to_string())
    } else if yscale <= 0.0 {
        Err("Need to specify yscale > 0.0".to_string())
    } else if zscale <= 0.0 {
        Err("Need to specify zscale > 0.0".to_string())
    } else {
        let (filtered_voxel_data, element_blocks) = filter_voxel_data(data, remove);
        let mut element_node_connectivity: Connectivity = filtered_voxel_data
            .iter()
            .map(|entry| {
                vec![
                    entry[0]
                        + entry[1] * nelxplus1
                        + entry[2] * nelxplus1 * nelyplus1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + entry[1] * nelxplus1
                        + entry[2] * nelxplus1 * nelyplus1
                        + 1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + (entry[1] + 1) * nelxplus1
                        + entry[2] * nelxplus1 * nelyplus1
                        + 1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + (entry[1] + 1) * nelxplus1
                        + entry[2] * nelxplus1 * nelyplus1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + entry[1] * nelxplus1
                        + (entry[2] + 1) * nelxplus1 * nelyplus1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + entry[1] * nelxplus1
                        + (entry[2] + 1) * nelxplus1 * nelyplus1
                        + 1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + (entry[1] + 1) * nelxplus1
                        + (entry[2] + 1) * nelxplus1 * nelyplus1
                        + 1
                        + NODE_NUMBERING_OFFSET,
                    entry[0]
                        + (entry[1] + 1) * nelxplus1
                        + (entry[2] + 1) * nelxplus1 * nelyplus1
                        + NODE_NUMBERING_OFFSET,
                ]
            })
            .collect();
        element_node_connectivity_node_renumbering(&mut element_node_connectivity);
        let number_of_nodes = element_node_connectivity
            .clone()
            .into_iter()
            .flatten()
            .unique()
            .collect::<Vec<usize>>()
            .len();
        let mut nodal_coordinates = vec![vec![0.0; 3]; number_of_nodes];
        filtered_voxel_data
            .iter()
            .zip(element_node_connectivity.iter())
            .for_each(|(entry, connectivity)| {
                nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64) * xscale + xtranslate,
                    (entry[1] as f64) * yscale + ytranslate,
                    (entry[2] as f64) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64 + 1.0) * xscale + xtranslate,
                    (entry[1] as f64) * yscale + ytranslate,
                    (entry[2] as f64) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64 + 1.0) * xscale + xtranslate,
                    (entry[1] as f64 + 1.0) * yscale + ytranslate,
                    (entry[2] as f64) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64) * xscale + xtranslate,
                    (entry[1] as f64 + 1.0) * yscale + ytranslate,
                    (entry[2] as f64) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64) * xscale + xtranslate,
                    (entry[1] as f64) * yscale + ytranslate,
                    (entry[2] as f64 + 1.0) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64 + 1.0) * xscale + xtranslate,
                    (entry[1] as f64) * yscale + ytranslate,
                    (entry[2] as f64 + 1.0) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64 + 1.0) * xscale + xtranslate,
                    (entry[1] as f64 + 1.0) * yscale + ytranslate,
                    (entry[2] as f64 + 1.0) * zscale + ztranslate,
                ];
                nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET] = vec![
                    (entry[0] as f64) * xscale + xtranslate,
                    (entry[1] as f64 + 1.0) * yscale + ytranslate,
                    (entry[2] as f64 + 1.0) * zscale + ztranslate,
                ];
            });
        Ok((element_blocks, element_node_connectivity, nodal_coordinates))
    }
}

struct IntermediateError {
    message: String,
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

impl From<String> for IntermediateError {
    fn from(error: String) -> IntermediateError {
        IntermediateError { message: error }
    }
}

fn voxel_data_from_npy(file_path: &str) -> Result<VoxelData, ReadNpyError> {
    VoxelData::read_npy(File::open(file_path)?)
}

fn voxel_data_from_spn(file_path: &str, nel: Nel) -> Result<VoxelData, IntermediateError> {
    if nel[0] < 1 {
        Err("Need to specify nelx > 0".to_string())?
    } else if nel[1] < 1 {
        Err("Need to specify nely > 0".to_string())?
    } else if nel[2] < 1 {
        Err("Need to specify nelz > 0".to_string())?
    } else {
        let flat = BufReader::new(File::open(file_path)?)
            .lines()
            .map(|line| line.unwrap().parse().unwrap())
            .collect::<Vec<u8>>();
        let mut data = VoxelData::zeros((nel[0], nel[1], nel[2]));
        data.axis_iter_mut(Axis(2))
            .enumerate()
            .for_each(|(k, mut data_k)| {
                data_k
                    .axis_iter_mut(Axis(1))
                    .enumerate()
                    .for_each(|(j, mut data_jk)| {
                        data_jk.iter_mut().enumerate().for_each(|(i, data_ijk)| {
                            *data_ijk = flat[i + nel[0] * j + nel[0] * nel[1] * k]
                        })
                    })
            });
        Ok(data)
    }
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
