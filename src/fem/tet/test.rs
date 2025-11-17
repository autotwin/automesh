use crate::FiniteElementMethods;

use super::{super::{Coordinates, Connectivity}, TetrahedralFiniteElements};
use conspire::math::{Tensor, TensorVec};

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

    // Known volume V = 1/6 (approx)= 0.1666667
    let volumes = fem.volumes();
    assert!((volumes[0] - 1.0 / 6.0).abs() < EPSILON);

    // Test edge lengths and maximum edge ratio
    let connectivity = &fem.get_element_node_connectivity()[0]; // Get the connectivity for the first (and only) element
    let (e0, e1, e2, e3, e4, e5) = fem.edge_vectors(connectivity);

    let found_edge_lengths = [
        e0.norm(), // n1 - n0
        e1.norm(), // n2 - n1
        e2.norm(), // n0 - n2
        e3.norm(), // n3 - n0
        e4.norm(), // n3 - n1
        e5.norm(), // n3 - n2
    ];

    // Gold standard known lengths
    let known_edge_lengths = [
        1.0, // n1 - n0
        (1.25_f64).sqrt(), // n2 - n1
        (1.25_f64).sqrt(), // n0 - n2
        (1.50_f64).sqrt(), // n3 - n0
        (1.50_f64).sqrt(), // n3 - n1
        (1.25_f64).sqrt(), // n3 - n2
    ];

    // Iterator-based element-by-element comparison
    found_edge_lengths
        .iter()
        .zip(known_edge_lengths.iter())
        .enumerate()
        .for_each(|(i, (&found, &known))| {
            let diff = (found - known).abs();
            assert!(
                diff < EPSILON,
                "Edge length mismatch at index {}. Known: {}, Found: {}.  Difference: {}",
                i,
                known,
                found,
                diff
            );
        });
}
