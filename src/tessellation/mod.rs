use super::{
    Vector,
    fem::{
        FiniteElementSpecifics, HexahedralFiniteElements, Size, Smoothing,
        TetrahedralFiniteElements, TriangularFiniteElements,
    },
};
use conspire::math::TensorArray;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Error as ErrorIO};
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
        let (_, connectivity, coordinates) = finite_elements.into();
        let mut normal = Vector::zero();
        let faces = connectivity
            .into_iter()
            .map(|connectivity| {
                normal = TriangularFiniteElements::normal(&coordinates, connectivity);
                IndexedTriangle {
                    normal: Normal::new([normal[0] as f32, normal[1] as f32, normal[2] as f32]),
                    vertices: connectivity,
                }
            })
            .collect();
        let vertices = coordinates
            .into_iter()
            .map(|coordinate| {
                Vertex::new([
                    coordinate[0] as f32,
                    coordinate[1] as f32,
                    coordinate[2] as f32,
                ])
            })
            .collect();
        Tessellation::new(IndexedMesh { vertices, faces })
    }
}

impl TryFrom<&str> for Tessellation {
    type Error = ErrorIO;
    fn try_from(file: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            data: read_stl(&mut File::open(file)?)?,
        })
    }
}

impl Tessellation {
    /// Construct a tessellation from an IndexedMesh.
    pub fn new(indexed_mesh: IndexedMesh) -> Self {
        Self { data: indexed_mesh }
    }
    /// Returns and moves the data associated with the tessellation.
    pub fn data(self) -> IndexedMesh {
        self.data
    }
    /// Returns a reference to the internal tessellation data.
    pub fn get_data(&self) -> &IndexedMesh {
        &self.data
    }
    /// Isotropic remeshing of the tessellation.
    pub fn remesh(self, iterations: usize, smoothing_method: &Smoothing, size: Size) -> Self {
        let mut finite_elements = TriangularFiniteElements::from(self);
        finite_elements.remesh(iterations, smoothing_method, size);
        finite_elements.into()
    }
    /// Writes the tessellation data to a new STL file.
    pub fn write_stl(&self, file_path: &str) -> Result<(), ErrorIO> {
        write_tessellation_to_stl(self.get_data(), file_path)
    }
}

fn write_tessellation_to_stl(data: &IndexedMesh, file_path: &str) -> Result<(), ErrorIO> {
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
