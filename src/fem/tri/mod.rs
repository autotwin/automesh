#[cfg(test)]
pub mod test;

mod remesh;

#[cfg(feature = "profile")]
use std::time::Instant;

use self::remesh::{edge_info, remesh, split_edges};
use super::{
    super::{Vectors, tree::Edges},
    Connectivity, Coordinate, Coordinates, FiniteElementMethods, FiniteElementSpecifics,
    FiniteElements, Metrics, Size, Smoothing, Tessellation, Vector,
};
use conspire::{
    geometry::mesh::{Connectivity as MeshConnectivity, Mesh, Verdict},
    math::{CrossProduct, Tensor, TensorArray, Vector as VectorConspire, assert_eq_within_tols},
    mechanics::Scalar,
};
use ndarray::{Array2, s};
use ndarray_npy::WriteNpyExt;
use std::{
    f64::consts::PI,
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    path::Path,
};

const FOUR_THIRDS: Scalar = 4.0 / 3.0;

/// The number of nodes in a triangular finite element.
pub const TRI: usize = 3;

const O: usize = 2;
const NUM_NODES_FACE: usize = 1;

type Curvatures = VectorConspire;
type Lengths = conspire::math::Vector;

/// The triangular finite elements type.
pub type TriangularFiniteElements = FiniteElements<TRI>;

impl From<Tessellation> for TriangularFiniteElements {
    fn from(tessellation: Tessellation) -> Self {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let (connectivity, coordinates, _) = tessellation.into();
        let element_node_connectivity: Connectivity<TRI> = connectivity
            .into_members()
            .iter()
            .flat_map(|block| block.into_iter().map(|nodes| nodes.try_into().unwrap()))
            .collect();
        let element_blocks = vec![1; element_node_connectivity.len()];
        let triangular_finite_elements = TriangularFiniteElements::from((
            element_blocks,
            element_node_connectivity,
            coordinates.into(),
        ));
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mSerializing triangles\x1b[0m {:?}",
            time.elapsed()
        );
        triangular_finite_elements
    }
}

