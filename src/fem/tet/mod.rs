use super::{FiniteElementSpecifics, FiniteElements, Tessellation};

/// The number of nodes in a tetrahedral finite element.
pub const TET: usize = 4;

/// The tetrahedral finite elements type.
pub type TetrahedralFiniteElements = FiniteElements<TET>;

impl FiniteElementSpecifics for TetrahedralFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 2, 3],
            1 => vec![0, 2, 3],
            2 => vec![0, 1, 3],
            3 => vec![0, 1, 2],
            _ => panic!(),
        }
    }
    fn into_tesselation(self) -> Tessellation {
        unimplemented!()
    }
}
