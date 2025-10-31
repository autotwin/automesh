use super::{
    super::py::{IntoFoo, PyCoordinates, PyIntermediateError},
    Blocks, Connectivity, FiniteElementMethods, FiniteElementSpecifics, HEX, Smoothing, TRI,
    finite_element_data_from_exo, finite_element_data_from_inp, write_finite_elements_to_abaqus,
    write_finite_elements_to_exodus, write_finite_elements_to_mesh, write_finite_elements_to_vtk,
};
use pyo3::prelude::*;

pub fn register_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    parent_module.add_class::<HexahedralFiniteElements>()?;
    parent_module.add_class::<TriangularFiniteElements>()?;
    Ok(())
}

/// The hexahedral finite elements class.
#[pyclass]
pub struct HexahedralFiniteElements {
    element_blocks: Blocks,
    element_node_connectivity: Connectivity<HEX>,
    nodal_coordinates: PyCoordinates,
}

/// The tetrahedral finite elements class.
#[pyclass]
pub struct TetrahedralFiniteElements {
    element_blocks: Blocks,
    element_node_connectivity: Connectivity<HEX>,
    nodal_coordinates: PyCoordinates,
}

/// The triangular finite elements class.
#[pyclass]
pub struct TriangularFiniteElements {
    element_blocks: Blocks,
    element_node_connectivity: Connectivity<TRI>,
    nodal_coordinates: PyCoordinates,
}

