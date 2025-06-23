use super::{
    super::{
        Blocks, NSD,
        fem::py::{HexahedralFiniteElements, TetrahedralFiniteElements},
        py::{IntoFoo, PyIntermediateError},
    },
    Extraction, VoxelData, defeature_voxels, diff_voxels, extract_voxels,
    finite_element_data_from_data, voxel_data_from_npy, voxel_data_from_spn, write_voxels_to_npy,
    write_voxels_to_spn,
};
use pyo3::prelude::*;

pub fn register_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    parent_module.add_class::<Voxels>()?;
    Ok(())
}

/// The voxels class.
#[pyclass]
pub struct Voxels {
    data: VoxelData,
    remove: Blocks,
    scale: [f64; NSD],
    translate: [f64; NSD],
}

#[pymethods]
impl Voxels {
    /// Converts the voxels to hexahedral finite elements.
    pub fn as_hexahedra(&self) -> Result<HexahedralFiniteElements, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_data(
                self.data.clone(),
                self.remove.clone().into(),
                self.scale.into(),
                self.translate.into(),
            );
        Ok(HexahedralFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Converts the voxels type into hexahedral finite elements.
    #[pyo3(signature = (remove=[].to_vec(), scale=[1.0, 1.0, 1.0], translate=[0.0, 0.0, 0.0]))]
    pub fn as_finite_elements(
        &self,
        remove: Blocks,
        scale: [f64; NSD],
        translate: [f64; NSD],
    ) -> Result<HexahedralFiniteElements, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_data(
                self.data.clone(),
                remove.into(),
                scale.into(),
                translate.into(),
            );
        Ok(HexahedralFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Converts the voxels type into tetrahedral finite elements.
    #[pyo3(signature = (remove=[].to_vec(), scale=[1.0, 1.0, 1.0], translate=[0.0, 0.0, 0.0]))]
    pub fn as_tetrahedra(
        &self,
        remove: Blocks,
        scale: [f64; NSD],
        translate: [f64; NSD],
    ) -> Result<TetrahedralFiniteElements, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_data(
                self.data.clone(),
                remove.into(),
                scale.into(),
                translate.into(),
            );
        Ok(TetrahedralFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Defeatures clusters with less than a minimum number of voxels.
    pub fn defeature(&mut self, min_num_voxels: usize) {
        self.data = defeature_voxels(
            min_num_voxels,
            super::Voxels {
                data: self.data.clone(),
                remove: self.remove.clone().into(),
                scale: self.scale.into(),
                translate: self.translate.into(),
            },
        )
        .get_data()
        .clone()
    }
    /// Shows the difference between two segmentations.
    pub fn diff(&self, voxels: &Self) -> Self {
        Self {
            data: diff_voxels(&self.data, &voxels.data),
            remove: vec![],
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
    /// Extract a specified range of voxels from the segmentation.
    pub fn extract(&mut self, extraction: [usize; 6]) {
        extract_voxels(
            &mut super::Voxels {
                data: self.data.clone(),
                remove: self.remove.clone().into(),
                scale: self.scale.into(),
                translate: self.translate.into(),
            },
            Extraction::from(extraction),
        )
    }
    /// Constructs and returns a new voxels type from an NPY file.
    #[staticmethod]
    #[pyo3(signature = (file_path, remove=[].to_vec(), scale=[1.0, 1.0, 1.0], translate=[0.0, 0.0, 0.0]))]
    pub fn from_npy(
        file_path: &str,
        remove: Blocks,
        scale: [f64; NSD],
        translate: [f64; NSD],
    ) -> Result<Self, PyIntermediateError> {
        Ok(Self {
            data: voxel_data_from_npy(file_path)?,
            remove,
            scale,
            translate,
        })
    }
    /// Constructs and returns a new voxels type from an SPN file.
    #[staticmethod]
    #[pyo3(signature = (file_path, nel, remove=[].to_vec(), scale=[1.0, 1.0, 1.0], translate=[0.0, 0.0, 0.0]))]
    pub fn from_spn(
        file_path: &str,
        nel: [usize; NSD],
        remove: Blocks,
        scale: [f64; NSD],
        translate: [f64; NSD],
    ) -> Result<Self, PyIntermediateError> {
        Ok(Self {
            data: voxel_data_from_spn(file_path, nel.into())?,
            remove,
            scale,
            translate,
        })
    }
    /// Writes the internal voxels data to an NPY file.
    pub fn write_npy(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_voxels_to_npy(&self.data, file_path)?)
    }
    /// Writes the internal voxels data to an SPN file.
    pub fn write_spn(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_voxels_to_spn(&self.data, file_path)?)
    }
}
