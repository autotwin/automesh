use super::fem::{
    HexahedralFiniteElements, TetrahedralFiniteElements,
    TriangularFiniteElements,
};
use conspire::{
    geometry::{
        Coordinate, Coordinates,
        mesh::{Connectivity, Mesh},
    },
    math::Tensor,
};

/// Re-export of conspire's tessellation type.
pub use conspire::geometry::mesh::Tessellation;

const D: usize = 3;

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
        let (_, connectivity, nodal_coordinates) = finite_elements.into();
        let coordinates: Coordinates<D> = nodal_coordinates
            .iter()
            .map(|coordinate| Coordinate::const_from([coordinate[0], coordinate[1], coordinate[2]]))
            .collect();
        let connectivities = vec![Connectivity::Triangular(connectivity.into())];
        Mesh::from((connectivities, coordinates)).into()
    }
}