#[pymethods]
impl HexahedralFiniteElements {
    /// Constructs and returns a new hexahedral finite elements class from data.
    #[new]
    pub fn from_data(
        element_blocks: Blocks,
        element_node_connectivity: Connectivity<HEX>,
        nodal_coordinates: PyCoordinates,
    ) -> Self {
        Self {
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        }
    }
    /// Constructs and returns a new hexahedral finite elements class from an Exodus file.
    #[staticmethod]
    pub fn from_exo(file_path: &str) -> Result<Self, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_exo(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Constructs and returns a new hexahedral finite elements class from an Abaqus file.
    #[staticmethod]
    pub fn from_inp(file_path: &str) -> Result<Self, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_inp(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Smooths the nodal coordinates according to the provided smoothing method.
    #[pyo3(signature = (method="Taubin", hierarchical=false, iterations=10, pass_band=0.1, scale=0.6307))]
    pub fn smooth(
        &mut self,
        method: &str,
        hierarchical: bool,
        iterations: usize,
        pass_band: f64,
        scale: f64,
    ) -> Result<(), PyIntermediateError> {
        let mut finite_elements = super::HexahedralFiniteElements::from((
            self.element_blocks.clone(),
            self.element_node_connectivity.clone(),
            self.nodal_coordinates.as_foo(),
        ));
        finite_elements.node_element_connectivity()?;
        finite_elements.node_node_connectivity()?;
        if hierarchical {
            finite_elements.nodal_hierarchy()?;
        }
        finite_elements.nodal_influencers();
        match method {
            "Gauss" | "gauss" | "Gaussian" | "gaussian" | "Laplacian" | "Laplace" | "laplacian"
            | "laplace" => {
                finite_elements.smooth(&Smoothing::Laplacian(iterations, scale))?;
            }
            "Taubin" | "taubin" => {
                finite_elements.smooth(&Smoothing::Taubin(iterations, pass_band, scale))?;
            }
            _ => return Err(format!("Invalid smoothing method {method} specified."))?,
        }
        self.element_blocks = finite_elements.element_blocks;
        self.element_node_connectivity = finite_elements.element_node_connectivity;
        self.nodal_coordinates = finite_elements.nodal_coordinates.as_foo();
        Ok(())
    }
    /// Writes the finite elements data to a new Exodus file.
    pub fn write_exo(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_exodus(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements data to a new Abaqus file.
    pub fn write_inp(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_abaqus(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements data to a new Mesh file.
    pub fn write_mesh(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_mesh(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements quality metrics to a new file.
    pub fn write_metrics(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(super::HexahedralFiniteElements::from((
            self.element_blocks.clone(),
            self.element_node_connectivity.clone(),
            self.nodal_coordinates.as_foo(),
        ))
        .write_metrics(file_path)?)
    }
    /// Writes the finite elements data to a new VTK file.
    pub fn write_vtk(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_vtk(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
}

#[pymethods]
impl TetrahedralFiniteElements {
    /// Constructs and returns a new tetrahedral finite elements class from data.
    #[new]
    pub fn from_data(
        element_blocks: Blocks,
        element_node_connectivity: Connectivity<HEX>,
        nodal_coordinates: PyCoordinates,
    ) -> Self {
        Self {
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        }
    }
    /// Constructs and returns a new hexahedral finite elements class from an Exodus file.
    #[staticmethod]
    pub fn from_exo(file_path: &str) -> Result<Self, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_exo(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Constructs and returns a new tetrahedral finite elements class from an Abaqus file.
    #[staticmethod]
    pub fn from_inp(file_path: &str) -> Result<Self, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_inp(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Smooths the nodal coordinates according to the provided smoothing method.
    #[pyo3(signature = (method="Taubin", hierarchical=false, iterations=10, pass_band=0.1, scale=0.6307))]
    pub fn smooth(
        &mut self,
        method: &str,
        hierarchical: bool,
        iterations: usize,
        pass_band: f64,
        scale: f64,
    ) -> Result<(), PyIntermediateError> {
        let mut finite_elements = super::HexahedralFiniteElements::from((
            self.element_blocks.clone(),
            self.element_node_connectivity.clone(),
            self.nodal_coordinates.as_foo(),
        ));
        finite_elements.node_element_connectivity()?;
        finite_elements.node_node_connectivity()?;
        if hierarchical {
            finite_elements.nodal_hierarchy()?;
        }
        finite_elements.nodal_influencers();
        match method {
            "Gauss" | "gauss" | "Gaussian" | "gaussian" | "Laplacian" | "Laplace" | "laplacian"
            | "laplace" => {
                finite_elements.smooth(&Smoothing::Laplacian(iterations, scale))?;
            }
            "Taubin" | "taubin" => {
                finite_elements.smooth(&Smoothing::Taubin(iterations, pass_band, scale))?;
            }
            _ => return Err(format!("Invalid smoothing method {method} specified."))?,
        }
        self.element_blocks = finite_elements.element_blocks;
        self.element_node_connectivity = finite_elements.element_node_connectivity;
        self.nodal_coordinates = finite_elements.nodal_coordinates.as_foo();
        Ok(())
    }
    /// Writes the finite elements data to a new Exodus file.
    pub fn write_exo(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_exodus(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements data to a new Abaqus file.
    pub fn write_inp(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_abaqus(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements data to a new Mesh file.
    pub fn write_mesh(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_mesh(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements quality metrics to a new file.
    pub fn write_metrics(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(super::HexahedralFiniteElements::from((
            self.element_blocks.clone(),
            self.element_node_connectivity.clone(),
            self.nodal_coordinates.as_foo(),
        ))
        .write_metrics(file_path)?)
    }
    /// Writes the finite elements data to a new VTK file.
    pub fn write_vtk(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_vtk(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
}

#[pymethods]
impl TriangularFiniteElements {
    /// Constructs and returns a new triangular finite elements class from data.
    #[new]
    pub fn from_data(
        element_blocks: Blocks,
        element_node_connectivity: Connectivity<TRI>,
        nodal_coordinates: PyCoordinates,
    ) -> Self {
        Self {
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        }
    }
    /// Constructs and returns a new hexahedral finite elements class from an Exodus file.
    #[staticmethod]
    pub fn from_exo(file_path: &str) -> Result<Self, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_exo(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Constructs and returns a new hexahedral finite elements class from an Abaqus file.
    #[staticmethod]
    pub fn from_inp(file_path: &str) -> Result<Self, PyIntermediateError> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_inp(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates.as_foo(),
        ))
    }
    /// Smooths the nodal coordinates according to the provided smoothing method.
    #[pyo3(signature = (method="Taubin", hierarchical=false, iterations=10, pass_band=0.1, scale=0.6307))]
    pub fn smooth(
        &mut self,
        method: &str,
        hierarchical: bool,
        iterations: usize,
        pass_band: f64,
        scale: f64,
    ) -> Result<(), PyIntermediateError> {
        let mut finite_elements = super::TriangularFiniteElements::from((
            self.element_blocks.clone(),
            self.element_node_connectivity.clone(),
            self.nodal_coordinates.as_foo(),
        ));
        finite_elements.node_element_connectivity()?;
        finite_elements.node_node_connectivity()?;
        if hierarchical {
            finite_elements.nodal_hierarchy()?;
        }
        finite_elements.nodal_influencers();
        match method {
            "Gauss" | "gauss" | "Gaussian" | "gaussian" | "Laplacian" | "Laplace" | "laplacian"
            | "laplace" => {
                finite_elements.smooth(&Smoothing::Laplacian(iterations, scale))?;
            }
            "Taubin" | "taubin" => {
                finite_elements.smooth(&Smoothing::Taubin(iterations, pass_band, scale))?;
            }
            _ => return Err(format!("Invalid smoothing method {method} specified."))?,
        }
        self.element_blocks = finite_elements.element_blocks;
        self.element_node_connectivity = finite_elements.element_node_connectivity;
        self.nodal_coordinates = finite_elements.nodal_coordinates.as_foo();
        Ok(())
    }
    /// Writes the finite elements data to a new Exodus file.
    pub fn write_exo(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_exodus(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements data to a new Abaqus file.
    pub fn write_inp(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_abaqus(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements data to a new Mesh file.
    pub fn write_mesh(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_mesh(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
    /// Writes the finite elements quality metrics to a new file.
    pub fn write_metrics(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(super::TriangularFiniteElements::from((
            self.element_blocks.clone(),
            self.element_node_connectivity.clone(),
            self.nodal_coordinates.as_foo(),
        ))
        .write_metrics(file_path)?)
    }
    /// Writes the finite elements data to a new VTK file.
    pub fn write_vtk(&self, file_path: &str) -> Result<(), PyIntermediateError> {
        Ok(write_finite_elements_to_vtk(
            file_path,
            &self.element_blocks,
            &self.element_node_connectivity,
            &self.nodal_coordinates.as_foo(),
        )?)
    }
}