impl FiniteElementSpecifics<NUM_NODES_FACE, O> for TriangularFiniteElements {
    fn connected_nodes(node: &usize) -> [usize; O] {
        match node {
            0 => [1, 2],
            1 => [0, 2],
            2 => [0, 1],
            _ => panic!(),
        }
    }
    fn connected_nodes_face(_node: &usize) -> [usize; 2] {
        unimplemented!()
    }
    fn exterior_faces(&self) -> Connectivity<NUM_NODES_FACE> {
        unimplemented!()
    }
    fn exterior_faces_interior_points(&self, _grid_length: usize) -> Coordinates {
        todo!()
    }
    fn faces(&self) -> Connectivity<NUM_NODES_FACE> {
        unimplemented!()
    }
    fn interior_points(&self, _grid_length: usize) -> Coordinates {
        todo!()
    }
    fn maximum_edge_ratios(&self) -> Metrics {
        // Knupp 2006
        // https://www.osti.gov/servlets/purl/901967
        // page 19 and 26
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = 0.0;
        let mut l1 = 0.0;
        let mut l2 = 0.0;
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = (&nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]])
                    .norm();
                l1 = (&nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]])
                    .norm();
                l2 = (&nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]])
                    .norm();
                [l0, l1, l2].into_iter().reduce(f64::max).unwrap()
                    / [l0, l1, l2].into_iter().reduce(f64::min).unwrap()
            })
            .collect()
    }
    fn maximum_skews(&self) -> Metrics {
        let deg_to_rad = PI / 180.0;
        let equilateral_rad = 60.0 * deg_to_rad;
        let minimum_angles = self.minimum_angles();
        minimum_angles
            .iter()
            .map(|angle| (equilateral_rad - angle) / (equilateral_rad))
            .collect()
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        let connectivity = self
            .get_element_node_connectivity()
            .clone()
            .into_iter()
            .collect::<Vec<[usize; TRI]>>();
        let coordinates = self
            .get_nodal_coordinates()
            .iter()
            .map(|coordinate| [coordinate[0], coordinate[1], coordinate[2]])
            .collect::<Vec<[f64; 3]>>();
        let mesh = Mesh::<3>::from((
            vec![MeshConnectivity::Triangular(connectivity.into())],
            coordinates.into(),
        ));
        mesh.minimum_scaled_jacobians()
            .into_iter()
            .flatten()
            .collect()
    }
    fn remesh(&mut self, iterations: usize, smoothing_method: &Smoothing, size: Size) {
        remesh(self, iterations, smoothing_method, size)
    }
    fn write_metrics(&self, file_path: &str) -> Result<(), ErrorIO> {
        let areas = self.areas();
        let maximum_skews = self.maximum_skews();
        let maximum_edge_ratios = self.maximum_edge_ratios();
        let minimum_angles = self.minimum_angles();
        let minimum_scaled_jacobians = self.minimum_scaled_jacobians();
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut file = BufWriter::new(File::create(file_path)?);
        let input_extension = Path::new(&file_path)
            .extension()
            .and_then(|ext| ext.to_str());
        match input_extension {
            Some("csv") => {
                let header_string =
                    "maximum edge ratio,minimum scaled jacobian,maximum skew,element volume\n"
                        .to_string();
                file.write_all(header_string.as_bytes())?;
                maximum_edge_ratios
                    .iter()
                    .zip(
                        minimum_scaled_jacobians.iter().zip(
                            maximum_skews
                                .iter()
                                .zip(areas.iter().zip(minimum_angles.iter())),
                        ),
                    )
                    .try_for_each(
                        |(
                            maximum_edge_ratio,
                            (minimum_scaled_jacobian, (maximum_skew, (area, minimum_angle))),
                        )| {
                            file.write_all(
                                format!(
                                    "{maximum_edge_ratio:>10.6e},{minimum_scaled_jacobian:>10.6e},{maximum_skew:>10.6e},{area:>10.6e},{minimum_angle:>10.6e}\n",
                                )
                                .as_bytes(),
                            )
                        },
                    )?;
                file.flush()?
            }
            Some("npy") => {
                let n_columns = 5; // total number of triangle metrics
                let idx_ratios = 0; // maximum edge ratios
                let idx_jacobians = 1; // minimum scaled jacobians
                let idx_skews = 2; // maximum skews
                let idx_areas = 3; // areas
                let idx_angles = 4; // minimum angles
                let mut metrics_set =
                    Array2::<f64>::from_elem((minimum_scaled_jacobians.len(), n_columns), 0.0);
                metrics_set
                    .slice_mut(s![.., idx_ratios])
                    .assign(&maximum_edge_ratios);
                metrics_set
                    .slice_mut(s![.., idx_jacobians])
                    .assign(&minimum_scaled_jacobians);
                metrics_set
                    .slice_mut(s![.., idx_skews])
                    .assign(&maximum_skews);
                metrics_set.slice_mut(s![.., idx_areas]).assign(&areas);
                metrics_set
                    .slice_mut(s![.., idx_angles])
                    .assign(&minimum_angles);
                metrics_set.write_npy(file).unwrap();
            }
            _ => panic!(
                "Unsupported file extension for metrics output: {:?}.  Please use 'csv' or 'npy'.",
                input_extension
            ),
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mWriting triangular metrics to file\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(())
    }
}

