#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    FiniteElementMethods, FiniteElementSpecifics, FiniteElements, Metrics, Tessellation, Vector,
    NODE_NUMBERING_OFFSET,
};
use conspire::math::{Tensor, TensorArray};
use ndarray::{s, Array2};
use ndarray_npy::WriteNpyExt;
use std::{
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    path::Path,
};

const J_EQUILATERAL: f64 = 0.8660254037844387;

/// The number of nodes in a triangular finite element.
pub const TRI: usize = 3;

/// The triangular finite elements type.
pub type TriangularFiniteElements = FiniteElements<TRI>;

impl FiniteElementSpecifics for TriangularFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 2],
            1 => vec![0, 2],
            2 => vec![0, 1],
            _ => panic!(),
        }
    }
    fn into_tesselation(self) -> Tessellation {
        let mut normal = Vector::zero();
        let mut vertices_tri = [0; TRI];
        let nodal_coordinates = self.get_nodal_coordinates();
        let vertices = nodal_coordinates
            .iter()
            .map(|coordinate| {
                stl_io::Vertex::new([
                    coordinate[0] as f32,
                    coordinate[1] as f32,
                    coordinate[2] as f32,
                ])
            })
            .collect();
        let faces = self
            .get_element_node_connectivity()
            .iter()
            .map(|&connectivity| {
                vertices_tri = [
                    connectivity[0] - NODE_NUMBERING_OFFSET,
                    connectivity[1] - NODE_NUMBERING_OFFSET,
                    connectivity[2] - NODE_NUMBERING_OFFSET,
                ];
                normal = (&nodal_coordinates[vertices_tri[1]]
                    - &nodal_coordinates[vertices_tri[0]])
                    .cross(
                        &(&nodal_coordinates[vertices_tri[2]]
                            - &nodal_coordinates[vertices_tri[0]]),
                    )
                    .normalized();
                stl_io::IndexedTriangle {
                    normal: stl_io::Normal::new([
                        normal[0] as f32,
                        normal[1] as f32,
                        normal[2] as f32,
                    ]),
                    vertices: vertices_tri,
                }
            })
            .collect();
        Tessellation::new(stl_io::IndexedMesh { vertices, faces })
    }
    fn maximum_edge_ratios(&self) -> Metrics {
        // Knupp 2006
        // https://www.osti.gov/servlets/purl/901967
        // page 19 and 26
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = 0.0;
        let mut l1 = 0.0;
        let mut l2 = 0.0;
        let maximum_edge_ratios = self
            .get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = (&nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET])
                    .norm();
                l1 = (&nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET])
                    .norm();
                l2 = (&nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET])
                    .norm();
                [l0, l1, l2].into_iter().reduce(f64::max).unwrap()
                    / [l0, l1, l2].into_iter().reduce(f64::min).unwrap()
            })
            .collect();
        maximum_edge_ratios
    }
    fn maximum_skews(&self) -> Metrics {
        let deg_to_rad = std::f64::consts::PI / 180.0;
        let equilateral_rad = 60.0 * deg_to_rad;
        let minimum_angles = self.minimum_angles();
        let maximum_skews = minimum_angles
            .iter()
            .map(|angle| (equilateral_rad - angle) / (equilateral_rad))
            .collect();
        maximum_skews
    }
    fn minimum_scaled_jacobians(&self) -> Metrics {
        let minimum_scaled_jacobians = self
            .minimum_angles()
            .iter()
            .map(|angle| (angle.sin() / J_EQUILATERAL))
            .collect();
        minimum_scaled_jacobians
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
                                    "{:>10.6e},{:>10.6e},{:>10.6e},{:>10.6e},{:>10.6e}\n",
                                    maximum_edge_ratio,
                                    minimum_scaled_jacobian,
                                    maximum_skew,
                                    area,
                                    minimum_angle
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
            _ => panic!("print error message with input and extension"),
        }
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mWriting triangle metrics to file\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(())
    }
}

impl TriangularFiniteElements {
    fn areas(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = Vector::zero();
        let mut l1 = Vector::zero();
        let element_areas = self
            .get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET];
                l1 = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET];
                0.5 * (l0.cross(&l1)).norm()
            })
            .collect();
        element_areas
    }
    fn minimum_angles(&self) -> Metrics {
        let nodal_coordinates = self.get_nodal_coordinates();
        let mut l0 = Vector::zero();
        let mut l1 = Vector::zero();
        let mut l2 = Vector::zero();
        let flip = -1.0;
        let minimum_angles = self
            .get_element_node_connectivity()
            .iter()
            .map(|connectivity| {
                l0 = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET];
                l1 = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET];
                l2 = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                    - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET];
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
            .collect();
        minimum_angles
    }
}
