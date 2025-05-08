#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{
    calculate_maximum_edge_ratios, calculate_maximum_skews, calculate_minimum_scaled_jacobians,
    metrics_headers, Connectivity, Coordinates, FiniteElementSpecifics, FiniteElements, Metrics,
    Tessellation, Vector, NODE_NUMBERING_OFFSET,
};
use conspire::math::{Tensor, TensorArray};
use ndarray::{s, Array2};
use ndarray_npy::WriteNpyExt;
use std::{
    fs::File,
    io::{BufWriter, Error as ErrorIO, Write},
    path::Path,
};

/// The number of nodes in a hexahedral finite element.
pub const HEX: usize = 8;

/// The hexahedral finite elements type.
pub type HexahedralFiniteElements = FiniteElements<HEX>;

impl FiniteElementSpecifics for HexahedralFiniteElements {
    fn connected_nodes(node: &usize) -> Vec<usize> {
        match node {
            0 => vec![1, 3, 4],
            1 => vec![0, 2, 5],
            2 => vec![1, 3, 6],
            3 => vec![0, 2, 7],
            4 => vec![0, 5, 7],
            5 => vec![1, 4, 6],
            6 => vec![2, 5, 7],
            7 => vec![3, 4, 6],
            _ => panic!(),
        }
    }
    fn into_tesselation(self) -> Tessellation {
        unimplemented!()
    }
}

pub fn write_finite_elements_metrics_hex<const N: usize>(
    file_path: &str,
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Result<(), ErrorIO> {
    // #TODO: consider rearchitect, as these types of if-type-checks
    // indicate rearchitecture may help code logic.
    if N != HEX {
        panic!("Only implemented for hexahedral elements.")
    }
    let maximum_edge_ratios =
        calculate_maximum_edge_ratios(element_node_connectivity, nodal_coordinates);
    let minimum_scaled_jacobians =
        calculate_minimum_scaled_jacobians(element_node_connectivity, nodal_coordinates);
    let maximum_skews = calculate_maximum_skews(element_node_connectivity, nodal_coordinates);
    let volumes = calculate_element_volumes_hex(element_node_connectivity, nodal_coordinates);
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut file = BufWriter::new(File::create(file_path)?);
    let input_extension = Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str());
    match input_extension {
        Some("csv") => {
            let header_string = metrics_headers::<N>();
            file.write_all(header_string.as_bytes())?;
            maximum_edge_ratios
                .iter()
                .zip(
                    minimum_scaled_jacobians
                        .iter()
                        .zip(maximum_skews.iter().zip(volumes.iter())),
                )
                .try_for_each(
                    |(maximum_edge_ratio, (minimum_scaled_jacobian, (maximum_skew, volume)))| {
                        file.write_all(
                            format!(
                                "{:>10.6e},{:>10.6e},{:>10.6e},{:>10.6e}\n",
                                maximum_edge_ratio, minimum_scaled_jacobian, maximum_skew, volume,
                            )
                            .as_bytes(),
                        )
                    },
                )?;
            file.flush()?
        }
        Some("npy") => {
            let n_columns = 4; // total number of hexahedral metrics
            let idx_ratios = 0; // maximum edge ratios
            let idx_jacobians = 1; // minimum scaled jacobians
            let idx_skews = 2; // maximum skews
            let idx_volumes = 3; // areas
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
            metrics_set.slice_mut(s![.., idx_volumes]).assign(&volumes);
            metrics_set.write_npy(file).unwrap();
        }
        _ => panic!("print error message with input and extension"),
    }
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mWriting hexahedron metrics to file\x1b[0m {:?}",
        time.elapsed()
    );
    Ok(())
}

