use crate::fem::calculate_element_volumes_hex;

use super::{
    automesh_header, calculate_maximum_edge_ratios, calculate_maximum_skews,
    calculate_minimum_scaled_jacobians, metrics_headers,
    tri::{calculate_element_areas_tri, calculate_minimum_angles_tri},
    Blocks, Connectivity, Coordinates, FiniteElementMethods, HexahedralFiniteElements, Nodes,
    Smoothing, VecConnectivity, HEX, TRI,
};
use conspire::math::{Tensor, TensorVec};

const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const RAD_TO_DEG: f64 = 1.0 / DEG_TO_RAD;

const EPSILON: f64 = 1.0e-14;
const SMOOTHING_SCALE: f64 = 0.3;

const ZERO: f64 = 0.0;
const ONE_FIFTH: f64 = 1.0 / 5.0;
const ONE_FOURTH: f64 = 1.0 / 4.0;
const ONE_THIRD: f64 = 1.0 / 3.0;

trait FooClone {
    fn clone_foo(&self) -> Self;
}

impl FooClone for Coordinates {
    fn clone_foo(&self) -> Self {
        self.iter().cloned().collect()
    }
}

#[allow(clippy::too_many_arguments)]
fn test_finite_elements(
    element_blocks: Blocks,
    element_node_connectivity: Connectivity<HEX>,
    nodal_coordinates: Coordinates,
    node_element_connectivity_gold: VecConnectivity,
    node_node_connectivity_gold: VecConnectivity,
    exterior_nodes_gold: Nodes,
    interface_nodes_gold: Nodes,
    interior_nodes_gold: Nodes,
    laplacian_gold: Option<Coordinates>,
    smoothed_coordinates_gold: Option<Vec<Coordinates>>,
    nodal_influencers_gold: VecConnectivity,
) {
    let mut finite_elements = HexahedralFiniteElements::from_data(
        element_blocks.clone(),
        element_node_connectivity.clone(),
        nodal_coordinates.clone_foo(),
    );
    assert_eq!(
        finite_elements.node_node_connectivity(),
        Err("Need to calculate the node-to-element connectivity first")
    );
    assert_eq!(
        finite_elements.nodal_hierarchy(),
        Err("Need to calculate the node-to-element connectivity first")
    );
    finite_elements.node_element_connectivity().unwrap();
    finite_elements.node_node_connectivity().unwrap();
    finite_elements.nodal_hierarchy().unwrap();
    finite_elements.nodal_influencers();
    assert_eq!(
        finite_elements.get_nodal_influencers(),
        &nodal_influencers_gold
    );
    assert_eq!(
        finite_elements.get_node_element_connectivity(),
        &node_element_connectivity_gold
    );
    assert_eq!(
        finite_elements.get_node_node_connectivity(),
        &node_node_connectivity_gold
    );
    assert_eq!(finite_elements.get_exterior_nodes(), &exterior_nodes_gold);
    assert_eq!(finite_elements.get_interface_nodes(), &interface_nodes_gold);
    assert_eq!(finite_elements.get_interior_nodes(), &interior_nodes_gold);
    if let Some(gold) = laplacian_gold {
        let laplacian = finite_elements.laplacian(finite_elements.get_node_node_connectivity());
        assert!(laplacian.len() == gold.len());
        laplacian
            .iter()
            .zip(gold.iter())
            .for_each(|(coordinates, gold_coordinates)| {
                coordinates.iter().zip(gold_coordinates.iter()).for_each(
                    |(coordinate, gold_coordinate)| {
                        assert!((coordinate - gold_coordinate).abs() < EPSILON)
                    },
                )
            });
    }
    if let Some(gold_set) = smoothed_coordinates_gold {
        gold_set.iter().enumerate().for_each(|(index, gold)| {
            let iterations = index + 1;
            let mut finite_elements = HexahedralFiniteElements::from_data(
                element_blocks.clone(),
                element_node_connectivity.clone(),
                nodal_coordinates.clone_foo(),
            );
            finite_elements.node_element_connectivity().unwrap();
            finite_elements.node_node_connectivity().unwrap();
            finite_elements.nodal_hierarchy().unwrap();
            finite_elements.nodal_influencers();
            finite_elements
                .smooth(Smoothing::Laplacian(iterations, SMOOTHING_SCALE))
                .unwrap();
            let smoothed_nodal_coordinates = finite_elements.get_nodal_coordinates();
            assert!(smoothed_nodal_coordinates.len() == gold.len());
            smoothed_nodal_coordinates.iter().zip(gold.iter()).for_each(
                |(coordinates, gold_coordinates)| {
                    coordinates.iter().zip(gold_coordinates.iter()).for_each(
                        |(coordinate, gold_coordinate)| {
                            assert!((coordinate - gold_coordinate).abs() < EPSILON)
                        },
                    )
                },
            );
        });
        let mut finite_elements = HexahedralFiniteElements::from_data(
            element_blocks.clone(),
            element_node_connectivity.clone(),
            nodal_coordinates.clone_foo(),
        );
        finite_elements.node_element_connectivity().unwrap();
        finite_elements.node_node_connectivity().unwrap();
        finite_elements.nodal_hierarchy().unwrap();
        let prescribed_nodes = finite_elements.get_boundary_nodes().clone();
        finite_elements
            .set_prescribed_nodes(Some(prescribed_nodes), None)
            .unwrap();
        finite_elements
            .smooth(Smoothing::Laplacian(1, SMOOTHING_SCALE))
            .unwrap();
        finite_elements
            .get_nodal_coordinates()
            .iter()
            .zip(nodal_coordinates.iter())
            .for_each(|(a, b)| assert_eq!(a, b));
    }
}

