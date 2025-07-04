use automesh::{
    FiniteElementMethods, NODE_NUMBERING_OFFSET, TET, TetrahedralFiniteElements, Voxels,
};
use conspire::{
    constitutive::{
        Constitutive,
        solid::{AppliedLoad, elastic::Elastic, hyperelastic::NeoHookean},
    },
    fem::{
        ElementBlock, FiniteElementBlock, FiniteElementBlockMethods,
        HyperelasticFiniteElementBlock, LinearTetrahedron,
    },
    math::{
        Matrix, Tensor, TensorVec, TestError, Vector, assert_eq_within_tols,
        optimize::{EqualityConstraint, NewtonRaphson},
    },
    mechanics::Scalar,
};

const PARAMETERS: &[Scalar; 2] = &[13.0, 3.0];
const STRAIN: Scalar = 1.23;

fn segmentation() -> [[[u8; 6]; 6]; 6] {
    let mut segmentation = [[[1; 6]; 6]; 6];
    segmentation[2][2][2] = 2;
    segmentation[2][2][3] = 2;
    segmentation[2][3][2] = 2;
    segmentation[2][3][3] = 2;
    segmentation[3][2][2] = 3;
    segmentation[3][2][3] = 3;
    segmentation[3][3][2] = 3;
    segmentation[3][3][3] = 3;
    segmentation
}

macro_rules! affine_test {
    ($fem: ident, $corner: expr) => {
        let (_, mut connectivity, coordinates) = $fem.data();
        connectivity.iter_mut().for_each(|entry| {
            entry
                .iter_mut()
                .for_each(|node| *node -= NODE_NUMBERING_OFFSET)
        });
        let block = ElementBlock::<LinearTetrahedron<NeoHookean<_>>, TET>::new(
            PARAMETERS,
            connectivity,
            coordinates.clone().into(),
        );
        let side_length = segmentation().len() as Scalar;
        let length = coordinates
            .iter()
            .filter(|coordinate| coordinate[0] == 0.0 || coordinate[0].abs() == side_length)
            .count()
            + 3;
        let width = coordinates.len() * 3;
        let mut matrix = Matrix::zero(length, width);
        let mut vector = Vector::zero(length);
        let mut index = 0;
        coordinates
            .iter()
            .enumerate()
            .for_each(|(node, coordinate)| {
                if coordinate[0] == 0.0 || coordinate[0] == side_length {
                    matrix[index][3 * node] = 1.0;
                    if coordinate[0] > 0.0 {
                        vector[index] = coordinate[0] + STRAIN
                    } else {
                        vector[index] = coordinate[0]
                    }
                    index += 1;
                }
            });
        matrix[length - 3][0 * 3 + 1] = 1.0;
        matrix[length - 2][0 * 3 + 2] = 1.0;
        matrix[length - 1][$corner * 3 + 2] = 1.0;
        vector[length - 3] = 0.0;
        vector[length - 2] = 0.0;
        vector[length - 1] = 0.0;
        let solution = block.minimize(
            coordinates.into(),
            NewtonRaphson::default(),
            EqualityConstraint::Linear(matrix, vector),
        )?;
        let (deformation_gradient, _) = NeoHookean::new(PARAMETERS)
            .solve(AppliedLoad::UniaxialStress(STRAIN / side_length + 1.0))?;
        block
            .deformation_gradients(&solution)
            .iter()
            .try_for_each(|deformation_gradients_e| {
                deformation_gradients_e
                    .iter()
                    .try_for_each(|deformation_gradient_g| {
                        assert_eq_within_tols(deformation_gradient_g, &deformation_gradient)
                    })
            })?;
    };
}

#[test]
fn voxels_to_direct_tets() -> Result<(), TestError> {
    let voxels = Voxels::from(segmentation());
    let fem = TetrahedralFiniteElements::from(voxels);
    affine_test!(fem, 35);
    Ok(())
}