impl TriangularFiniteElements {
    fn area(coordinates: &Coordinates, &[node_0, node_1, node_2]: &[usize; TRI]) -> f64 {
        0.5 * ((&coordinates[node_2] - &coordinates[node_1])
            .cross(&coordinates[node_0] - &coordinates[node_2]))
        .norm()
    }
    fn areas(&self) -> Metrics {
        let coordinates = self.get_nodal_coordinates();
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| Self::area(coordinates, connectivity))
            .collect()
    }
    /// Determines whether the triangle contains the point or not.
    pub fn contains(
        point: &Coordinate,
        coordinates: &Coordinates,
        [node_0, node_1, node_2]: [usize; TRI],
    ) -> bool {
        let area = Self::area(coordinates, &[node_0, node_1, node_2]);
        let v_0 = &coordinates[node_0] - point;
        let v_1 = &coordinates[node_1] - point;
        let v_2 = &coordinates[node_2] - point;
        let area_01 = v_1.cross(&v_0).norm();
        let area_12 = v_2.cross(v_1).norm();
        let area_20 = v_0.cross(v_2).norm();
        assert_eq_within_tols(&(2.0 * area), &(area_01 + area_12 + area_20)).is_ok()
    }
    /// Determines whether the triangle is intersected by the vector and returns the point of intersection.
    pub fn intersection(
        direction: &Coordinate,
        origin: &Coordinate,
        coordinates: &Coordinates,
        connectivity: [usize; TRI],
    ) -> Option<Coordinate> {
        let normal = Self::normal(coordinates, connectivity);
        let product = direction * &normal;
        if product.abs() < f64::EPSILON {
            None
        } else {
            let distance = (normal * (&coordinates[connectivity[0]] - origin)) / product;
            let point = origin + direction * distance;
            if Self::contains(&point, coordinates, connectivity) {
                Some(point)
            } else {
                None
            }
        }
    }
    /// Computes and returns the closest point in the triangle to another point.
    /// This implementation uses a non-iterative barycentric coordinate appraoch.
    pub fn closest_point(
        point: &Coordinate,
        coordinates: &Coordinates,
        [node_0, node_1, node_2]: [usize; TRI],
    ) -> Coordinate {
        // Set up vertex coordinates
        let coordinates_0 = &coordinates[node_0];
        let coordinates_1 = &coordinates[node_1];
        let coordinates_2 = &coordinates[node_2];
        // Set up edge vectors
        let v_01 = coordinates_1 - coordinates_0; // edge from v0 to v1
        let v_02 = coordinates_2 - coordinates_0; // edge from v0 to v2
        // Check if point is in the vertex region outside v0
        let v_0p = point - coordinates_0; // vector from v0 to target point P
        let d1 = &v_01 * &v_0p; // project P onto v_01
        let d2 = &v_02 * v_0p; // project P onto v_02
        if d1 <= 0.0 && d2 <= 0.0 {
            return coordinates_0.clone(); // v0 is the closest point
        }
        // Check if point is in the vertex region outside v1
        let v_1p = point - coordinates_1; // vector from v1 to target point P
        let d3 = &v_01 * &v_1p;
        let d4 = &v_02 * v_1p;
        if d3 >= 0.0 && d4 <= d3 {
            return coordinates_1.clone(); // v1 is the closest point
        }
        // Check if point is in the vertex region outside v2
        let v_2p = point - coordinates_2;
        let d5 = &v_01 * &v_2p;
        let d6 = &v_02 * v_2p;
        if d6 >= 0.0 && d5 <= d6 {
            return coordinates_2.clone(); // v2 is the closest point
        }
        // Check if point is in edge region of v_01
        let vc = d1 * d4 - d3 * d2; // area-like calculation (barycentric weight)
        if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            let v = d1 / (d1 - d3); // barycentric parameter
            return coordinates_0 + v_01 * v; // projection onto edge v_01
        }
        // Check if point is in edge region of v_02
        let vb = d5 * d2 - d1 * d6;
        if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            let w = d2 / (d2 - d6);
            return coordinates_0 + v_02 * w; // projection onto edge v_02
        }
        // Check if point is in edge region of v_12
        let va = d3 * d6 - d5 * d4;
        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
            // projection onto edge v_12
            return coordinates_1
                + (coordinates_2 - coordinates_1) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
        }
        // Point is inside the face region
        // Compute the closest point using all three barycentric coordinates (u, v, w)
        let denom = va + vb + vc;
        // Result = v0 + v(v1 - v0) + w(v2 - v0)
        coordinates_0 + v_01 * (vb / denom) + v_02 * (vc / denom)
    }
    /// Calculates and returns the Gaussian curvature.
    pub fn curvature(&self) -> Result<Curvatures, String> {
        let mut edge = Vector::zero();
        let mut edge_norm = 0.0;
        let mut edges_weight = 0.0;
        let mut element_index_1 = 0;
        let mut element_index_2 = 0;
        let mut node_c = 0;
        let mut node_d = 0;
        let element_node_connectivity = self.get_element_node_connectivity();
        let node_element_connectivity = self.get_node_element_connectivity();
        let node_node_connectivity = self.get_node_node_connectivity();
        let nodal_coordinates = self.get_nodal_coordinates();
        if !node_node_connectivity.is_empty() {
            Ok(self
                .get_nodal_coordinates()
                .iter()
                .zip(node_node_connectivity.iter().enumerate())
                .map(|(coordinates_a, (node_a, nodes))| {
                    edges_weight = 0.0;
                    nodes
                        .iter()
                        .map(|&node_b| {
                            [element_index_1, element_index_2, node_c, node_d] = edge_info(
                                node_a,
                                node_b,
                                element_node_connectivity,
                                node_element_connectivity,
                            );
                            edge = coordinates_a - &nodal_coordinates[node_b];
                            edge_norm = edge.norm();
                            edges_weight += edge_norm;
                            ((&nodal_coordinates[node_c] - &nodal_coordinates[node_a])
                                .cross(&nodal_coordinates[node_b] - &nodal_coordinates[node_c])
                                .normalized()
                                * (&nodal_coordinates[node_d] - &nodal_coordinates[node_b])
                                    .cross(&nodal_coordinates[node_a] - &nodal_coordinates[node_d])
                                    .normalized())
                            .acos()
                                / PI
                                * edge_norm
                        })
                        .sum::<Scalar>()
                        / edges_weight
                })
                .collect())
        } else {
            Err("Need to calculate the node-to-node connectivity first".to_string())
        }
    }
    fn minimum_angles(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = Vector::zero();
        let mut l1 = Vector::zero();
        let mut l2 = Vector::zero();
        let flip = -1.0;
        self.get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = &nodal_coordinates[connectivity[2]] - &nodal_coordinates[connectivity[1]];
                l1 = &nodal_coordinates[connectivity[0]] - &nodal_coordinates[connectivity[2]];
                l2 = &nodal_coordinates[connectivity[1]] - &nodal_coordinates[connectivity[0]];
                l0.normalize();
                l1.normalize();
                l2.normalize();
                [
                    ((&l0 * flip) * &l1).acos(),
                    ((&l1 * flip) * &l2).acos(),
                    ((&l2 * flip) * &l0).acos(),
                ]
                .into_iter()
                .reduce(f64::min)
                .unwrap()
            })
            .collect()
    }
    /// Computes and returns the normal vector for a triangle.
    pub fn normal(coordinates: &Coordinates, [node_0, node_1, node_2]: [usize; TRI]) -> Vector {
        (&coordinates[node_1] - &coordinates[node_0])
            .cross(&coordinates[node_2] - &coordinates[node_0])
            .normalized()
    }
    /// Computes and returns the normal vectors for all triangles.
    pub fn normals(&self) -> Vectors {
        let coordinates = self.get_nodal_coordinates();
        let connectivity = self.get_element_node_connectivity();
        connectivity
            .iter()
            .map(|&connectivity| Self::normal(coordinates, connectivity))
            .collect()
    }
    /// Iteratively refine until all edges are smaller than a size.
    pub fn refine(&mut self, size: Scalar) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut edges: Edges = self
            .get_element_node_connectivity()
            .iter()
            .flat_map(|&[node_0, node_1, node_2]| {
                [[node_0, node_1], [node_1, node_2], [node_2, node_0]].into_iter()
            })
            .collect();
        edges.iter_mut().for_each(|edge| edge.sort());
        edges.sort();
        edges.dedup();
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut lengths = Lengths::zero(edges.len());
        edges
            .iter()
            .zip(lengths.iter_mut())
            .for_each(|(&[node_a, node_b], length)| {
                *length = (&nodal_coordinates[node_a] - &nodal_coordinates[node_b]).norm()
            });
        self.boundary_nodes = vec![];
        self.exterior_nodes = vec![];
        self.interface_nodes = vec![];
        self.interior_nodes = vec![];
        loop {
            if lengths.iter().any(|length| length > &size) {
                split_edges(self, &mut edges, &mut lengths, size / FOUR_THIRDS)
                //
                // Would be nice to bake this into remeshing with two options based on input args:
                // (1) just go for some number of iterations like typically do
                // (2) iterate until all edge below size
                //     - would still do edge collapse (get all edges close to size, just no larger than size)
                //
            } else {
                break;
            }
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mSplitting large edges\x1b[0m {:?}",
            time.elapsed()
        );
    }
}
