use super::{
    fem::{FiniteElementMethods, TriangularFiniteElements, NODE_NUMBERING_OFFSET},
    Coordinate,
};
use conspire::math::TensorArray;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Error};
use stl_io::{read_stl, write_stl, IndexedMesh, Triangle, Vertex};

/// The tessellation type.
#[derive(Debug, PartialEq)]
pub struct Tessellation {
    data: IndexedMesh,
}

impl Tessellation {
    /// Construct a tessellation from an IndexedMesh.
    pub fn new(indexed_mesh: IndexedMesh) -> Self {
        Self { data: indexed_mesh }
    }
    /// Constructs and returns a new tessellation type from an STL file.
    pub fn from_stl(file_path: &str) -> Result<Self, Error> {
        let mut file = File::open(file_path)?;
        let data = read_stl(&mut file)?;
        Ok(Self { data })
    }
    /// Converts the tessellation into finite elements, consuming the tessellation.
    pub fn into_finite_elements(self) -> TriangularFiniteElements {
        let data = self.get_data();
        let element_blocks = vec![1; data.faces.len()];
        let nodal_coordinates = data
            .vertices
            .iter()
            .map(|&vertex| Coordinate::new([vertex[0].into(), vertex[1].into(), vertex[2].into()]))
            .collect();
        let element_node_connectivity = data
            .faces
            .iter()
            .map(|face| {
                [
                    face.vertices[0] + NODE_NUMBERING_OFFSET,
                    face.vertices[1] + NODE_NUMBERING_OFFSET,
                    face.vertices[2] + NODE_NUMBERING_OFFSET,
                ]
            })
            .collect();
        TriangularFiniteElements::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        )
    }
    /// Returns a reference to the internal tessellation data.
    pub fn get_data(&self) -> &IndexedMesh {
        &self.data
    }
    /// Writes the tessellation data to a new STL file.
    pub fn write_stl(&self, file_path: &str) -> Result<(), Error> {
        write_tessellation_to_stl(self.get_data(), file_path)
    }
}

fn write_tessellation_to_stl(data: &IndexedMesh, file_path: &str) -> Result<(), Error> {
    let mut file = BufWriter::new(File::create(file_path)?);
    let mesh_iter = data.faces.iter().map(|face| Triangle {
        normal: face.normal,
        vertices: face
            .vertices
            .iter()
            .map(|&vertex| data.vertices[vertex])
            .collect::<Vec<Vertex>>()
            .try_into()
            .unwrap(),
    });
    write_stl(&mut file, mesh_iter)?;
    Ok(())
}

// Implement Display trait for better debugging output
impl fmt::Display for Tessellation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Tessellation with {} vertices and {} faces",
            self.data.vertices.len(),
            self.data.faces.len()
        )
    }
}