#[test]
fn single() {
    let element_blocks = vec![11];
    let element_node_connectivity = vec![[1, 2, 4, 3, 5, 6, 8, 7]];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![vec![1]; 8];
    let node_node_connectivity_gold = vec![
        vec![2, 3, 5],
        vec![1, 4, 6],
        vec![1, 4, 7],
        vec![2, 3, 8],
        vec![1, 6, 7],
        vec![2, 5, 8],
        vec![3, 5, 8],
        vec![4, 6, 7],
    ];
    let exterior_nodes_gold = (1..=8).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let laplacian_gold = Coordinates::new(&[
        [ONE_THIRD, ONE_THIRD, ONE_THIRD],
        [-ONE_THIRD, ONE_THIRD, ONE_THIRD],
        [ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [-ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [ONE_THIRD, ONE_THIRD, -ONE_THIRD],
        [-ONE_THIRD, ONE_THIRD, -ONE_THIRD],
        [ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
        [-ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
    ]);
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        Some(laplacian_gold),
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn double_x() {
    let element_blocks = vec![11; 2];
    let element_node_connectivity = vec![[1, 2, 5, 4, 7, 8, 11, 10], [2, 3, 6, 5, 8, 9, 12, 11]];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2],
        vec![1],
        vec![1, 2],
        vec![2],
        vec![1],
        vec![1, 2],
        vec![2],
        vec![1],
        vec![1, 2],
        vec![2],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 4, 7],
        vec![1, 3, 5, 8],
        vec![2, 6, 9],
        vec![1, 5, 10],
        vec![2, 4, 6, 11],
        vec![3, 5, 12],
        vec![1, 8, 10],
        vec![2, 7, 9, 11],
        vec![3, 8, 12],
        vec![4, 7, 11],
        vec![5, 8, 10, 12],
        vec![6, 9, 11],
    ];
    let exterior_nodes_gold = (1..=12).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let laplacian_gold = Coordinates::new(&[
        [ONE_THIRD, ONE_THIRD, ONE_THIRD],
        [ZERO, ONE_FOURTH, ONE_FOURTH],
        [-ONE_THIRD, ONE_THIRD, ONE_THIRD],
        [ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [ZERO, -ONE_FOURTH, ONE_FOURTH],
        [-ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [ONE_THIRD, ONE_THIRD, -ONE_THIRD],
        [ZERO, ONE_FOURTH, -ONE_FOURTH],
        [-ONE_THIRD, ONE_THIRD, -ONE_THIRD],
        [ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
        [ZERO, -ONE_FOURTH, -ONE_FOURTH],
        [-ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
    ]);
    let smoothed_coordinates_gold = vec![
        Coordinates::new(&[
            [0.1, 0.100, 0.100],
            [1.0, 0.075, 0.075],
            [1.9, 0.100, 0.100],
            [0.1, 0.900, 0.100],
            [1.0, 0.925, 0.075],
            [1.9, 0.900, 0.100],
            [0.1, 0.100, 0.900],
            [1.0, 0.075, 0.925],
            [1.9, 0.100, 0.900],
            [0.1, 0.900, 0.900],
            [1.0, 0.925, 0.925],
            [1.9, 0.900, 0.900],
        ]),
        Coordinates::new(&[
            [0.19, 0.1775, 0.1775],
            [1.00, 0.1425, 0.1425],
            [1.81, 0.1775, 0.1775],
            [0.19, 0.8225, 0.1775],
            [1.00, 0.8575, 0.1425],
            [1.81, 0.8225, 0.1775],
            [0.19, 0.1775, 0.8225],
            [1.00, 0.1425, 0.8575],
            [1.81, 0.1775, 0.8225],
            [0.19, 0.8225, 0.8225],
            [1.00, 0.8575, 0.8575],
            [1.81, 0.8225, 0.8225],
        ]),
    ];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        Some(laplacian_gold),
        Some(smoothed_coordinates_gold),
        nodal_influencers_gold,
    );
}

#[test]
fn double_y() {
    let element_blocks = vec![11; 2];
    let element_node_connectivity = vec![[1, 2, 4, 3, 7, 8, 10, 9], [3, 4, 6, 5, 9, 10, 12, 11]];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1],
        vec![1, 2],
        vec![1, 2],
        vec![2],
        vec![2],
        vec![1],
        vec![1],
        vec![1, 2],
        vec![1, 2],
        vec![2],
        vec![2],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 3, 7],
        vec![1, 4, 8],
        vec![1, 4, 5, 9],
        vec![2, 3, 6, 10],
        vec![3, 6, 11],
        vec![4, 5, 12],
        vec![1, 8, 9],
        vec![2, 7, 10],
        vec![3, 7, 10, 11],
        vec![4, 8, 9, 12],
        vec![5, 9, 12],
        vec![6, 10, 11],
    ];
    let exterior_nodes_gold = (1..=12).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn triple() {
    let element_blocks = vec![11; 3];
    let element_node_connectivity = vec![
        [1, 2, 6, 5, 9, 10, 14, 13],
        [2, 3, 7, 6, 10, 11, 15, 14],
        [3, 4, 8, 7, 11, 12, 16, 15],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 5, 9],
        vec![1, 3, 6, 10],
        vec![2, 4, 7, 11],
        vec![3, 8, 12],
        vec![1, 6, 13],
        vec![2, 5, 7, 14],
        vec![3, 6, 8, 15],
        vec![4, 7, 16],
        vec![1, 10, 13],
        vec![2, 9, 11, 14],
        vec![3, 10, 12, 15],
        vec![4, 11, 16],
        vec![5, 9, 14],
        vec![6, 10, 13, 15],
        vec![7, 11, 14, 16],
        vec![8, 12, 15],
    ];
    let exterior_nodes_gold = (1..=16).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn quadruple() {
    let element_blocks = vec![11; 4];
    let element_node_connectivity = vec![
        [1, 2, 7, 6, 11, 12, 17, 16],
        [2, 3, 8, 7, 12, 13, 18, 17],
        [3, 4, 9, 8, 13, 14, 19, 18],
        [4, 5, 10, 9, 14, 15, 20, 19],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 6, 11],
        vec![1, 3, 7, 12],
        vec![2, 4, 8, 13],
        vec![3, 5, 9, 14],
        vec![4, 10, 15],
        vec![1, 7, 16],
        vec![2, 6, 8, 17],
        vec![3, 7, 9, 18],
        vec![4, 8, 10, 19],
        vec![5, 9, 20],
        vec![1, 12, 16],
        vec![2, 11, 13, 17],
        vec![3, 12, 14, 18],
        vec![4, 13, 15, 19],
        vec![5, 14, 20],
        vec![6, 11, 17],
        vec![7, 12, 16, 18],
        vec![8, 13, 17, 19],
        vec![9, 14, 18, 20],
        vec![10, 15, 19],
    ];
    let exterior_nodes_gold = (1..=20).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn quadruple_2_voids() {
    let element_blocks = vec![99; 2];
    let element_node_connectivity = vec![[1, 2, 6, 5, 9, 10, 14, 13], [3, 4, 8, 7, 11, 12, 16, 15]];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1],
        vec![2],
        vec![2],
        vec![1],
        vec![1],
        vec![2],
        vec![2],
        vec![1],
        vec![1],
        vec![2],
        vec![2],
        vec![1],
        vec![1],
        vec![2],
        vec![2],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 5, 9],
        vec![1, 6, 10],
        vec![4, 7, 11],
        vec![3, 8, 12],
        vec![1, 6, 13],
        vec![2, 5, 14],
        vec![3, 8, 15],
        vec![4, 7, 16],
        vec![1, 10, 13],
        vec![2, 9, 14],
        vec![3, 12, 15],
        vec![4, 11, 16],
        vec![5, 9, 14],
        vec![6, 10, 13],
        vec![7, 11, 16],
        vec![8, 12, 15],
    ];
    let exterior_nodes_gold = (1..=16).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn quadruple_2_blocks() {
    let element_blocks = vec![11, 21, 21, 11];
    let element_node_connectivity = vec![
        [1, 2, 7, 6, 11, 12, 17, 16],
        [2, 3, 8, 7, 12, 13, 18, 17],
        [3, 4, 9, 8, 13, 14, 19, 18],
        [4, 5, 10, 9, 14, 15, 20, 19],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 6, 11],
        vec![1, 3, 7, 12],
        vec![2, 4, 8, 13],
        vec![3, 5, 9, 14],
        vec![4, 10, 15],
        vec![1, 7, 16],
        vec![2, 6, 8, 17],
        vec![3, 7, 9, 18],
        vec![4, 8, 10, 19],
        vec![5, 9, 20],
        vec![1, 12, 16],
        vec![2, 11, 13, 17],
        vec![3, 12, 14, 18],
        vec![4, 13, 15, 19],
        vec![5, 14, 20],
        vec![6, 11, 17],
        vec![7, 12, 16, 18],
        vec![8, 13, 17, 19],
        vec![9, 14, 18, 20],
        vec![10, 15, 19],
    ];
    let exterior_nodes_gold = (1..=20).collect();
    let interface_nodes_gold = vec![2, 4, 7, 9, 12, 14, 17, 19];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn quadruple_2_blocks_void() {
    let element_blocks = vec![11, 21, 11];
    let element_node_connectivity = vec![
        [1, 2, 7, 6, 11, 12, 17, 16],
        [2, 3, 8, 7, 12, 13, 18, 17],
        [4, 5, 10, 9, 14, 15, 20, 19],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2],
        vec![3],
        vec![3],
        vec![1],
        vec![1, 2],
        vec![2],
        vec![3],
        vec![3],
        vec![1],
        vec![1, 2],
        vec![2],
        vec![3],
        vec![3],
        vec![1],
        vec![1, 2],
        vec![2],
        vec![3],
        vec![3],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 6, 11],
        vec![1, 3, 7, 12],
        vec![2, 8, 13],
        vec![5, 9, 14],
        vec![4, 10, 15],
        vec![1, 7, 16],
        vec![2, 6, 8, 17],
        vec![3, 7, 18],
        vec![4, 10, 19],
        vec![5, 9, 20],
        vec![1, 12, 16],
        vec![2, 11, 13, 17],
        vec![3, 12, 18],
        vec![4, 15, 19],
        vec![5, 14, 20],
        vec![6, 11, 17],
        vec![7, 12, 16, 18],
        vec![8, 13, 17],
        vec![9, 14, 20],
        vec![10, 15, 19],
    ];
    let exterior_nodes_gold = (1..=20).collect();
    let interface_nodes_gold = vec![2, 7, 12, 17];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn cube() {
    let element_blocks = vec![11; 8];
    let element_node_connectivity = vec![
        [1, 2, 5, 4, 10, 11, 14, 13],
        [2, 3, 6, 5, 11, 12, 15, 14],
        [4, 5, 8, 7, 13, 14, 17, 16],
        [5, 6, 9, 8, 14, 15, 18, 17],
        [10, 11, 14, 13, 19, 20, 23, 22],
        [11, 12, 15, 14, 20, 21, 24, 23],
        [13, 14, 17, 16, 22, 23, 26, 25],
        [14, 15, 18, 17, 23, 24, 27, 26],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [0.0, 0.0, 2.0],
        [1.0, 0.0, 2.0],
        [2.0, 0.0, 2.0],
        [0.0, 1.0, 2.0],
        [1.0, 1.0, 2.0],
        [2.0, 1.0, 2.0],
        [0.0, 2.0, 2.0],
        [1.0, 2.0, 2.0],
        [2.0, 2.0, 2.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2],
        vec![1, 3],
        vec![1, 2, 3, 4],
        vec![2, 4],
        vec![3],
        vec![3, 4],
        vec![4],
        vec![1, 5],
        vec![1, 2, 5, 6],
        vec![2, 6],
        vec![1, 3, 5, 7],
        vec![1, 2, 3, 4, 5, 6, 7, 8],
        vec![2, 4, 6, 8],
        vec![3, 7],
        vec![3, 4, 7, 8],
        vec![4, 8],
        vec![5],
        vec![5, 6],
        vec![6],
        vec![5, 7],
        vec![5, 6, 7, 8],
        vec![6, 8],
        vec![7],
        vec![7, 8],
        vec![8],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 4, 10],
        vec![1, 3, 5, 11],
        vec![2, 6, 12],
        vec![1, 5, 7, 13],
        vec![2, 4, 6, 8, 14],
        vec![3, 5, 9, 15],
        vec![4, 8, 16],
        vec![5, 7, 9, 17],
        vec![6, 8, 18],
        vec![1, 11, 13, 19],
        vec![2, 10, 12, 14, 20],
        vec![3, 11, 15, 21],
        vec![4, 10, 14, 16, 22],
        vec![5, 11, 13, 15, 17, 23],
        vec![6, 12, 14, 18, 24],
        vec![7, 13, 17, 25],
        vec![8, 14, 16, 18, 26],
        vec![9, 15, 17, 27],
        vec![10, 20, 22],
        vec![11, 19, 21, 23],
        vec![12, 20, 24],
        vec![13, 19, 23, 25],
        vec![14, 20, 22, 24, 26],
        vec![15, 21, 23, 27],
        vec![16, 22, 26],
        vec![17, 23, 25, 27],
        vec![18, 24, 26],
    ];
    let mut exterior_nodes_gold = (1..=27).collect::<Nodes>();
    exterior_nodes_gold.remove(13);
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![14];
    let nodal_influencers_gold = vec![
        vec![2, 4, 10],
        vec![1, 3, 5, 11],
        vec![2, 6, 12],
        vec![1, 5, 7, 13],
        vec![2, 4, 6, 8],
        vec![3, 5, 9, 15],
        vec![4, 8, 16],
        vec![5, 7, 9, 17],
        vec![6, 8, 18],
        vec![1, 11, 13, 19],
        vec![2, 10, 12, 20],
        vec![3, 11, 15, 21],
        vec![4, 10, 16, 22],
        vec![5, 11, 13, 15, 17, 23],
        vec![6, 12, 18, 24],
        vec![7, 13, 17, 25],
        vec![8, 16, 18, 26],
        vec![9, 15, 17, 27],
        vec![10, 20, 22],
        vec![11, 19, 21, 23],
        vec![12, 20, 24],
        vec![13, 19, 23, 25],
        vec![20, 22, 24, 26],
        vec![15, 21, 23, 27],
        vec![16, 22, 26],
        vec![17, 23, 25, 27],
        vec![18, 24, 26],
    ];
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn cube_multi() {
    let element_blocks = vec![82, 2, 2, 2, 31, 44];
    let element_node_connectivity = vec![
        [1, 2, 5, 4, 10, 11, 14, 13],
        [2, 3, 6, 5, 11, 12, 15, 14],
        [4, 5, 8, 7, 13, 14, 17, 16],
        [5, 6, 9, 8, 14, 15, 18, 17],
        [11, 12, 15, 14, 19, 20, 22, 21],
        [14, 15, 18, 17, 21, 22, 24, 23],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [1.0, 0.0, 2.0],
        [2.0, 0.0, 2.0],
        [1.0, 1.0, 2.0],
        [2.0, 1.0, 2.0],
        [1.0, 2.0, 2.0],
        [2.0, 2.0, 2.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2],
        vec![1, 3],
        vec![1, 2, 3, 4],
        vec![2, 4],
        vec![3],
        vec![3, 4],
        vec![4],
        vec![1],
        vec![1, 2, 5],
        vec![2, 5],
        vec![1, 3],
        vec![1, 2, 3, 4, 5, 6],
        vec![2, 4, 5, 6],
        vec![3],
        vec![3, 4, 6],
        vec![4, 6],
        vec![5],
        vec![5],
        vec![5, 6],
        vec![5, 6],
        vec![6],
        vec![6],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 4, 10],
        vec![1, 3, 5, 11],
        vec![2, 6, 12],
        vec![1, 5, 7, 13],
        vec![2, 4, 6, 8, 14],
        vec![3, 5, 9, 15],
        vec![4, 8, 16],
        vec![5, 7, 9, 17],
        vec![6, 8, 18],
        vec![1, 11, 13],
        vec![2, 10, 12, 14, 19],
        vec![3, 11, 15, 20],
        vec![4, 10, 14, 16],
        vec![5, 11, 13, 15, 17, 21],
        vec![6, 12, 14, 18, 22],
        vec![7, 13, 17],
        vec![8, 14, 16, 18, 23],
        vec![9, 15, 17, 24],
        vec![11, 20, 21],
        vec![12, 19, 22],
        vec![14, 19, 22, 23],
        vec![15, 20, 21, 24],
        vec![17, 21, 24],
        vec![18, 22, 23],
    ];
    let exterior_nodes_gold = (1..=24).collect();
    let interface_nodes_gold = vec![2, 4, 5, 11, 12, 13, 14, 15, 17, 18, 21, 22];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn cube_with_inclusion() {
    let mut element_blocks = vec![11; 27];
    element_blocks[13] = 88;
    let element_node_connectivity = vec![
        [1, 2, 6, 5, 17, 18, 22, 21],
        [2, 3, 7, 6, 18, 19, 23, 22],
        [3, 4, 8, 7, 19, 20, 24, 23],
        [5, 6, 10, 9, 21, 22, 26, 25],
        [6, 7, 11, 10, 22, 23, 27, 26],
        [7, 8, 12, 11, 23, 24, 28, 27],
        [9, 10, 14, 13, 25, 26, 30, 29],
        [10, 11, 15, 14, 26, 27, 31, 30],
        [11, 12, 16, 15, 27, 28, 32, 31],
        [17, 18, 22, 21, 33, 34, 38, 37],
        [18, 19, 23, 22, 34, 35, 39, 38],
        [19, 20, 24, 23, 35, 36, 40, 39],
        [21, 22, 26, 25, 37, 38, 42, 41],
        [22, 23, 27, 26, 38, 39, 43, 42],
        [23, 24, 28, 27, 39, 40, 44, 43],
        [25, 26, 30, 29, 41, 42, 46, 45],
        [26, 27, 31, 30, 42, 43, 47, 46],
        [27, 28, 32, 31, 43, 44, 48, 47],
        [33, 34, 38, 37, 49, 50, 54, 53],
        [34, 35, 39, 38, 50, 51, 55, 54],
        [35, 36, 40, 39, 51, 52, 56, 55],
        [37, 38, 42, 41, 53, 54, 58, 57],
        [38, 39, 43, 42, 54, 55, 59, 58],
        [39, 40, 44, 43, 55, 56, 60, 59],
        [41, 42, 46, 45, 57, 58, 62, 61],
        [42, 43, 47, 46, 58, 59, 63, 62],
        [43, 44, 48, 47, 59, 60, 64, 63],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [3.0, 2.0, 0.0],
        [0.0, 3.0, 0.0],
        [1.0, 3.0, 0.0],
        [2.0, 3.0, 0.0],
        [3.0, 3.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [3.0, 2.0, 1.0],
        [0.0, 3.0, 1.0],
        [1.0, 3.0, 1.0],
        [2.0, 3.0, 1.0],
        [3.0, 3.0, 1.0],
        [0.0, 0.0, 2.0],
        [1.0, 0.0, 2.0],
        [2.0, 0.0, 2.0],
        [3.0, 0.0, 2.0],
        [0.0, 1.0, 2.0],
        [1.0, 1.0, 2.0],
        [2.0, 1.0, 2.0],
        [3.0, 1.0, 2.0],
        [0.0, 2.0, 2.0],
        [1.0, 2.0, 2.0],
        [2.0, 2.0, 2.0],
        [3.0, 2.0, 2.0],
        [0.0, 3.0, 2.0],
        [1.0, 3.0, 2.0],
        [2.0, 3.0, 2.0],
        [3.0, 3.0, 2.0],
        [0.0, 0.0, 3.0],
        [1.0, 0.0, 3.0],
        [2.0, 0.0, 3.0],
        [3.0, 0.0, 3.0],
        [0.0, 1.0, 3.0],
        [1.0, 1.0, 3.0],
        [2.0, 1.0, 3.0],
        [3.0, 1.0, 3.0],
        [0.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [2.0, 2.0, 3.0],
        [3.0, 2.0, 3.0],
        [0.0, 3.0, 3.0],
        [1.0, 3.0, 3.0],
        [2.0, 3.0, 3.0],
        [3.0, 3.0, 3.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3],
        vec![1, 4],
        vec![1, 2, 4, 5],
        vec![2, 3, 5, 6],
        vec![3, 6],
        vec![4, 7],
        vec![4, 5, 7, 8],
        vec![5, 6, 8, 9],
        vec![6, 9],
        vec![7],
        vec![7, 8],
        vec![8, 9],
        vec![9],
        vec![1, 10],
        vec![1, 2, 10, 11],
        vec![2, 3, 11, 12],
        vec![3, 12],
        vec![1, 4, 10, 13],
        vec![1, 2, 4, 5, 10, 11, 13, 14],
        vec![2, 3, 5, 6, 11, 12, 14, 15],
        vec![3, 6, 12, 15],
        vec![4, 7, 13, 16],
        vec![4, 5, 7, 8, 13, 14, 16, 17],
        vec![5, 6, 8, 9, 14, 15, 17, 18],
        vec![6, 9, 15, 18],
        vec![7, 16],
        vec![7, 8, 16, 17],
        vec![8, 9, 17, 18],
        vec![9, 18],
        vec![10, 19],
        vec![10, 11, 19, 20],
        vec![11, 12, 20, 21],
        vec![12, 21],
        vec![10, 13, 19, 22],
        vec![10, 11, 13, 14, 19, 20, 22, 23],
        vec![11, 12, 14, 15, 20, 21, 23, 24],
        vec![12, 15, 21, 24],
        vec![13, 16, 22, 25],
        vec![13, 14, 16, 17, 22, 23, 25, 26],
        vec![14, 15, 17, 18, 23, 24, 26, 27],
        vec![15, 18, 24, 27],
        vec![16, 25],
        vec![16, 17, 25, 26],
        vec![17, 18, 26, 27],
        vec![18, 27],
        vec![19],
        vec![19, 20],
        vec![20, 21],
        vec![21],
        vec![19, 22],
        vec![19, 20, 22, 23],
        vec![20, 21, 23, 24],
        vec![21, 24],
        vec![22, 25],
        vec![22, 23, 25, 26],
        vec![23, 24, 26, 27],
        vec![24, 27],
        vec![25],
        vec![25, 26],
        vec![26, 27],
        vec![27],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 5, 17],
        vec![1, 3, 6, 18],
        vec![2, 4, 7, 19],
        vec![3, 8, 20],
        vec![1, 6, 9, 21],
        vec![2, 5, 7, 10, 22],
        vec![3, 6, 8, 11, 23],
        vec![4, 7, 12, 24],
        vec![5, 10, 13, 25],
        vec![6, 9, 11, 14, 26],
        vec![7, 10, 12, 15, 27],
        vec![8, 11, 16, 28],
        vec![9, 14, 29],
        vec![10, 13, 15, 30],
        vec![11, 14, 16, 31],
        vec![12, 15, 32],
        vec![1, 18, 21, 33],
        vec![2, 17, 19, 22, 34],
        vec![3, 18, 20, 23, 35],
        vec![4, 19, 24, 36],
        vec![5, 17, 22, 25, 37],
        vec![6, 18, 21, 23, 26, 38],
        vec![7, 19, 22, 24, 27, 39],
        vec![8, 20, 23, 28, 40],
        vec![9, 21, 26, 29, 41],
        vec![10, 22, 25, 27, 30, 42],
        vec![11, 23, 26, 28, 31, 43],
        vec![12, 24, 27, 32, 44],
        vec![13, 25, 30, 45],
        vec![14, 26, 29, 31, 46],
        vec![15, 27, 30, 32, 47],
        vec![16, 28, 31, 48],
        vec![17, 34, 37, 49],
        vec![18, 33, 35, 38, 50],
        vec![19, 34, 36, 39, 51],
        vec![20, 35, 40, 52],
        vec![21, 33, 38, 41, 53],
        vec![22, 34, 37, 39, 42, 54],
        vec![23, 35, 38, 40, 43, 55],
        vec![24, 36, 39, 44, 56],
        vec![25, 37, 42, 45, 57],
        vec![26, 38, 41, 43, 46, 58],
        vec![27, 39, 42, 44, 47, 59],
        vec![28, 40, 43, 48, 60],
        vec![29, 41, 46, 61],
        vec![30, 42, 45, 47, 62],
        vec![31, 43, 46, 48, 63],
        vec![32, 44, 47, 64],
        vec![33, 50, 53],
        vec![34, 49, 51, 54],
        vec![35, 50, 52, 55],
        vec![36, 51, 56],
        vec![37, 49, 54, 57],
        vec![38, 50, 53, 55, 58],
        vec![39, 51, 54, 56, 59],
        vec![40, 52, 55, 60],
        vec![41, 53, 58, 61],
        vec![42, 54, 57, 59, 62],
        vec![43, 55, 58, 60, 63],
        vec![44, 56, 59, 64],
        vec![45, 57, 62],
        vec![46, 58, 61, 63],
        vec![47, 59, 62, 64],
        vec![48, 60, 63],
    ];
    let interface_nodes_gold = vec![22, 23, 26, 27, 38, 39, 42, 43];
    let exterior_nodes_gold = (1..=64)
        .filter(|node| !interface_nodes_gold.contains(node))
        .collect();
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn bracket() {
    let element_blocks = vec![1; 12];
    let element_node_connectivity = vec![
        [1, 2, 7, 6, 22, 23, 28, 27],
        [2, 3, 8, 7, 23, 24, 29, 28],
        [3, 4, 9, 8, 24, 25, 30, 29],
        [4, 5, 10, 9, 25, 26, 31, 30],
        [6, 7, 12, 11, 27, 28, 33, 32],
        [7, 8, 13, 12, 28, 29, 34, 33],
        [8, 9, 14, 13, 29, 30, 35, 34],
        [9, 10, 15, 14, 30, 31, 36, 35],
        [11, 12, 17, 16, 32, 33, 38, 37],
        [12, 13, 18, 17, 33, 34, 39, 38],
        [16, 17, 20, 19, 37, 38, 41, 40],
        [17, 18, 21, 20, 38, 39, 42, 41],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [3.0, 2.0, 0.0],
        [4.0, 2.0, 0.0],
        [0.0, 3.0, 0.0],
        [1.0, 3.0, 0.0],
        [2.0, 3.0, 0.0],
        [0.0, 4.0, 0.0],
        [1.0, 4.0, 0.0],
        [2.0, 4.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [3.0, 2.0, 1.0],
        [4.0, 2.0, 1.0],
        [0.0, 3.0, 1.0],
        [1.0, 3.0, 1.0],
        [2.0, 3.0, 1.0],
        [0.0, 4.0, 1.0],
        [1.0, 4.0, 1.0],
        [2.0, 4.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1, 5],
        vec![1, 2, 5, 6],
        vec![2, 3, 6, 7],
        vec![3, 4, 7, 8],
        vec![4, 8],
        vec![5, 9],
        vec![5, 6, 9, 10],
        vec![6, 7, 10],
        vec![7, 8],
        vec![8],
        vec![9, 11],
        vec![9, 10, 11, 12],
        vec![10, 12],
        vec![11],
        vec![11, 12],
        vec![12],
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1, 5],
        vec![1, 2, 5, 6],
        vec![2, 3, 6, 7],
        vec![3, 4, 7, 8],
        vec![4, 8],
        vec![5, 9],
        vec![5, 6, 9, 10],
        vec![6, 7, 10],
        vec![7, 8],
        vec![8],
        vec![9, 11],
        vec![9, 10, 11, 12],
        vec![10, 12],
        vec![11],
        vec![11, 12],
        vec![12],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 6, 22],
        vec![1, 3, 7, 23],
        vec![2, 4, 8, 24],
        vec![3, 5, 9, 25],
        vec![4, 10, 26],
        vec![1, 7, 11, 27],
        vec![2, 6, 8, 12, 28],
        vec![3, 7, 9, 13, 29],
        vec![4, 8, 10, 14, 30],
        vec![5, 9, 15, 31],
        vec![6, 12, 16, 32],
        vec![7, 11, 13, 17, 33],
        vec![8, 12, 14, 18, 34],
        vec![9, 13, 15, 35],
        vec![10, 14, 36],
        vec![11, 17, 19, 37],
        vec![12, 16, 18, 20, 38],
        vec![13, 17, 21, 39],
        vec![16, 20, 40],
        vec![17, 19, 21, 41],
        vec![18, 20, 42],
        vec![1, 23, 27],
        vec![2, 22, 24, 28],
        vec![3, 23, 25, 29],
        vec![4, 24, 26, 30],
        vec![5, 25, 31],
        vec![6, 22, 28, 32],
        vec![7, 23, 27, 29, 33],
        vec![8, 24, 28, 30, 34],
        vec![9, 25, 29, 31, 35],
        vec![10, 26, 30, 36],
        vec![11, 27, 33, 37],
        vec![12, 28, 32, 34, 38],
        vec![13, 29, 33, 35, 39],
        vec![14, 30, 34, 36],
        vec![15, 31, 35],
        vec![16, 32, 38, 40],
        vec![17, 33, 37, 39, 41],
        vec![18, 34, 38, 42],
        vec![19, 37, 41],
        vec![20, 38, 40, 42],
        vec![21, 39, 41],
    ];
    let exterior_nodes_gold = (1..=42).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    let laplacian_gold = Coordinates::new(&[
        [ONE_THIRD, ONE_THIRD, ONE_THIRD],
        [ZERO, ONE_FOURTH, ONE_FOURTH],
        [ZERO, ONE_FOURTH, ONE_FOURTH],
        [ZERO, ONE_FOURTH, ONE_FOURTH],
        [-ONE_THIRD, ONE_THIRD, ONE_THIRD],
        [ONE_FOURTH, ZERO, ONE_FOURTH],
        [ZERO, ZERO, ONE_FIFTH],
        [ZERO, ZERO, ONE_FIFTH],
        [ZERO, ZERO, ONE_FIFTH],
        [-ONE_FOURTH, ZERO, ONE_FOURTH],
        [ONE_FOURTH, ZERO, ONE_FOURTH],
        [ZERO, ZERO, ONE_FIFTH],
        [ZERO, ZERO, ONE_FIFTH],
        [ZERO, -ONE_FOURTH, ONE_FOURTH],
        [-ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [ONE_FOURTH, ZERO, ONE_FOURTH],
        [ZERO, ZERO, ONE_FIFTH],
        [-ONE_FOURTH, ZERO, ONE_FOURTH],
        [ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [ZERO, -ONE_FOURTH, ONE_FOURTH],
        [-ONE_THIRD, -ONE_THIRD, ONE_THIRD],
        [ONE_THIRD, ONE_THIRD, -ONE_THIRD],
        [ZERO, ONE_FOURTH, -ONE_FOURTH],
        [ZERO, ONE_FOURTH, -ONE_FOURTH],
        [ZERO, ONE_FOURTH, -ONE_FOURTH],
        [-ONE_THIRD, ONE_THIRD, -ONE_THIRD],
        [ONE_FOURTH, ZERO, -ONE_FOURTH],
        [ZERO, ZERO, -ONE_FIFTH],
        [ZERO, ZERO, -ONE_FIFTH],
        [ZERO, ZERO, -ONE_FIFTH],
        [-ONE_FOURTH, ZERO, -ONE_FOURTH],
        [ONE_FOURTH, ZERO, -ONE_FOURTH],
        [ZERO, ZERO, -ONE_FIFTH],
        [ZERO, ZERO, -ONE_FIFTH],
        [ZERO, -ONE_FOURTH, -ONE_FOURTH],
        [-ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
        [ONE_FOURTH, ZERO, -ONE_FOURTH],
        [ZERO, ZERO, -ONE_FIFTH],
        [-ONE_FOURTH, ZERO, -ONE_FOURTH],
        [ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
        [ZERO, -ONE_FOURTH, -ONE_FOURTH],
        [-ONE_THIRD, -ONE_THIRD, -ONE_THIRD],
    ]);
    let smoothed_coordinates_gold = vec![
        Coordinates::new(&[
            [0.1, 0.1, 0.1],
            [1.0, 0.075, 0.075],
            [2.0, 0.075, 0.075],
            [3.0, 0.075, 0.075],
            [3.9, 0.1, 0.1],
            [0.075, 1.0, 0.075],
            [1.0, 1.0, 0.06],
            [2.0, 1.0, 0.06],
            [3.0, 1.0, 0.06],
            [3.925, 1.0, 0.075],
            [0.075, 2.0, 0.075],
            [1.0, 2.0, 0.06],
            [2.0, 2.0, 0.06],
            [3.0, 1.925, 0.075],
            [3.9, 1.9, 0.1],
            [0.075, 3.0, 0.075],
            [1.0, 3.0, 0.06],
            [1.925, 3.0, 0.075],
            [0.1, 3.9, 0.1],
            [1.0, 3.925, 0.075],
            [1.9, 3.9, 0.1],
            [0.1, 0.1, 0.9],
            [1.0, 0.075, 0.925],
            [2.0, 0.075, 0.925],
            [3.0, 0.075, 0.925],
            [3.9, 0.1, 0.9],
            [0.075, 1.0, 0.925],
            [1.0, 1.0, 0.94],
            [2.0, 1.0, 0.94],
            [3.0, 1.0, 0.94],
            [3.925, 1.0, 0.925],
            [0.075, 2.0, 0.925],
            [1.0, 2.0, 0.94],
            [2.0, 2.0, 0.94],
            [3.0, 1.925, 0.925],
            [3.9, 1.9, 0.9],
            [0.075, 3.0, 0.925],
            [1.0, 3.0, 0.94],
            [1.925, 3.0, 0.925],
            [0.1, 3.9, 0.9],
            [1.0, 3.925, 0.925],
            [1.9, 3.9, 0.9],
        ]),
        Coordinates::new(&[
            [0.1875, 0.1875, 0.175],
            [1.0075, 0.14625, 0.1395],
            [2.0, 0.144375, 0.137625],
            [2.9925, 0.14625, 0.1395],
            [3.8125, 0.1875, 0.175],
            [0.14625, 1.0075, 0.1395],
            [1.0045, 1.0045, 0.1146],
            [2.0, 1.0045, 0.1137],
            [2.9955, 1.0, 0.1155],
            [3.851875, 1.0, 0.141375],
            [0.144375, 2.0, 0.137625],
            [1.0045, 2.0, 0.1137],
            [1.9955, 1.9955, 0.1146],
            [2.9925, 1.859375, 0.138375],
            [3.8125, 1.8125, 0.175],
            [0.14625, 2.9925, 0.1395],
            [1.0, 2.9955, 0.1155],
            [1.859375, 2.9925, 0.138375],
            [0.1875, 3.8125, 0.175],
            [1.0, 3.851875, 0.141375],
            [1.8125, 3.8125, 0.175],
            [0.1875, 0.1875, 0.8250],
            [1.0075, 0.14625, 0.8605],
            [2.0, 0.144375, 0.862375],
            [2.9925, 0.14625, 0.8605],
            [3.8125, 0.1875, 0.8250],
            [0.14625, 1.0075, 0.8605],
            [1.0045, 1.0045, 0.8854],
            [2.0, 1.0045, 0.8863],
            [2.9955, 1.0, 0.8845],
            [3.851875, 1.0, 0.858625],
            [0.144375, 2.0, 0.862375],
            [1.0045, 2.0, 0.8863],
            [1.9955, 1.9955, 0.8854],
            [2.9925, 1.859375, 0.861625],
            [3.8125, 1.8125, 0.8250],
            [0.14625, 2.9925, 0.8605],
            [1.0, 2.9955, 0.8845],
            [1.859375, 2.9925, 0.861625],
            [0.1875, 3.8125, 0.8250],
            [1.0, 3.851875, 0.858625],
            [1.8125, 3.8125, 0.8250],
        ]),
    ];
    test_finite_elements(
        element_blocks.clone(),
        element_node_connectivity.clone(),
        nodal_coordinates.clone_foo(),
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        Some(laplacian_gold),
        Some(smoothed_coordinates_gold),
        nodal_influencers_gold,
    );
    let cos_15 = 15.0_f64.to_radians().cos();
    let cos_30 = 30.0_f64.to_radians().cos();
    let sin_15 = 15.0_f64.to_radians().sin();
    let sin_30 = 30.0_f64.to_radians().sin();
    let prescribed_nodes_homogeneous = vec![
        1, 2, 3, 4, 5, 6, 11, 16, 19, 22, 23, 24, 25, 26, 27, 32, 37, 40,
    ];
    let prescribed_nodes_inhomogeneous = vec![10, 15, 20, 21, 31, 36, 41, 42];
    let prescribed_nodes_inhomogeneous_coordinates = Coordinates::new(&[
        [4.5 * cos_15, 4.5 * sin_15, 0.0],
        [4.5 * cos_30, 4.5 * sin_30, 0.0],
        [1.5, 4.0, 0.0],
        [3.5, 4.0, 0.0],
        [4.5 * cos_15, 4.5 * sin_15, 1.0],
        [4.5 * cos_30, 4.5 * sin_30, 1.0],
        [1.5, 4.0, 1.0],
        [3.5, 4.0, 1.0],
    ]);
    let mut finite_elements = HexahedralFiniteElements::from_data(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    );
    finite_elements.node_element_connectivity().unwrap();
    finite_elements.node_node_connectivity().unwrap();
    finite_elements.nodal_hierarchy().unwrap();
    finite_elements
        .set_prescribed_nodes(
            Some(prescribed_nodes_homogeneous),
            Some((
                prescribed_nodes_inhomogeneous_coordinates,
                prescribed_nodes_inhomogeneous,
            )),
        )
        .unwrap();
    finite_elements.nodal_influencers();
    finite_elements
        .smooth(Smoothing::Laplacian(10, SMOOTHING_SCALE))
        .unwrap();
    finite_elements
        .get_nodal_coordinates()
        .iter()
        .zip(
            vec![
                vec![0.0, 0.0, 0.0],
                vec![1.0, 0.0, 0.0],
                vec![2.0, 0.0, 0.0],
                vec![3.0, 0.0, 0.0],
                vec![4.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0],
                vec![1.0076218690550747, 0.9988829259123082, 0.24593434133370803],
                vec![2.0218051968023, 0.993985105791881, 0.2837944855813176],
                vec![3.0816593568068398, 0.9931227966186256, 0.24898414051620496],
                vec![4.346666218300808, 1.1646857029613433, 0.0],
                vec![0.0, 2.0, 0.0],
                vec![1.0346002406957664, 1.992982526945126, 0.2837944855813176],
                vec![2.0408618916639916, 1.9528647520642073, 0.3332231502067546],
                vec![2.9955771790244468, 1.7619821132207711, 0.29909606343914835],
                vec![3.897114317029974, 2.2499999999999996, 0.0],
                vec![0.0, 3.0, 0.0],
                vec![1.157261281731803, 2.9982665159532105, 0.24898414051620493],
                vec![2.1973691292662734, 2.991054895165017, 0.29909606343914835],
                vec![0.0, 4.0, 0.0],
                vec![1.5, 4.0, 0.0],
                vec![3.5, 4.0, 0.0],
                vec![0.0, 0.0, 1.0],
                vec![1.0, 0.0, 1.0],
                vec![2.0, 0.0, 1.0],
                vec![3.0, 0.0, 1.0],
                vec![4.0, 0.0, 1.0],
                vec![0.0, 1.0, 1.0],
                vec![1.0076218690550747, 0.9988829259123082, 0.7540656586662919],
                vec![2.0218051968023, 0.993985105791881, 0.7162055144186824],
                vec![3.0816593568068398, 0.9931227966186257, 0.7510158594837951],
                vec![4.346666218300808, 1.1646857029613433, 1.0],
                vec![0.0, 2.0, 1.0],
                vec![1.0346002406957664, 1.9929825269451262, 0.7162055144186824],
                vec![2.0408618916639916, 1.9528647520642073, 0.6667768497932453],
                vec![2.9955771790244468, 1.7619821132207711, 0.7009039365608517],
                vec![3.897114317029974, 2.2499999999999996, 1.0],
                vec![0.0, 3.0, 1.0],
                vec![1.157261281731803, 2.9982665159532105, 0.751015859483795],
                vec![2.1973691292662734, 2.991054895165017, 0.7009039365608516],
                vec![0.0, 4.0, 1.0],
                vec![1.5, 4.0, 1.0],
                vec![3.5, 4.0, 1.0],
            ]
            .iter(),
        )
        .for_each(|(data, gold)| {
            data.iter()
                .zip(gold.iter())
                .for_each(|(data_entry, gold_entry)| assert_eq!(data_entry, gold_entry))
        });
}

#[test]
fn letter_f() {
    let element_blocks = vec![11; 8];
    let element_node_connectivity = vec![
        [1, 2, 4, 3, 19, 20, 22, 21],
        [3, 4, 6, 5, 21, 22, 24, 23],
        [5, 6, 9, 8, 23, 24, 27, 26],
        [6, 7, 10, 9, 24, 25, 28, 27],
        [8, 9, 12, 11, 26, 27, 30, 29],
        [11, 12, 16, 15, 29, 30, 34, 33],
        [12, 13, 17, 16, 30, 31, 35, 34],
        [13, 14, 18, 17, 31, 32, 36, 35],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [0.0, 3.0, 0.0],
        [1.0, 3.0, 0.0],
        [2.0, 3.0, 0.0],
        [0.0, 4.0, 0.0],
        [1.0, 4.0, 0.0],
        [2.0, 4.0, 0.0],
        [3.0, 4.0, 0.0],
        [0.0, 5.0, 0.0],
        [1.0, 5.0, 0.0],
        [2.0, 5.0, 0.0],
        [3.0, 5.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [0.0, 3.0, 1.0],
        [1.0, 3.0, 1.0],
        [2.0, 3.0, 1.0],
        [0.0, 4.0, 1.0],
        [1.0, 4.0, 1.0],
        [2.0, 4.0, 1.0],
        [3.0, 4.0, 1.0],
        [0.0, 5.0, 1.0],
        [1.0, 5.0, 1.0],
        [2.0, 5.0, 1.0],
        [3.0, 5.0, 1.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1],
        vec![1, 2],
        vec![1, 2],
        vec![2, 3],
        vec![2, 3, 4],
        vec![4],
        vec![3, 5],
        vec![3, 4, 5],
        vec![4],
        vec![5, 6],
        vec![5, 6, 7],
        vec![7, 8],
        vec![8],
        vec![6],
        vec![6, 7],
        vec![7, 8],
        vec![8],
        vec![1],
        vec![1],
        vec![1, 2],
        vec![1, 2],
        vec![2, 3],
        vec![2, 3, 4],
        vec![4],
        vec![3, 5],
        vec![3, 4, 5],
        vec![4],
        vec![5, 6],
        vec![5, 6, 7],
        vec![7, 8],
        vec![8],
        vec![6],
        vec![6, 7],
        vec![7, 8],
        vec![8],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 3, 19],
        vec![1, 4, 20],
        vec![1, 4, 5, 21],
        vec![2, 3, 6, 22],
        vec![3, 6, 8, 23],
        vec![4, 5, 7, 9, 24],
        vec![6, 10, 25],
        vec![5, 9, 11, 26],
        vec![6, 8, 10, 12, 27],
        vec![7, 9, 28],
        vec![8, 12, 15, 29],
        vec![9, 11, 13, 16, 30],
        vec![12, 14, 17, 31],
        vec![13, 18, 32],
        vec![11, 16, 33],
        vec![12, 15, 17, 34],
        vec![13, 16, 18, 35],
        vec![14, 17, 36],
        vec![1, 20, 21],
        vec![2, 19, 22],
        vec![3, 19, 22, 23],
        vec![4, 20, 21, 24],
        vec![5, 21, 24, 26],
        vec![6, 22, 23, 25, 27],
        vec![7, 24, 28],
        vec![8, 23, 27, 29],
        vec![9, 24, 26, 28, 30],
        vec![10, 25, 27],
        vec![11, 26, 30, 33],
        vec![12, 27, 29, 31, 34],
        vec![13, 30, 32, 35],
        vec![14, 31, 36],
        vec![15, 29, 34],
        vec![16, 30, 33, 35],
        vec![17, 31, 34, 36],
        vec![18, 32, 35],
    ];
    let exterior_nodes_gold = (1..=36).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn letter_f_3d() {
    let element_blocks = vec![11; 39];
    let element_node_connectivity = vec![
        [1, 2, 7, 6, 31, 32, 37, 36],
        [2, 3, 8, 7, 32, 33, 38, 37],
        [3, 4, 9, 8, 33, 34, 39, 38],
        [4, 5, 10, 9, 34, 35, 40, 39],
        [6, 7, 12, 11, 36, 37, 42, 41],
        [7, 8, 13, 12, 37, 38, 43, 42],
        [8, 9, 14, 13, 38, 39, 44, 43],
        [9, 10, 15, 14, 39, 40, 45, 44],
        [11, 12, 17, 16, 41, 42, 47, 46],
        [12, 13, 18, 17, 42, 43, 48, 47],
        [13, 14, 19, 18, 43, 44, 49, 48],
        [14, 15, 20, 19, 44, 45, 50, 49],
        [16, 17, 22, 21, 46, 47, 52, 51],
        [17, 18, 23, 22, 47, 48, 53, 52],
        [18, 19, 24, 23, 48, 49, 54, 53],
        [19, 20, 25, 24, 49, 50, 55, 54],
        [21, 22, 27, 26, 51, 52, 57, 56],
        [22, 23, 28, 27, 52, 53, 58, 57],
        [23, 24, 29, 28, 53, 54, 59, 58],
        [24, 25, 30, 29, 54, 55, 60, 59],
        [31, 32, 37, 36, 61, 62, 64, 63],
        [36, 37, 42, 41, 63, 64, 66, 65],
        [41, 42, 47, 46, 65, 66, 71, 70],
        [42, 43, 48, 47, 66, 67, 72, 71],
        [43, 44, 49, 48, 67, 68, 73, 72],
        [44, 45, 50, 49, 68, 69, 74, 73],
        [46, 47, 52, 51, 70, 71, 76, 75],
        [51, 52, 57, 56, 75, 76, 81, 80],
        [52, 53, 58, 57, 76, 77, 82, 81],
        [53, 54, 59, 58, 77, 78, 83, 82],
        [54, 55, 60, 59, 78, 79, 84, 83],
        [61, 62, 64, 63, 85, 86, 88, 87],
        [63, 64, 66, 65, 87, 88, 90, 89],
        [65, 66, 71, 70, 89, 90, 92, 91],
        [70, 71, 76, 75, 91, 92, 94, 93],
        [75, 76, 81, 80, 93, 94, 99, 98],
        [76, 77, 82, 81, 94, 95, 100, 99],
        [77, 78, 83, 82, 95, 96, 101, 100],
        [78, 79, 84, 83, 96, 97, 102, 101],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [3.0, 2.0, 0.0],
        [4.0, 2.0, 0.0],
        [0.0, 3.0, 0.0],
        [1.0, 3.0, 0.0],
        [2.0, 3.0, 0.0],
        [3.0, 3.0, 0.0],
        [4.0, 3.0, 0.0],
        [0.0, 4.0, 0.0],
        [1.0, 4.0, 0.0],
        [2.0, 4.0, 0.0],
        [3.0, 4.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 5.0, 0.0],
        [1.0, 5.0, 0.0],
        [2.0, 5.0, 0.0],
        [3.0, 5.0, 0.0],
        [4.0, 5.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [3.0, 2.0, 1.0],
        [4.0, 2.0, 1.0],
        [0.0, 3.0, 1.0],
        [1.0, 3.0, 1.0],
        [2.0, 3.0, 1.0],
        [3.0, 3.0, 1.0],
        [4.0, 3.0, 1.0],
        [0.0, 4.0, 1.0],
        [1.0, 4.0, 1.0],
        [2.0, 4.0, 1.0],
        [3.0, 4.0, 1.0],
        [4.0, 4.0, 1.0],
        [0.0, 5.0, 1.0],
        [1.0, 5.0, 1.0],
        [2.0, 5.0, 1.0],
        [3.0, 5.0, 1.0],
        [4.0, 5.0, 1.0],
        [0.0, 0.0, 2.0],
        [1.0, 0.0, 2.0],
        [0.0, 1.0, 2.0],
        [1.0, 1.0, 2.0],
        [0.0, 2.0, 2.0],
        [1.0, 2.0, 2.0],
        [2.0, 2.0, 2.0],
        [3.0, 2.0, 2.0],
        [4.0, 2.0, 2.0],
        [0.0, 3.0, 2.0],
        [1.0, 3.0, 2.0],
        [2.0, 3.0, 2.0],
        [3.0, 3.0, 2.0],
        [4.0, 3.0, 2.0],
        [0.0, 4.0, 2.0],
        [1.0, 4.0, 2.0],
        [2.0, 4.0, 2.0],
        [3.0, 4.0, 2.0],
        [4.0, 4.0, 2.0],
        [0.0, 5.0, 2.0],
        [1.0, 5.0, 2.0],
        [2.0, 5.0, 2.0],
        [3.0, 5.0, 2.0],
        [4.0, 5.0, 2.0],
        [0.0, 0.0, 3.0],
        [1.0, 0.0, 3.0],
        [0.0, 1.0, 3.0],
        [1.0, 1.0, 3.0],
        [0.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [0.0, 3.0, 3.0],
        [1.0, 3.0, 3.0],
        [0.0, 4.0, 3.0],
        [1.0, 4.0, 3.0],
        [2.0, 4.0, 3.0],
        [3.0, 4.0, 3.0],
        [4.0, 4.0, 3.0],
        [0.0, 5.0, 3.0],
        [1.0, 5.0, 3.0],
        [2.0, 5.0, 3.0],
        [3.0, 5.0, 3.0],
        [4.0, 5.0, 3.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1, 5],
        vec![1, 2, 5, 6],
        vec![2, 3, 6, 7],
        vec![3, 4, 7, 8],
        vec![4, 8],
        vec![5, 9],
        vec![5, 6, 9, 10],
        vec![6, 7, 10, 11],
        vec![7, 8, 11, 12],
        vec![8, 12],
        vec![9, 13],
        vec![9, 10, 13, 14],
        vec![10, 11, 14, 15],
        vec![11, 12, 15, 16],
        vec![12, 16],
        vec![13, 17],
        vec![13, 14, 17, 18],
        vec![14, 15, 18, 19],
        vec![15, 16, 19, 20],
        vec![16, 20],
        vec![17],
        vec![17, 18],
        vec![18, 19],
        vec![19, 20],
        vec![20],
        vec![1, 21],
        vec![1, 2, 21],
        vec![2, 3],
        vec![3, 4],
        vec![4],
        vec![1, 5, 21, 22],
        vec![1, 2, 5, 6, 21, 22],
        vec![2, 3, 6, 7],
        vec![3, 4, 7, 8],
        vec![4, 8],
        vec![5, 9, 22, 23],
        vec![5, 6, 9, 10, 22, 23, 24],
        vec![6, 7, 10, 11, 24, 25],
        vec![7, 8, 11, 12, 25, 26],
        vec![8, 12, 26],
        vec![9, 13, 23, 27],
        vec![9, 10, 13, 14, 23, 24, 27],
        vec![10, 11, 14, 15, 24, 25],
        vec![11, 12, 15, 16, 25, 26],
        vec![12, 16, 26],
        vec![13, 17, 27, 28],
        vec![13, 14, 17, 18, 27, 28, 29],
        vec![14, 15, 18, 19, 29, 30],
        vec![15, 16, 19, 20, 30, 31],
        vec![16, 20, 31],
        vec![17, 28],
        vec![17, 18, 28, 29],
        vec![18, 19, 29, 30],
        vec![19, 20, 30, 31],
        vec![20, 31],
        vec![21, 32],
        vec![21, 32],
        vec![21, 22, 32, 33],
        vec![21, 22, 32, 33],
        vec![22, 23, 33, 34],
        vec![22, 23, 24, 33, 34],
        vec![24, 25],
        vec![25, 26],
        vec![26],
        vec![23, 27, 34, 35],
        vec![23, 24, 27, 34, 35],
        vec![24, 25],
        vec![25, 26],
        vec![26],
        vec![27, 28, 35, 36],
        vec![27, 28, 29, 35, 36, 37],
        vec![29, 30, 37, 38],
        vec![30, 31, 38, 39],
        vec![31, 39],
        vec![28, 36],
        vec![28, 29, 36, 37],
        vec![29, 30, 37, 38],
        vec![30, 31, 38, 39],
        vec![31, 39],
        vec![32],
        vec![32],
        vec![32, 33],
        vec![32, 33],
        vec![33, 34],
        vec![33, 34],
        vec![34, 35],
        vec![34, 35],
        vec![35, 36],
        vec![35, 36, 37],
        vec![37, 38],
        vec![38, 39],
        vec![39],
        vec![36],
        vec![36, 37],
        vec![37, 38],
        vec![38, 39],
        vec![39],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 6, 31],
        vec![1, 3, 7, 32],
        vec![2, 4, 8, 33],
        vec![3, 5, 9, 34],
        vec![4, 10, 35],
        vec![1, 7, 11, 36],
        vec![2, 6, 8, 12, 37],
        vec![3, 7, 9, 13, 38],
        vec![4, 8, 10, 14, 39],
        vec![5, 9, 15, 40],
        vec![6, 12, 16, 41],
        vec![7, 11, 13, 17, 42],
        vec![8, 12, 14, 18, 43],
        vec![9, 13, 15, 19, 44],
        vec![10, 14, 20, 45],
        vec![11, 17, 21, 46],
        vec![12, 16, 18, 22, 47],
        vec![13, 17, 19, 23, 48],
        vec![14, 18, 20, 24, 49],
        vec![15, 19, 25, 50],
        vec![16, 22, 26, 51],
        vec![17, 21, 23, 27, 52],
        vec![18, 22, 24, 28, 53],
        vec![19, 23, 25, 29, 54],
        vec![20, 24, 30, 55],
        vec![21, 27, 56],
        vec![22, 26, 28, 57],
        vec![23, 27, 29, 58],
        vec![24, 28, 30, 59],
        vec![25, 29, 60],
        vec![1, 32, 36, 61],
        vec![2, 31, 33, 37, 62],
        vec![3, 32, 34, 38],
        vec![4, 33, 35, 39],
        vec![5, 34, 40],
        vec![6, 31, 37, 41, 63],
        vec![7, 32, 36, 38, 42, 64],
        vec![8, 33, 37, 39, 43],
        vec![9, 34, 38, 40, 44],
        vec![10, 35, 39, 45],
        vec![11, 36, 42, 46, 65],
        vec![12, 37, 41, 43, 47, 66],
        vec![13, 38, 42, 44, 48, 67],
        vec![14, 39, 43, 45, 49, 68],
        vec![15, 40, 44, 50, 69],
        vec![16, 41, 47, 51, 70],
        vec![17, 42, 46, 48, 52, 71],
        vec![18, 43, 47, 49, 53, 72],
        vec![19, 44, 48, 50, 54, 73],
        vec![20, 45, 49, 55, 74],
        vec![21, 46, 52, 56, 75],
        vec![22, 47, 51, 53, 57, 76],
        vec![23, 48, 52, 54, 58, 77],
        vec![24, 49, 53, 55, 59, 78],
        vec![25, 50, 54, 60, 79],
        vec![26, 51, 57, 80],
        vec![27, 52, 56, 58, 81],
        vec![28, 53, 57, 59, 82],
        vec![29, 54, 58, 60, 83],
        vec![30, 55, 59, 84],
        vec![31, 62, 63, 85],
        vec![32, 61, 64, 86],
        vec![36, 61, 64, 65, 87],
        vec![37, 62, 63, 66, 88],
        vec![41, 63, 66, 70, 89],
        vec![42, 64, 65, 67, 71, 90],
        vec![43, 66, 68, 72],
        vec![44, 67, 69, 73],
        vec![45, 68, 74],
        vec![46, 65, 71, 75, 91],
        vec![47, 66, 70, 72, 76, 92],
        vec![48, 67, 71, 73],
        vec![49, 68, 72, 74],
        vec![50, 69, 73],
        vec![51, 70, 76, 80, 93],
        vec![52, 71, 75, 77, 81, 94],
        vec![53, 76, 78, 82, 95],
        vec![54, 77, 79, 83, 96],
        vec![55, 78, 84, 97],
        vec![56, 75, 81, 98],
        vec![57, 76, 80, 82, 99],
        vec![58, 77, 81, 83, 100],
        vec![59, 78, 82, 84, 101],
        vec![60, 79, 83, 102],
        vec![61, 86, 87],
        vec![62, 85, 88],
        vec![63, 85, 88, 89],
        vec![64, 86, 87, 90],
        vec![65, 87, 90, 91],
        vec![66, 88, 89, 92],
        vec![70, 89, 92, 93],
        vec![71, 90, 91, 94],
        vec![75, 91, 94, 98],
        vec![76, 92, 93, 95, 99],
        vec![77, 94, 96, 100],
        vec![78, 95, 97, 101],
        vec![79, 96, 102],
        vec![80, 93, 99],
        vec![81, 94, 98, 100],
        vec![82, 95, 99, 101],
        vec![83, 96, 100, 102],
        vec![84, 97, 101],
    ];
    let exterior_nodes_gold = (1..=102).collect();
    let interface_nodes_gold = vec![];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn sparse() {
    let element_blocks = vec![
        2, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1,
        2, 1, 1, 1, 2, 2, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 1, 1, 1, 2, 1,
    ];
    let element_node_connectivity = vec![
        [1, 2, 4, 3, 29, 30, 36, 35],
        [3, 4, 10, 9, 35, 36, 42, 41],
        [5, 6, 12, 11, 37, 38, 44, 43],
        [6, 7, 13, 12, 38, 39, 45, 44],
        [8, 9, 15, 14, 40, 41, 47, 46],
        [9, 10, 16, 15, 41, 42, 48, 47],
        [11, 12, 18, 17, 43, 44, 50, 49],
        [15, 16, 22, 21, 47, 48, 54, 53],
        [17, 18, 24, 23, 49, 50, 56, 55],
        [18, 19, 25, 24, 50, 51, 57, 56],
        [20, 21, 27, 26, 52, 53, 59, 58],
        [21, 22, 28, 27, 53, 54, 60, 59],
        [31, 32, 38, 37, 64, 65, 71, 70],
        [32, 33, 39, 38, 65, 66, 72, 71],
        [34, 35, 41, 40, 67, 68, 74, 73],
        [35, 36, 42, 41, 68, 69, 75, 74],
        [40, 41, 47, 46, 73, 74, 80, 79],
        [43, 44, 50, 49, 76, 77, 83, 82],
        [44, 45, 51, 50, 77, 78, 84, 83],
        [46, 47, 53, 52, 79, 80, 86, 85],
        [49, 50, 56, 55, 82, 83, 89, 88],
        [54, 55, 61, 60, 87, 88, 93, 92],
        [62, 63, 69, 68, 96, 97, 102, 101],
        [63, 64, 70, 69, 97, 98, 103, 102],
        [64, 65, 71, 70, 98, 99, 104, 103],
        [70, 71, 77, 76, 103, 104, 110, 109],
        [75, 76, 82, 81, 108, 109, 114, 113],
        [76, 77, 83, 82, 109, 110, 115, 114],
        [81, 82, 88, 87, 113, 114, 119, 118],
        [82, 83, 89, 88, 114, 115, 120, 119],
        [86, 87, 92, 91, 117, 118, 123, 122],
        [88, 89, 94, 93, 119, 120, 125, 124],
        [89, 90, 95, 94, 120, 121, 126, 125],
        [98, 99, 104, 103, 130, 131, 137, 136],
        [99, 100, 105, 104, 131, 132, 138, 137],
        [101, 102, 108, 107, 134, 135, 141, 140],
        [102, 103, 109, 108, 135, 136, 142, 141],
        [106, 107, 112, 111, 139, 140, 146, 145],
        [108, 109, 114, 113, 141, 142, 148, 147],
        [111, 112, 117, 116, 145, 146, 151, 150],
        [112, 113, 118, 117, 146, 147, 152, 151],
        [114, 115, 120, 119, 148, 149, 154, 153],
        [118, 119, 124, 123, 152, 153, 159, 158],
        [120, 121, 126, 125, 154, 155, 161, 160],
        [127, 128, 134, 133, 162, 163, 168, 167],
        [129, 130, 136, 135, 164, 165, 170, 169],
        [130, 131, 137, 136, 165, 166, 171, 170],
        [133, 134, 140, 139, 167, 168, 174, 173],
        [134, 135, 141, 140, 168, 169, 175, 174],
        [135, 136, 142, 141, 169, 170, 176, 175],
        [136, 137, 143, 142, 170, 171, 177, 176],
        [137, 138, 144, 143, 171, 172, 178, 177],
        [141, 142, 148, 147, 175, 176, 180, 179],
        [147, 148, 153, 152, 179, 180, 185, 184],
        [148, 149, 154, 153, 180, 181, 186, 185],
        [150, 151, 157, 156, 182, 183, 189, 188],
        [151, 152, 158, 157, 183, 184, 190, 189],
        [154, 155, 161, 160, 186, 187, 192, 191],
    ];
    let nodal_coordinates = Coordinates::new(&[
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 1.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        [5.0, 1.0, 0.0],
        [0.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [3.0, 2.0, 0.0],
        [4.0, 2.0, 0.0],
        [5.0, 2.0, 0.0],
        [0.0, 3.0, 0.0],
        [1.0, 3.0, 0.0],
        [2.0, 3.0, 0.0],
        [3.0, 3.0, 0.0],
        [4.0, 3.0, 0.0],
        [5.0, 3.0, 0.0],
        [0.0, 4.0, 0.0],
        [1.0, 4.0, 0.0],
        [2.0, 4.0, 0.0],
        [3.0, 4.0, 0.0],
        [4.0, 4.0, 0.0],
        [5.0, 4.0, 0.0],
        [0.0, 5.0, 0.0],
        [1.0, 5.0, 0.0],
        [2.0, 5.0, 0.0],
        [1.0, 0.0, 1.0],
        [2.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [5.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [3.0, 1.0, 1.0],
        [4.0, 1.0, 1.0],
        [5.0, 1.0, 1.0],
        [0.0, 2.0, 1.0],
        [1.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [3.0, 2.0, 1.0],
        [4.0, 2.0, 1.0],
        [5.0, 2.0, 1.0],
        [0.0, 3.0, 1.0],
        [1.0, 3.0, 1.0],
        [2.0, 3.0, 1.0],
        [3.0, 3.0, 1.0],
        [4.0, 3.0, 1.0],
        [5.0, 3.0, 1.0],
        [0.0, 4.0, 1.0],
        [1.0, 4.0, 1.0],
        [2.0, 4.0, 1.0],
        [3.0, 4.0, 1.0],
        [4.0, 4.0, 1.0],
        [5.0, 4.0, 1.0],
        [0.0, 5.0, 1.0],
        [1.0, 5.0, 1.0],
        [2.0, 5.0, 1.0],
        [3.0, 5.0, 1.0],
        [1.0, 0.0, 2.0],
        [2.0, 0.0, 2.0],
        [3.0, 0.0, 2.0],
        [4.0, 0.0, 2.0],
        [5.0, 0.0, 2.0],
        [0.0, 1.0, 2.0],
        [1.0, 1.0, 2.0],
        [2.0, 1.0, 2.0],
        [3.0, 1.0, 2.0],
        [4.0, 1.0, 2.0],
        [5.0, 1.0, 2.0],
        [0.0, 2.0, 2.0],
        [1.0, 2.0, 2.0],
        [2.0, 2.0, 2.0],
        [3.0, 2.0, 2.0],
        [4.0, 2.0, 2.0],
        [5.0, 2.0, 2.0],
        [0.0, 3.0, 2.0],
        [1.0, 3.0, 2.0],
        [2.0, 3.0, 2.0],
        [3.0, 3.0, 2.0],
        [4.0, 3.0, 2.0],
        [5.0, 3.0, 2.0],
        [0.0, 4.0, 2.0],
        [1.0, 4.0, 2.0],
        [2.0, 4.0, 2.0],
        [3.0, 4.0, 2.0],
        [4.0, 4.0, 2.0],
        [5.0, 4.0, 2.0],
        [1.0, 5.0, 2.0],
        [2.0, 5.0, 2.0],
        [3.0, 5.0, 2.0],
        [4.0, 5.0, 2.0],
        [5.0, 5.0, 2.0],
        [1.0, 0.0, 3.0],
        [2.0, 0.0, 3.0],
        [3.0, 0.0, 3.0],
        [4.0, 0.0, 3.0],
        [5.0, 0.0, 3.0],
        [1.0, 1.0, 3.0],
        [2.0, 1.0, 3.0],
        [3.0, 1.0, 3.0],
        [4.0, 1.0, 3.0],
        [5.0, 1.0, 3.0],
        [0.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [2.0, 2.0, 3.0],
        [3.0, 2.0, 3.0],
        [4.0, 2.0, 3.0],
        [0.0, 3.0, 3.0],
        [1.0, 3.0, 3.0],
        [2.0, 3.0, 3.0],
        [3.0, 3.0, 3.0],
        [4.0, 3.0, 3.0],
        [0.0, 4.0, 3.0],
        [1.0, 4.0, 3.0],
        [2.0, 4.0, 3.0],
        [3.0, 4.0, 3.0],
        [4.0, 4.0, 3.0],
        [5.0, 4.0, 3.0],
        [1.0, 5.0, 3.0],
        [2.0, 5.0, 3.0],
        [3.0, 5.0, 3.0],
        [4.0, 5.0, 3.0],
        [5.0, 5.0, 3.0],
        [0.0, 0.0, 4.0],
        [1.0, 0.0, 4.0],
        [2.0, 0.0, 4.0],
        [3.0, 0.0, 4.0],
        [4.0, 0.0, 4.0],
        [5.0, 0.0, 4.0],
        [0.0, 1.0, 4.0],
        [1.0, 1.0, 4.0],
        [2.0, 1.0, 4.0],
        [3.0, 1.0, 4.0],
        [4.0, 1.0, 4.0],
        [5.0, 1.0, 4.0],
        [0.0, 2.0, 4.0],
        [1.0, 2.0, 4.0],
        [2.0, 2.0, 4.0],
        [3.0, 2.0, 4.0],
        [4.0, 2.0, 4.0],
        [5.0, 2.0, 4.0],
        [0.0, 3.0, 4.0],
        [1.0, 3.0, 4.0],
        [2.0, 3.0, 4.0],
        [3.0, 3.0, 4.0],
        [4.0, 3.0, 4.0],
        [0.0, 4.0, 4.0],
        [1.0, 4.0, 4.0],
        [2.0, 4.0, 4.0],
        [3.0, 4.0, 4.0],
        [4.0, 4.0, 4.0],
        [5.0, 4.0, 4.0],
        [0.0, 5.0, 4.0],
        [1.0, 5.0, 4.0],
        [2.0, 5.0, 4.0],
        [3.0, 5.0, 4.0],
        [4.0, 5.0, 4.0],
        [5.0, 5.0, 4.0],
        [0.0, 0.0, 5.0],
        [1.0, 0.0, 5.0],
        [2.0, 0.0, 5.0],
        [3.0, 0.0, 5.0],
        [4.0, 0.0, 5.0],
        [0.0, 1.0, 5.0],
        [1.0, 1.0, 5.0],
        [2.0, 1.0, 5.0],
        [3.0, 1.0, 5.0],
        [4.0, 1.0, 5.0],
        [5.0, 1.0, 5.0],
        [0.0, 2.0, 5.0],
        [1.0, 2.0, 5.0],
        [2.0, 2.0, 5.0],
        [3.0, 2.0, 5.0],
        [4.0, 2.0, 5.0],
        [5.0, 2.0, 5.0],
        [2.0, 3.0, 5.0],
        [3.0, 3.0, 5.0],
        [4.0, 3.0, 5.0],
        [0.0, 4.0, 5.0],
        [1.0, 4.0, 5.0],
        [2.0, 4.0, 5.0],
        [3.0, 4.0, 5.0],
        [4.0, 4.0, 5.0],
        [5.0, 4.0, 5.0],
        [0.0, 5.0, 5.0],
        [1.0, 5.0, 5.0],
        [2.0, 5.0, 5.0],
        [4.0, 5.0, 5.0],
        [5.0, 5.0, 5.0],
    ]);
    let node_element_connectivity_gold = vec![
        vec![1],
        vec![1],
        vec![1, 2],
        vec![1, 2],
        vec![3],
        vec![3, 4],
        vec![4],
        vec![5],
        vec![2, 5, 6],
        vec![2, 6],
        vec![3, 7],
        vec![3, 4, 7],
        vec![4],
        vec![5],
        vec![5, 6, 8],
        vec![6, 8],
        vec![7, 9],
        vec![7, 9, 10],
        vec![10],
        vec![11],
        vec![8, 11, 12],
        vec![8, 12],
        vec![9],
        vec![9, 10],
        vec![10],
        vec![11],
        vec![11, 12],
        vec![12],
        vec![1],
        vec![1],
        vec![13],
        vec![13, 14],
        vec![14],
        vec![15],
        vec![1, 2, 15, 16],
        vec![1, 2, 16],
        vec![3, 13],
        vec![3, 4, 13, 14],
        vec![4, 14],
        vec![5, 15, 17],
        vec![2, 5, 6, 15, 16, 17],
        vec![2, 6, 16],
        vec![3, 7, 18],
        vec![3, 4, 7, 18, 19],
        vec![4, 19],
        vec![5, 17, 20],
        vec![5, 6, 8, 17, 20],
        vec![6, 8],
        vec![7, 9, 18, 21],
        vec![7, 9, 10, 18, 19, 21],
        vec![10, 19],
        vec![11, 20],
        vec![8, 11, 12, 20],
        vec![8, 12, 22],
        vec![9, 21, 22],
        vec![9, 10, 21],
        vec![10],
        vec![11],
        vec![11, 12],
        vec![12, 22],
        vec![22],
        vec![23],
        vec![23, 24],
        vec![13, 24, 25],
        vec![13, 14, 25],
        vec![14],
        vec![15],
        vec![15, 16, 23],
        vec![16, 23, 24],
        vec![13, 24, 25, 26],
        vec![13, 14, 25, 26],
        vec![14],
        vec![15, 17],
        vec![15, 16, 17],
        vec![16, 27],
        vec![18, 26, 27, 28],
        vec![18, 19, 26, 28],
        vec![19],
        vec![17, 20],
        vec![17, 20],
        vec![27, 29],
        vec![18, 21, 27, 28, 29, 30],
        vec![18, 19, 21, 28, 30],
        vec![19],
        vec![20],
        vec![20, 31],
        vec![22, 29, 31],
        vec![21, 22, 29, 30, 32],
        vec![21, 30, 32, 33],
        vec![33],
        vec![31],
        vec![22, 31],
        vec![22, 32],
        vec![32, 33],
        vec![33],
        vec![23],
        vec![23, 24],
        vec![24, 25, 34],
        vec![25, 34, 35],
        vec![35],
        vec![23, 36],
        vec![23, 24, 36, 37],
        vec![24, 25, 26, 34, 37],
        vec![25, 26, 34, 35],
        vec![35],
        vec![38],
        vec![36, 38],
        vec![27, 36, 37, 39],
        vec![26, 27, 28, 37, 39],
        vec![26, 28],
        vec![38, 40],
        vec![38, 40, 41],
        vec![27, 29, 39, 41],
        vec![27, 28, 29, 30, 39, 42],
        vec![28, 30, 42],
        vec![40],
        vec![31, 40, 41],
        vec![29, 31, 41, 43],
        vec![29, 30, 32, 42, 43],
        vec![30, 32, 33, 42, 44],
        vec![33, 44],
        vec![31],
        vec![31, 43],
        vec![32, 43],
        vec![32, 33, 44],
        vec![33, 44],
        vec![45],
        vec![45],
        vec![46],
        vec![34, 46, 47],
        vec![34, 35, 47],
        vec![35],
        vec![45, 48],
        vec![36, 45, 48, 49],
        vec![36, 37, 46, 49, 50],
        vec![34, 37, 46, 47, 50, 51],
        vec![34, 35, 47, 51, 52],
        vec![35, 52],
        vec![38, 48],
        vec![36, 38, 48, 49],
        vec![36, 37, 39, 49, 50, 53],
        vec![37, 39, 50, 51, 53],
        vec![51, 52],
        vec![52],
        vec![38, 40],
        vec![38, 40, 41],
        vec![39, 41, 53, 54],
        vec![39, 42, 53, 54, 55],
        vec![42, 55],
        vec![40, 56],
        vec![40, 41, 56, 57],
        vec![41, 43, 54, 57],
        vec![42, 43, 54, 55],
        vec![42, 44, 55, 58],
        vec![44, 58],
        vec![56],
        vec![56, 57],
        vec![43, 57],
        vec![43],
        vec![44, 58],
        vec![44, 58],
        vec![45],
        vec![45],
        vec![46],
        vec![46, 47],
        vec![47],
        vec![45, 48],
        vec![45, 48, 49],
        vec![46, 49, 50],
        vec![46, 47, 50, 51],
        vec![47, 51, 52],
        vec![52],
        vec![48],
        vec![48, 49],
        vec![49, 50, 53],
        vec![50, 51, 53],
        vec![51, 52],
        vec![52],
        vec![53, 54],
        vec![53, 54, 55],
        vec![55],
        vec![56],
        vec![56, 57],
        vec![54, 57],
        vec![54, 55],
        vec![55, 58],
        vec![58],
        vec![56],
        vec![56, 57],
        vec![57],
        vec![58],
        vec![58],
    ];
    let node_node_connectivity_gold = vec![
        vec![2, 3, 29],
        vec![1, 4, 30],
        vec![1, 4, 9, 35],
        vec![2, 3, 10, 36],
        vec![6, 11, 37],
        vec![5, 7, 12, 38],
        vec![6, 13, 39],
        vec![9, 14, 40],
        vec![3, 8, 10, 15, 41],
        vec![4, 9, 16, 42],
        vec![5, 12, 17, 43],
        vec![6, 11, 13, 18, 44],
        vec![7, 12, 45],
        vec![8, 15, 46],
        vec![9, 14, 16, 21, 47],
        vec![10, 15, 22, 48],
        vec![11, 18, 23, 49],
        vec![12, 17, 19, 24, 50],
        vec![18, 25, 51],
        vec![21, 26, 52],
        vec![15, 20, 22, 27, 53],
        vec![16, 21, 28, 54],
        vec![17, 24, 55],
        vec![18, 23, 25, 56],
        vec![19, 24, 57],
        vec![20, 27, 58],
        vec![21, 26, 28, 59],
        vec![22, 27, 60],
        vec![1, 30, 35],
        vec![2, 29, 36],
        vec![32, 37, 64],
        vec![31, 33, 38, 65],
        vec![32, 39, 66],
        vec![35, 40, 67],
        vec![3, 29, 34, 36, 41, 68],
        vec![4, 30, 35, 42, 69],
        vec![5, 31, 38, 43, 70],
        vec![6, 32, 37, 39, 44, 71],
        vec![7, 33, 38, 45, 72],
        vec![8, 34, 41, 46, 73],
        vec![9, 35, 40, 42, 47, 74],
        vec![10, 36, 41, 48, 75],
        vec![11, 37, 44, 49, 76],
        vec![12, 38, 43, 45, 50, 77],
        vec![13, 39, 44, 51, 78],
        vec![14, 40, 47, 52, 79],
        vec![15, 41, 46, 48, 53, 80],
        vec![16, 42, 47, 54],
        vec![17, 43, 50, 55, 82],
        vec![18, 44, 49, 51, 56, 83],
        vec![19, 45, 50, 57, 84],
        vec![20, 46, 53, 58, 85],
        vec![21, 47, 52, 54, 59, 86],
        vec![22, 48, 53, 55, 60, 87],
        vec![23, 49, 54, 56, 61, 88],
        vec![24, 50, 55, 57, 89],
        vec![25, 51, 56],
        vec![26, 52, 59],
        vec![27, 53, 58, 60],
        vec![28, 54, 59, 61, 92],
        vec![55, 60, 93],
        vec![63, 68, 96],
        vec![62, 64, 69, 97],
        vec![31, 63, 65, 70, 98],
        vec![32, 64, 66, 71, 99],
        vec![33, 65, 72],
        vec![34, 68, 73],
        vec![35, 62, 67, 69, 74, 101],
        vec![36, 63, 68, 70, 75, 102],
        vec![37, 64, 69, 71, 76, 103],
        vec![38, 65, 70, 72, 77, 104],
        vec![39, 66, 71],
        vec![40, 67, 74, 79],
        vec![41, 68, 73, 75, 80],
        vec![42, 69, 74, 76, 81, 108],
        vec![43, 70, 75, 77, 82, 109],
        vec![44, 71, 76, 78, 83, 110],
        vec![45, 77, 84],
        vec![46, 73, 80, 85],
        vec![47, 74, 79, 86],
        vec![75, 82, 87, 113],
        vec![49, 76, 81, 83, 88, 114],
        vec![50, 77, 82, 84, 89, 115],
        vec![51, 78, 83],
        vec![52, 79, 86],
        vec![53, 80, 85, 87, 91, 117],
        vec![54, 81, 86, 88, 92, 118],
        vec![55, 82, 87, 89, 93, 119],
        vec![56, 83, 88, 90, 94, 120],
        vec![89, 95, 121],
        vec![86, 92, 122],
        vec![60, 87, 91, 93, 123],
        vec![61, 88, 92, 94, 124],
        vec![89, 93, 95, 125],
        vec![90, 94, 126],
        vec![62, 97, 101],
        vec![63, 96, 98, 102],
        vec![64, 97, 99, 103, 130],
        vec![65, 98, 100, 104, 131],
        vec![99, 105, 132],
        vec![68, 96, 102, 107, 134],
        vec![69, 97, 101, 103, 108, 135],
        vec![70, 98, 102, 104, 109, 136],
        vec![71, 99, 103, 105, 110, 137],
        vec![100, 104, 138],
        vec![107, 111, 139],
        vec![101, 106, 108, 112, 140],
        vec![75, 102, 107, 109, 113, 141],
        vec![76, 103, 108, 110, 114, 142],
        vec![77, 104, 109, 115],
        vec![106, 112, 116, 145],
        vec![107, 111, 113, 117, 146],
        vec![81, 108, 112, 114, 118, 147],
        vec![82, 109, 113, 115, 119, 148],
        vec![83, 110, 114, 120, 149],
        vec![111, 117, 150],
        vec![86, 112, 116, 118, 122, 151],
        vec![87, 113, 117, 119, 123, 152],
        vec![88, 114, 118, 120, 124, 153],
        vec![89, 115, 119, 121, 125, 154],
        vec![90, 120, 126, 155],
        vec![91, 117, 123],
        vec![92, 118, 122, 124, 158],
        vec![93, 119, 123, 125, 159],
        vec![94, 120, 124, 126, 160],
        vec![95, 121, 125, 161],
        vec![128, 133, 162],
        vec![127, 134, 163],
        vec![130, 135, 164],
        vec![98, 129, 131, 136, 165],
        vec![99, 130, 132, 137, 166],
        vec![100, 131, 138],
        vec![127, 134, 139, 167],
        vec![101, 128, 133, 135, 140, 168],
        vec![102, 129, 134, 136, 141, 169],
        vec![103, 130, 135, 137, 142, 170],
        vec![104, 131, 136, 138, 143, 171],
        vec![105, 132, 137, 144, 172],
        vec![106, 133, 140, 145, 173],
        vec![107, 134, 139, 141, 146, 174],
        vec![108, 135, 140, 142, 147, 175],
        vec![109, 136, 141, 143, 148, 176],
        vec![137, 142, 144, 177],
        vec![138, 143, 178],
        vec![111, 139, 146, 150],
        vec![112, 140, 145, 147, 151],
        vec![113, 141, 146, 148, 152, 179],
        vec![114, 142, 147, 149, 153, 180],
        vec![115, 148, 154, 181],
        vec![116, 145, 151, 156, 182],
        vec![117, 146, 150, 152, 157, 183],
        vec![118, 147, 151, 153, 158, 184],
        vec![119, 148, 152, 154, 159, 185],
        vec![120, 149, 153, 155, 160, 186],
        vec![121, 154, 161, 187],
        vec![150, 157, 188],
        vec![151, 156, 158, 189],
        vec![123, 152, 157, 159, 190],
        vec![124, 153, 158],
        vec![125, 154, 161, 191],
        vec![126, 155, 160, 192],
        vec![127, 163, 167],
        vec![128, 162, 168],
        vec![129, 165, 169],
        vec![130, 164, 166, 170],
        vec![131, 165, 171],
        vec![133, 162, 168, 173],
        vec![134, 163, 167, 169, 174],
        vec![135, 164, 168, 170, 175],
        vec![136, 165, 169, 171, 176],
        vec![137, 166, 170, 172, 177],
        vec![138, 171, 178],
        vec![139, 167, 174],
        vec![140, 168, 173, 175],
        vec![141, 169, 174, 176, 179],
        vec![142, 170, 175, 177, 180],
        vec![143, 171, 176, 178],
        vec![144, 172, 177],
        vec![147, 175, 180, 184],
        vec![148, 176, 179, 181, 185],
        vec![149, 180, 186],
        vec![150, 183, 188],
        vec![151, 182, 184, 189],
        vec![152, 179, 183, 185, 190],
        vec![153, 180, 184, 186],
        vec![154, 181, 185, 187, 191],
        vec![155, 186, 192],
        vec![156, 182, 189],
        vec![157, 183, 188, 190],
        vec![158, 184, 189],
        vec![160, 186, 192],
        vec![161, 187, 191],
    ];
    let exterior_nodes_gold = (1..=192).collect();
    let interface_nodes_gold = vec![
        3, 4, 6, 9, 10, 11, 12, 15, 16, 17, 18, 21, 22, 24, 27, 35, 36, 37, 38, 40, 41, 42, 43, 44,
        46, 47, 48, 49, 50, 53, 54, 55, 56, 59, 60, 63, 64, 65, 68, 69, 70, 71, 73, 74, 75, 77, 79,
        80, 81, 82, 83, 86, 87, 92, 97, 98, 99, 102, 103, 104, 108, 109, 113, 114, 115, 118, 120,
        121, 123, 125, 126, 130, 131, 135, 136, 137, 141, 142, 143, 147, 148, 150, 151, 152, 154,
        155, 157, 158, 160, 161, 170, 171, 176, 177, 179, 180, 183, 184, 189,
    ];
    let interior_nodes_gold = vec![];
    let nodal_influencers_gold = node_node_connectivity_gold.clone();
    test_finite_elements(
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
        node_element_connectivity_gold,
        node_node_connectivity_gold,
        exterior_nodes_gold,
        interface_nodes_gold,
        interior_nodes_gold,
        None,
        None,
        nodal_influencers_gold,
    );
}

#[test]
fn valence_3_and_4_noised() {
    // Reference: https://autotwin.github.io/automesh/cli/metrics.html#hexahedral-unit-tests
    // We test both of the noised elements, valence_03' (noised)
    // valence_04' (noised)

    // Gold values
    let maximum_edge_ratios_gold = [1.2922598186116965, 1.167883631481492];
    let mininum_scaled_jacobians_gold = [0.19173666980464177, 0.3743932367172326];
    let maximum_skews_gold = [0.6797482929789989, 0.4864935739781938];
    let element_volumes_gold = [1.24779970625, 0.9844007500000004];

    let element_node_connectivity = vec![[1, 2, 4, 3, 5, 6, 8, 7]];

    let nodal_coordinates = [
        Coordinates::new(&[
            [0.110000e0, 0.120000e0, -0.130000e0],
            [1.200000e0, -0.200000e0, 0.000000e0],
            [-0.500000e0, 1.866025e0, -0.200000e0],
            [0.500000e0, 0.866025e0, -0.400000e0],
            [0.000000e0, 0.000000e0, 1.000000e0],
            [1.000000e0, 0.000000e0, 1.000000e0],
            [-0.500000e0, 0.600000e0, 1.400000e0],
            [0.500000e0, 0.866025e0, 1.200000e0],
        ]),
        Coordinates::new(&[
            [0.100000e0, 0.200000e0, 0.300000e0],
            [1.200000e0, 0.300000e0, 0.400000e0],
            [-0.200000e0, 1.200000e0, -0.100000e0],
            [1.030000e0, 1.102000e0, -0.250000e0],
            [-0.001000e0, -0.021000e0, 1.002000e0],
            [1.200000e0, -0.100000e0, 1.100000e0],
            [0.000000e0, 1.000000e0, 1.000000e0],
            [1.010000e0, 1.020000e0, 1.030000e0],
        ]),
    ];

    let maximum_edge_ratios: Vec<f64> = nodal_coordinates
        .iter()
        .flat_map(|x| calculate_maximum_edge_ratios(&element_node_connectivity, x).to_vec())
        .collect();

    let minimum_scaled_jacobians: Vec<f64> = nodal_coordinates
        .iter()
        .flat_map(|x| calculate_minimum_scaled_jacobians(&element_node_connectivity, x).to_vec())
        .collect();

    let maximum_skews: Vec<f64> = nodal_coordinates
        .iter()
        .flat_map(|x| calculate_maximum_skews(&element_node_connectivity, x).to_vec())
        .collect();

    // measures in 3D are volumes
    let element_volumes: Vec<f64> = nodal_coordinates
        .iter()
        .flat_map(|x| calculate_element_volumes_hex(&element_node_connectivity, x).to_vec())
        .collect();

    // Assert that the calculated values are approximately equal to the gold values
    assert_eq!(maximum_edge_ratios.len(), maximum_edge_ratios_gold.len(),);
    assert_eq!(
        minimum_scaled_jacobians.len(),
        mininum_scaled_jacobians_gold.len(),
    );
    assert_eq!(maximum_skews.len(), maximum_skews_gold.len(),);
    assert_eq!(element_volumes.len(), element_volumes_gold.len(),);

    // for in alternative
    // for (calculated, gold) in maximum_edge_ratios
    //     .iter()
    //     .zip(maximum_edge_ratios_gold.iter())
    // {
    //     assert!(
    //         (calculated - gold).abs() < EPSILON,
    //         "Calculated maximum edge ratio {} is not approximately equal to gold value {}",
    //         calculated,
    //         gold
    //     );
    // }

    // foreach alternative
    maximum_edge_ratios
        .iter()
        .zip(maximum_edge_ratios_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    minimum_scaled_jacobians
        .iter()
        .zip(mininum_scaled_jacobians_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    maximum_skews
        .iter()
        .zip(maximum_skews_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    element_volumes
        .iter()
        .zip(element_volumes_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });
}

#[test]
fn triangular_unit_tests() {
    // Reference: https://autotwin.github.io/automesh/cli/metrics.html#triangular-unit-tests
    // The first twelve triangles come from
    // tests/input/single_valence_04_noise2.inp.
    // We use
    // tests/tesselation.rs::from_stl::file_single_valence_04_noise2()
    // test to generate the coordinates and connectivity
    // from the stl file.
    // The next triangle comes from
    // tests/input/one_facet.stl.
    // The next triangle is an equilateral triangle of side length 4.0.

    // Gold values are not from Cubit, which uses "Aspect Ratio" instead of Edge Ratio
    // Turns out these are NOT the same thing!

    // Gold values from ~/autotwin/automesh/sandbox/metrics.py
    let maximum_edge_ratios_gold = [
        1.5078464057882237,
        1.5501674700560748,
        1.7870232669806838,
        1.915231466534568,
        2.230231996264181,
        1.6226774766497245,
        1.240081839656528,
        1.3849480786032335,
        1.6058086747499203,
        1.4288836646598568,
        1.2752274437112696,
        1.4361231071914424,
        std::f64::consts::SQRT_2, // 1.4142135623730951,
        1.0,
        1.0,
        1.2559260603991087,
    ];

    // Gold values from ~/autotwin/automesh/sandbox/metrics.py
    let minimum_angles_gold_deg = [
        41.20248899996187,
        39.796107567803936,
        33.61245209189106,
        31.00176761760843,
        21.661723789672273,
        37.33286786833477,
        51.03508450304211,
        46.05826353883047,
        38.512721702731355,
        44.27219859255808,
        49.65307785734987,
        44.12050798480872,
        45.00000000000001,
        59.99999999999999,
        59.99999999999999,
        48.794845448004004,
    ];
    // Gold values from ~/autotwin/automesh/sandbox/metrics.py
    let maximum_skews_gold = [
        0.3132918500006357,
        0.33673154053660104,
        0.4397924651351493,
        0.4833038730398595,
        0.6389712701721287,
        0.3777855355277538,
        0.14941525828263144,
        0.23236227435282555,
        0.35812130495447764,
        0.2621300234573654,
        0.17244870237750215,
        0.26465820025318804,
        0.2499999999999999,
        1.1842378929335003e-16,
        1.1842378929335003e-16,
        0.18675257586659993,
    ];
    // Gold values from ~/autotwin/automesh/sandbox/metrics.py and verified with Cubit
    let element_areas_gold = [
        0.6095033546646715,
        0.5498378247859254,
        0.5694533921062239,
        0.40221065958198676,
        0.34186812150301454,
        0.5705779745135626,
        0.42437710997648254,
        0.44293952755957805,
        0.6481635557480845,
        0.7040835887875813,
        0.6678959888148756,
        0.5158240173499096,
        0.5,
        6.928203230275509,
        0.43301270189221946,
        3.27324023180972,
    ];

    let minimum_scaled_jacobians_gold = [
        0.7606268158630964,
        0.7390747445600853,
        0.6392105272305011,
        0.5947452772930936,
        0.4262299581513255,
        0.700261936023385,
        0.8978156650410265,
        0.8314372958409268,
        0.7190186170534589,
        0.8060594150976131,
        0.8800416071493331,
        0.8038676339586197,
        0.8164965809277261,
        1.0,
        1.0,
        0.8687454713083852,
    ];

    let element_node_connectivity = vec![
        [1, 2, 3], // single_valence_04_noise2.inp begin
        [4, 2, 5],
        [1, 6, 2],
        [4, 3, 2],
        [4, 1, 3],
        [4, 7, 1],
        [2, 8, 5],
        [6, 8, 2],
        [7, 8, 6],
        [1, 7, 6],
        [4, 5, 8],
        [7, 4, 8],    // single_valence_04_noise2.inp end
        [9, 10, 11],  // one_facet.stl
        [12, 13, 14], // equilateral triangle of side length 4.0
        [15, 16, 17], // equilateral triangle of side length 1.0
        [18, 19, 20], // tilt.stl
    ];

    let nodal_coordinates = Coordinates::new(&[
        [-0.2, 1.2, -0.1], // single_valence_04_noise2.inp begin
        [1.180501, 0.39199, 0.3254445],
        [0.1, 0.2, 0.3],
        [-0.001, -0.021, 1.002],
        [1.2, -0.1, 1.1],
        [1.03, 1.102, -0.25],
        [0.0, 1.0, 1.0],
        [1.01, 1.02, 1.03],               // single_valence_04_noise2.inp end
        [0.0, 0.0, 1.0],                  // one_facet.stl begin
        [0.0, 0.0, 0.0],                  // ...
        [1.0, 0.0, 0.0],                  // one_facet.stl end
        [-2.0, 0.0, 0.0],                 // equilateral with edge length 4.0 start
        [2.0, 0.0, 0.0],                  // ...
        [0.0, 2.0 * 3.0_f64.sqrt(), 0.0], // equilateral with edge length 4.0 end
        [-0.5, 0.0, 0.0],                 // equilateral with edge length 1.0 start
        [0.5, 0.0, 0.0],                  // ...
        [0.0, 3.0_f64.sqrt() / 2.0, 0.0], // equilateral with edge length 1.0 end
        [0.0, 1.0, 3.0],                  // tilt.stl begin
        [2.0, 0.0, 2.0],
        [1.0, 1.0 + 3.0_f64.sqrt(), 1.0], // tile.stl end
    ]);

    let maximum_edge_ratios =
        calculate_maximum_edge_ratios(&element_node_connectivity, &nodal_coordinates);

    maximum_edge_ratios
        .iter()
        .zip(maximum_edge_ratios_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    let minimum_angles =
        calculate_minimum_angles_tri(&element_node_connectivity, &nodal_coordinates);

    let minimum_angles_deg: Vec<f64> = minimum_angles
        .iter()
        .map(|angle| angle * RAD_TO_DEG)
        .collect();

    minimum_angles_deg
        .iter()
        .zip(minimum_angles_gold_deg.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    let maximum_skews = calculate_maximum_skews(&element_node_connectivity, &nodal_coordinates);

    maximum_skews
        .iter()
        .zip(maximum_skews_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    let element_areas = calculate_element_areas_tri(&element_node_connectivity, &nodal_coordinates);

    element_areas
        .iter()
        .zip(element_areas_gold.iter())
        .for_each(|(calculated, gold)| assert!((calculated - gold).abs() < EPSILON,));

    // let element_areas = calculate_element_measures(&element_node_connectivity, &nodal_coordinates);
    let minimum_scaled_jacobians =
        calculate_minimum_scaled_jacobians(&element_node_connectivity, &nodal_coordinates);

    minimum_scaled_jacobians
        .iter()
        .zip(minimum_scaled_jacobians_gold.iter())
        .for_each(|(calculated, gold)| assert!((calculated - gold).abs() < EPSILON,));
}

#[test]
fn metrics_headers_test() {
    // The headers for metrics files are unique depending on if the
    // element type is hexahedral versus triangular.
    // This test assures both types are created correctly.

    // Test HEX headers
    let hex_header_gold =
        "maximum edge ratio,minimum scaled jacobian,maximum skew,element volume\n".to_string();
    let hex_header_result = metrics_headers::<HEX>();
    assert_eq!(hex_header_gold, hex_header_result);

    // Test TRI headers
    let tri_header_gold =
        "maximum edge ratio,minimum scaled jacobian,maximum skew,element area,minimum angle\n"
            .to_string();
    let tri_header_result = metrics_headers::<TRI>();
    assert_eq!(tri_header_gold, tri_header_result);

    // Test the headers used in several files, such as .csv and .exo output
    let automesh_header_gold =
        format!("autotwin.automesh, version {}", env!("CARGO_PKG_VERSION")).to_string();
    let automesh_header = automesh_header();

    let index = automesh_header.find(", autogenerated on").unwrap();
    let substring = &automesh_header[..index];
    assert_eq!(automesh_header_gold, substring)
}

// #[test]
// fn metrics_format_test() {
//     // The metrics have a specific spacing in the output file,
//     // depending on if the element type is hexahedral or triangular.
//     // This test assures both types are formatted correctly.
//
//     // Test HEX format
//     let hex_format_gold = "{:>20.6e}, {:>26.6e}, {:>26.6e}, {:>26.6e}\n".to_string();
//     let hex_format_result = metrics_format::<HEX>();
//     assert_eq!(hex_format_gold, hex_format_result);
//
//     // Test TRI format
//     let tri_format_gold =
//         "{:>20.6e}, {:>26.6e}, {:>26.6e}\n".to_string();
//     let tri_format_result = metrics_format::<TRI>();
//     assert_eq!(tri_format_gold, tri_format_result);
// }