pub fn calculate_maximum_edge_ratios_hex<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    // #TODO: consider rearchitect, as these types of if-type-checks
    // indicate rearchitecture may help code logic.
    if N != HEX {
        panic!("Only implemented for hexahedral elements.")
    }
    let mut l1 = 0.0;
    let mut l2 = 0.0;
    let mut l3 = 0.0;
    let maximum_edge_ratios = element_node_connectivity
        .iter()
        .map(|connectivity| {
            l1 = (&nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET])
                .norm();
            l2 = (&nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET])
                .norm();
            l3 = (&nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                + &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                - &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET])
                .norm();
            [l1, l2, l3].into_iter().reduce(f64::max).unwrap()
                / [l1, l2, l3].into_iter().reduce(f64::min).unwrap()
        })
        .collect();
    maximum_edge_ratios
}

pub fn calculate_minimum_scaled_jacobians_hex<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    // #TODO: consider rearchitect, as these types of if-type-checks
    // indicate rearchitecture may help code logic.
    if N != HEX {
        panic!("Only implemented for hexahedral elements.")
    }
    let mut u = Vector::zero();
    let mut v = Vector::zero();
    let mut w = Vector::zero();
    let mut n = Vector::zero();
    let minimum_scaled_jacobians = element_node_connectivity
        .iter()
        .map(|connectivity| {
            connectivity
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    match index {
                        0 => {
                            u = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        1 => {
                            u = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        2 => {
                            u = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        3 => {
                            u = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        4 => {
                            u = &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        5 => {
                            u = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        6 => {
                            u = &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        7 => {
                            u = &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            v = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                            w = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
                                - &nodal_coordinates[node - NODE_NUMBERING_OFFSET];
                        }
                        _ => panic!(),
                    }
                    n = u.cross(&v);
                    (&n * &w) / u.norm() / v.norm() / w.norm()
                })
                .collect::<Vec<f64>>()
                .into_iter()
                .reduce(f64::min)
                .unwrap()
        })
        .collect();
    minimum_scaled_jacobians
}

pub fn calculate_element_principal_axes<const N: usize>(
    connectivity: &[usize; N],
    nodal_coordinates: &Coordinates,
) -> (Vector, Vector, Vector) {
    // #TODO: consider rearchitect, as these types of if-type-checks
    // indicate rearchitecture may help code logic.
    if N != HEX {
        panic!("Only implemented for hexahedral elements.")
    }
    let x1 = &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET];
    let x2 = &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET];
    let x3 = &nodal_coordinates[connectivity[4] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[0] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[5] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[1] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[6] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[2] - NODE_NUMBERING_OFFSET]
        + &nodal_coordinates[connectivity[7] - NODE_NUMBERING_OFFSET]
        - &nodal_coordinates[connectivity[3] - NODE_NUMBERING_OFFSET];
    (x1, x2, x3)
}

pub fn calculate_maximum_skews_hex<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    // #TODO: consider rearchitect, as these types of if-type-checks
    // indicate rearchitecture may help code logic.
    if N != HEX {
        panic!("Only implemented for hexahedral elements.")
    }
    let mut x1 = Vector::zero();
    let mut x2 = Vector::zero();
    let mut x3 = Vector::zero();
    let maximum_skews = element_node_connectivity
        .iter()
        .map(|connectivity| {
            (x1, x2, x3) = calculate_element_principal_axes(connectivity, nodal_coordinates);
            x1.normalize();
            x2.normalize();
            x3.normalize();
            [(&x1 * &x2).abs(), (&x1 * &x3).abs(), (&x2 * &x3).abs()]
                .into_iter()
                .reduce(f64::max)
                .unwrap()
        })
        .collect();
    maximum_skews
}

pub fn calculate_element_volumes_hex<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    // #TODO: consider rearchitect, as these types of if-type-checks
    // indicate rearchitecture may help code logic.
    if N != HEX {
        panic!("Only implemented for hexahedral elements.")
    }
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut x1 = Vector::zero();
    let mut x2 = Vector::zero();
    let mut x3 = Vector::zero();
    let element_volumes = element_node_connectivity
        .iter()
        .map(|connectivity| {
            (x1, x2, x3) = calculate_element_principal_axes(connectivity, nodal_coordinates);
            &x2.cross(&x3) * &x1 / 64.0
        })
        .collect();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mHexahedron element volumes\x1b[0m {:?}",
        time.elapsed()
    );
    element_volumes
}
