#[cfg(feature = "python")]
pub mod py;

#[cfg(test)]
pub mod test;

use super::fem::{ElementBlocks, ElementConnectivity, FiniteElements, NodalCoordinates};
use itertools::Itertools;
use ndarray::{Array3, Axis};
use ndarray_npy::{ReadNpyExt, WriteNpyExt};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, ErrorKind},
};

const NODE_NUMBERING_OFFSET: usize = 1;

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
    pub fn from_npy(file_path: &str) -> Self {
        let data = voxel_data_from_npy(file_path);
        Self { data }
    }
    /// Constructs and returns a new voxels type from an SPN file.
    pub fn from_spn(file_path: &str, nel: Nel) -> Self {
        let data = voxel_data_from_spn(file_path, nel);
        Self { data }
    }
    /// Returns a reference to the internal voxel data.
    pub fn get_data(&self) -> &VoxelData {
        &self.data
    }
    #[doc = svgbobdoc::transform!(
    /// Converts the voxels type into a finite element type, consuming the voxels type.
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
    pub fn into_finite_elements(self, scale: &Scale, translate: &Translate) -> FiniteElements {
        let (element_blocks, element_connectivity, nodal_coordinates) =
            finite_element_data_from_npy_data(self.get_data(), scale, translate);
        FiniteElements::from_data(element_blocks, element_connectivity, nodal_coordinates)
    }
    /// Writes the internal voxels data to a NPY file.
    pub fn write_npy(&self, file_path: &str) {
        write_voxels_to_npy(self.get_data(), file_path)
    }
}

fn element_connectivity_node_renumbering(element_connectivity: &mut ElementConnectivity) {
    element_connectivity
        .clone()
        .into_iter()
        .flatten()
        .unique()
        .sorted()
        .enumerate()
        .filter(|(index, id)| &(index + 1) != id)
        .for_each(|(index, id)| {
            element_connectivity
                .iter_mut()
                .flatten()
                .filter(|entry| *entry == &id)
                .for_each(|entry| *entry = index + 1)
        });
}

fn filter_voxel_data(data: &VoxelData) -> (VoxelDataSized<3>, ElementBlocks) {
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
                        .filter(|(_, &data_kji)| data_kji > 0)
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
    scale: &Scale,
    translate: &Translate,
) -> (ElementBlocks, ElementConnectivity, NodalCoordinates) {
    let shape = data.shape();
    let nelxplus1 = shape[0] + 1;
    let nelyplus1 = shape[1] + 1;
    let xscale = scale[0];
    let yscale = scale[1];
    let zscale = scale[2];
    let xtranslate = translate[0];
    let ytranslate = translate[1];
    let ztranslate = translate[2];
    let (filtered_voxel_data, element_blocks) = filter_voxel_data(data);
    let mut element_connectivity: ElementConnectivity = filtered_voxel_data
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
    element_connectivity_node_renumbering(&mut element_connectivity);
    let number_of_nodes = element_connectivity
        .clone()
        .into_iter()
        .flatten()
        .unique()
        .collect::<Vec<usize>>()
        .len();
    let mut nodal_coordinates = vec![vec![0.0; 3]; number_of_nodes];
    filtered_voxel_data
        .iter()
        .zip(element_connectivity.iter())
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
    (element_blocks, element_connectivity, nodal_coordinates)
}

fn voxel_data_from_npy(file_path: &str) -> VoxelData {
    if !file_path.ends_with(".npy") {
        panic!("File type must be .npy.")
    }
    let npy_file = match File::open(file_path) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                panic!("Could not find the .npy file.")
            }
            _ => {
                panic!("Could not open the .npy file.");
            }
        },
    };
    VoxelData::read_npy(npy_file).expect("Could not open the .npy file")
}

fn voxel_data_from_spn(file_path: &str, nel: Nel) -> VoxelData {
    let flat = BufReader::new(File::open(file_path).expect("File was not found."))
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
    data
}

fn write_voxels_to_npy(data: &VoxelData, file_path: &str) {
    data.write_npy(BufWriter::new(File::create(file_path).unwrap()))
        .unwrap();
}