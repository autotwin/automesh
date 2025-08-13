use super::{
    Vector,
    fem::{
        FiniteElementMethods, HexahedralFiniteElements, NODE_NUMBERING_OFFSET, Smoothing, TRI,
        TetrahedralFiniteElements, TriangularFiniteElements,
    },
};
use conspire::math::{Tensor, TensorArray};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Error};
use stl_io::{IndexedMesh, IndexedTriangle, Normal, Triangle, Vertex, read_stl, write_stl};

/// The tessellation type.
#[derive(Debug, PartialEq)]
pub struct Tessellation {
    data: IndexedMesh,
}

impl From<HexahedralFiniteElements> for Tessellation {
    fn from(_: HexahedralFiniteElements) -> Self {
        unimplemented!()
    }
}

impl From<TetrahedralFiniteElements> for Tessellation {
    fn from(_: TetrahedralFiniteElements) -> Self {
        unimplemented!()
    }
}

impl From<TriangularFiniteElements> for Tessellation {
    fn from(finite_elements: TriangularFiniteElements) -> Self {
        let mut normal = Vector::zero();
        let mut vertices_tri = [0; TRI];
        let nodal_coordinates = finite_elements.get_nodal_coordinates();
        let vertices = nodal_coordinates
            .iter()
            .map(|coordinate| {
                Vertex::new([
                    coordinate[0] as f32,
                    coordinate[1] as f32,
                    coordinate[2] as f32,
                ])
            })
            .collect();
        let faces = finite_elements
            .get_element_node_connectivity()
            .iter()
            .map(|&connectivity| {
                vertices_tri = [
                    connectivity[0] - NODE_NUMBERING_OFFSET,
                    connectivity[1] - NODE_NUMBERING_OFFSET,
                    connectivity[2] - NODE_NUMBERING_OFFSET,
                ];
                normal = (&nodal_coordinates[vertices_tri[1]]
                    - &nodal_coordinates[vertices_tri[0]])
                    .cross(
                        &(&nodal_coordinates[vertices_tri[2]]
                            - &nodal_coordinates[vertices_tri[0]]),
                    )
                    .normalized();
                IndexedTriangle {
                    normal: Normal::new([normal[0] as f32, normal[1] as f32, normal[2] as f32]),
                    vertices: vertices_tri,
                }
            })
            .collect();
        Tessellation::new(IndexedMesh { vertices, faces })
    }
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
    /// Returns a reference to the internal tessellation data.
    pub fn get_data(&self) -> &IndexedMesh {
        &self.data
    }
    /// Isotropic remeshing of the tessellation.
    pub fn remesh(self, iterations: usize, smoothing_method: &Smoothing) -> Self {
        let mut finite_elements = TriangularFiniteElements::from(self);
        finite_elements.remesh(iterations, smoothing_method);
        finite_elements.into()
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

impl Display for Tessellation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Tessellation with {} vertices and {} faces",
            self.data.vertices.len(),
            self.data.faces.len()
        )
    }
}
