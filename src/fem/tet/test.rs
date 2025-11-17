use super::{super::{Coordinates, Connectivity}, TetrahedralFiniteElements};
use conspire::math::TensorVec;

const EPSILON: f64 = 1.0e-14;

#[test]
fn tetrahedral_unit_tests() {
    // https://autotwin.github.io/automesh/cli/metrics_tetrahedral.html

    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ]);
    let element_node_connectivity: Connectivity<4> = vec![[0, 1, 2, 3]];
    let element_blocks: Vec<u8> = vec![1];
    let fem = TetrahedralFiniteElements::from((
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    ));
    let volumes = fem.volumes();
    // Known volume V = 1/6 (approx)= 0.1666667
    assert!((volumes[0] - 1.0 / 6.0).abs() < EPSILON);
}
