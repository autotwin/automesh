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

    let found_edge_lengths: Vec<f64> = fem.edge_vectors(connectivity)
        .iter()
        .map(|v| v.norm())
        .collect::<Vec<f64>>();

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

#[test]
fn signed_element_volume_positive() {
    // A standard right-handed tetrahedron.  It volume should be positive.
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0], // Node 0
        [1.0, 0.0, 0.0], // Node 1
        [0.0, 1.0, 0.0], // Node 2
        [0.0, 0.0, 1.0], // Node 3
    ]);
    let element_node_connectivity: Connectivity<4> = vec![[0, 1, 2, 3]];
    let element_blocks: Vec<u8> = vec![1];
    let fem = TetrahedralFiniteElements::from((
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    ));

    // Known volume is 1/6 for this tetrahedron
    let known = 1.0 / 6.0;

    let found = fem.signed_element_volume(&fem.get_element_node_connectivity()[0]);

    assert!((known - found).abs() < EPSILON, "Expected positive volume {} but found {}", known, found);
}

#[test]
fn signed_element_volume_negative() {
    // An inverted (left-handed) tetrahedron.
    // By swapping nodes 1 and 2 in the connectivity, we invert the element.
    // Its volume should be negative.
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0], // Node 0
        [1.0, 0.0, 0.0], // Node 1
        [0.0, 1.0, 0.0], // Node 2
        [0.0, 0.0, 1.0], // Node 3
    ]);
    // Swapped connectivity [0, 2, 1, 3] vs standard [0, 1, 2, 3]
    let element_node_connectivity: Connectivity<4> = vec![[0, 2, 1, 3]];
    let element_blocks: Vec<u8> = vec![1];
    let fem = TetrahedralFiniteElements::from((
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    ));

    // Known volume is -1/6 for this inverted tetrahedron
    let known = -1.0 / 6.0;
    let found = fem.signed_element_volume(&fem.get_element_node_connectivity()[0]);

    assert!((known - found).abs() < EPSILON, "Expected negative volume {} but found {}", known, found);
}

#[test]
fn signed_element_volume_zero() {
    // A degenerate tetrahedron where all points are co-planar.
    // Its volume should be zero.
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0], // Node 0
        [1.0, 0.0, 0.0], // Node 1
        [0.0, 1.0, 0.0], // Node 2
        [1.0, 1.0, 0.0], // Node 3 (co-planar with 0, 1, 2)
    ]);
    let element_node_connectivity: Connectivity<4> = vec![[0, 1, 2, 3]];
    let element_blocks: Vec<u8> = vec![1];
    let fem = TetrahedralFiniteElements::from((
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    ));

    // Expected volume should be zero
    let known = 0.0;
    let found = fem.signed_element_volume(&fem.get_element_node_connectivity()[0]);

    assert!((known - found).abs() < EPSILON, "Expected zero volume but found {}", found);
}

#[test]
fn random_tetrahedron() {
    let nodal_coordinates = Coordinates::new(&[
        [0.5, 0.5, 0.5], // Node 0
        [1.8, 0.2, 1.1], // Node 1
        [0.1, 1.5, 0.3], // Node 2
        [1.3, 1.9, 2.0], // Node 3
    ]);
    let element_node_connectivity: Connectivity<4> = vec![[0, 1, 2, 3]];
    let element_blocks: Vec<u8> = vec![1];
    let fem = TetrahedralFiniteElements::from((
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    ));

    // Known volume for this tetrahedron
    let known = 0.22766666666666668;

    let found = fem.signed_element_volume(&fem.get_element_node_connectivity()[0]);

    assert!((known - found).abs() < EPSILON, "Expected positive volume {} but found {}", known, found);
}