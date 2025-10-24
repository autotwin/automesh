use crate::{fem::TriangularFiniteElements, tessellation::Tessellation, tree::Octree};

impl From<Tessellation> for Octree {
    fn from(tessellation: Tessellation) -> Self {
        TriangularFiniteElements::from(tessellation).into()
    }
}

impl From<TriangularFiniteElements> for Octree {
    fn from(_triangular_finite_elements: TriangularFiniteElements) -> Self {
        //
        // Do not forget to balance and pair at the end.
        //
        todo!()
    }
}
