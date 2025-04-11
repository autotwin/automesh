#[cfg(feature = "python")]
pub mod py;

#[cfg(test)]
pub mod test;

pub mod tri;
use tri::{
    calculate_maximum_edge_ratios_tri, calculate_maximum_skews_tri,
    calculate_minimum_scaled_jacobians_tri, write_finite_elements_metrics_tri,
};
pub use tri::{TriangularFiniteElements, TRI};

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{Coordinate, Coordinates, Tessellation, Vector, NSD};
use chrono::Utc;
use conspire::math::{Tensor, TensorArray, TensorVec};
use ndarray::{s, Array1, Array2};
use ndarray_npy::WriteNpyExt;
use netcdf::{create, Error as ErrorNetCDF};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Error as ErrorIO, Write},
    path::{Path, PathBuf},
};
use vtkio::{
    model::{
        Attributes, ByteOrder, CellType, Cells, DataSet, IOBuffer, UnstructuredGridPiece, Version,
        VertexNumbers, Vtk,
    },
    Error as ErrorVtk,
};

const ELEMENT_NUMBERING_OFFSET: usize = 1;
pub const NODE_NUMBERING_OFFSET: usize = 1;

/// A vector of finite element block IDs.
pub type Blocks = Vec<u8>;

/// An element-to-node connectivity.
pub type Connectivity<const N: usize> = Vec<[usize; N]>;

pub type VecConnectivity = Vec<Vec<usize>>;
pub type Metrics = Array1<f64>;
pub type Nodes = Vec<usize>;
pub type ReorderedConnectivity = Vec<Vec<i32>>;

/// Possible smoothing methods.
pub enum Smoothing {
    Laplacian(usize, f64),
    Taubin(usize, f64, f64),
}

/// The finite elements type.
pub struct FiniteElements<const N: usize> {
    boundary_nodes: Nodes,
    element_blocks: Blocks,
    element_node_connectivity: Connectivity<N>,
    exterior_nodes: Nodes,
    interface_nodes: Nodes,
    interior_nodes: Nodes,
    nodal_coordinates: Coordinates,
    nodal_influencers: VecConnectivity,
    node_element_connectivity: VecConnectivity,
    node_node_connectivity: VecConnectivity,
    prescribed_nodes: Nodes,
    prescribed_nodes_homogeneous: Nodes,
    prescribed_nodes_inhomogeneous: Nodes,
    prescribed_nodes_inhomogeneous_coordinates: Coordinates,
}

/// The number of nodes in a hexahedral finite element.
pub const HEX: usize = 8;

/// The hexahedral finite elements type.
pub type HexahedralFiniteElements = FiniteElements<HEX>;

/// Methods common to all finite element types.
pub trait FiniteElementMethods<const N: usize>
where
    Self: FiniteElementSpecifics + Sized,
{
    /// Constructs and returns a new finite elements type from data.
    fn from_data(
        element_blocks: Blocks,
        element_node_connectivity: Connectivity<N>,
        nodal_coordinates: Coordinates,
    ) -> Self;
    /// Constructs and returns a new finite elements type from an Abaqus input file.
    fn from_inp(file_path: &str) -> Result<Self, ErrorIO>;
    /// Calculates and returns the discrete Laplacian for the given node-to-node connectivity.
    fn laplacian(&self, node_node_connectivity: &VecConnectivity) -> Coordinates;
    /// Calculates and sets the nodal influencers.
    fn nodal_influencers(&mut self);
    /// Calculates and sets the nodal hierarchy.
    fn nodal_hierarchy(&mut self) -> Result<(), &str>;
    /// Calculates and sets the node-to-element connectivity.
    fn node_element_connectivity(&mut self) -> Result<(), &str>;
    /// Calculates and sets the node-to-node connectivity.
    fn node_node_connectivity(&mut self) -> Result<(), &str>;
    /// Smooths the nodal coordinates according to the provided smoothing method.
    fn smooth(&mut self, method: Smoothing) -> Result<(), &str>;
    /// Writes the finite elements data to a new Exodus file.
    fn write_exo(&self, file_path: &str) -> Result<(), ErrorNetCDF>;
    /// Writes the finite elements data to a new Abaqus file.
    fn write_inp(&self, file_path: &str) -> Result<(), ErrorIO>;
    /// Writes the finite elements data to a new Mesh file.
    fn write_mesh(&self, file_path: &str) -> Result<(), ErrorIO>;
    /// Writes the finite elements quality metrics to a new file.
    fn write_metrics(&self, file_path: &str) -> Result<(), ErrorIO>;
    /// Writes the finite elements data to a new VTK file.
    fn write_vtk(&self, file_path: &str) -> Result<(), ErrorVtk>;
    /// Returns a reference to the boundary nodes.
    fn get_boundary_nodes(&self) -> &Nodes;
    /// Returns a reference to the element blocks.
    fn get_element_blocks(&self) -> &Blocks;
    /// Returns a reference to element-to-node connectivity.
    fn get_element_node_connectivity(&self) -> &Connectivity<N>;
    /// Returns a reference to the exterior nodes.
    fn get_exterior_nodes(&self) -> &Nodes;
    /// Returns a reference to the interface nodes.
    fn get_interface_nodes(&self) -> &Nodes;
    /// Returns a reference to the interior nodes.
    fn get_interior_nodes(&self) -> &Nodes;
    /// Returns a reference to the nodal coordinates.
    fn get_nodal_coordinates(&self) -> &Coordinates;
    /// Returns a mutable reference to the nodal coordinates.
    fn get_nodal_coordinates_mut(&mut self) -> &mut Coordinates;
    /// Returns a reference to the nodal influencers.
    fn get_nodal_influencers(&self) -> &VecConnectivity;
    /// Returns a reference to the node-to-element connectivity.
    fn get_node_element_connectivity(&self) -> &VecConnectivity;
    /// Returns a reference to the node-to-node connectivity.
    fn get_node_node_connectivity(&self) -> &VecConnectivity;
    /// Returns a reference to the prescribed nodes.
    fn get_prescribed_nodes(&self) -> &Nodes;
    /// Returns a reference to the homogeneously-prescribed nodes.
    fn get_prescribed_nodes_homogeneous(&self) -> &Nodes;
    /// Returns a reference to the inhomogeneously-prescribed nodes.
    fn get_prescribed_nodes_inhomogeneous(&self) -> &Nodes;
    /// Returns a reference to the coordinates of the inhomogeneously-prescribed nodes.
    fn get_prescribed_nodes_inhomogeneous_coordinates(&self) -> &Coordinates;
    /// Sets the prescribed nodes if opted to do so.
    fn set_prescribed_nodes(
        &mut self,
        homogeneous: Option<Nodes>,
        inhomogeneous: Option<(Coordinates, Nodes)>,
    ) -> Result<(), &str>;
}

impl<const N: usize> FiniteElementMethods<N> for FiniteElements<N>
where
    Self: FiniteElementSpecifics + Sized,
{
    fn from_data(
        element_blocks: Blocks,
        element_node_connectivity: Connectivity<N>,
        nodal_coordinates: Coordinates,
    ) -> Self {
        Self {
            boundary_nodes: vec![],
            element_blocks,
            element_node_connectivity,
            exterior_nodes: vec![],
            interface_nodes: vec![],
            interior_nodes: vec![],
            nodal_coordinates,
            nodal_influencers: vec![],
            node_element_connectivity: vec![],
            node_node_connectivity: vec![],
            prescribed_nodes: vec![],
            prescribed_nodes_homogeneous: vec![],
            prescribed_nodes_inhomogeneous: vec![],
            prescribed_nodes_inhomogeneous_coordinates: Coordinates::zero(0),
        }
    }
    fn from_inp(file_path: &str) -> Result<Self, ErrorIO> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_inp(file_path)?;
        Ok(Self::from_data(
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        ))
    }
    fn laplacian(&self, node_node_connectivity: &VecConnectivity) -> Coordinates {
        let nodal_coordinates = self.get_nodal_coordinates();
        node_node_connectivity
            .iter()
            .enumerate()
            .map(|(node_index_i, connectivity)| {
                if connectivity.is_empty() {
                    Coordinate::zero()
                } else {
                    connectivity
                        .iter()
                        .map(|node_j| nodal_coordinates[node_j - NODE_NUMBERING_OFFSET].clone())
                        .sum::<Coordinate>()
                        / (connectivity.len() as f64)
                        - &nodal_coordinates[node_index_i]
                }
            })
            .collect()
    }
    fn nodal_influencers(&mut self) {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut nodal_influencers: VecConnectivity = self.get_node_node_connectivity().clone();
        let prescribed_nodes = self.get_prescribed_nodes();
        if !self.get_exterior_nodes().is_empty() {
            let mut boundary_nodes = self.get_boundary_nodes().clone();
            boundary_nodes
                .retain(|boundary_node| prescribed_nodes.binary_search(boundary_node).is_err());
            boundary_nodes.iter().for_each(|boundary_node| {
                nodal_influencers[boundary_node - NODE_NUMBERING_OFFSET].retain(|node| {
                    boundary_nodes.binary_search(node).is_ok()
                        || prescribed_nodes.binary_search(node).is_ok()
                })
            });
        }
        prescribed_nodes.iter().for_each(|prescribed_node| {
            nodal_influencers[prescribed_node - NODE_NUMBERING_OFFSET].clear()
        });
        self.nodal_influencers = nodal_influencers;
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mNodal influencers\x1b[0m {:?} ",
            time.elapsed()
        );
    }
    fn nodal_hierarchy(&mut self) -> Result<(), &str> {
        if N != HEX {
            return Err("Only implemented nodal_hierarchy method for hexes.");
        }
        let node_element_connectivity = self.get_node_element_connectivity();
        if !node_element_connectivity.is_empty() {
            #[cfg(feature = "profile")]
            let time = Instant::now();
            let element_blocks = self.get_element_blocks();
            let mut connected_blocks: Blocks = vec![];
            let mut exterior_nodes = vec![];
            let mut interface_nodes = vec![];
            let mut interior_nodes = vec![];
            let mut number_of_connected_blocks = 0;
            let mut number_of_connected_elements = 0;
            node_element_connectivity
                .iter()
                .enumerate()
                .for_each(|(node, connected_elements)| {
                    connected_blocks = connected_elements
                        .iter()
                        .map(|element| element_blocks[element - ELEMENT_NUMBERING_OFFSET])
                        .collect();
                    connected_blocks.sort();
                    connected_blocks.dedup();
                    number_of_connected_blocks = connected_blocks.len();
                    number_of_connected_elements = connected_elements.len();
                    if number_of_connected_blocks > 1 {
                        interface_nodes.push(node + NODE_NUMBERING_OFFSET);
                        //
                        // THIS IS WHERE IT IS ASSUMED THAT THE MESH IS PERFECTLY STRUCTURED
                        // ONLY AFFECTS HIERARCHICAL SMOOTHING
                        //
                        if number_of_connected_elements < HEX {
                            exterior_nodes.push(node + NODE_NUMBERING_OFFSET);
                        }
                    } else if number_of_connected_elements < HEX {
                        exterior_nodes.push(node + NODE_NUMBERING_OFFSET);
                    } else {
                        interior_nodes.push(node + NODE_NUMBERING_OFFSET);
                    }
                });
            exterior_nodes.sort();
            interior_nodes.sort();
            interface_nodes.sort();
            self.boundary_nodes = exterior_nodes
                .clone()
                .into_iter()
                .chain(interface_nodes.clone())
                .collect();
            self.boundary_nodes.sort();
            self.boundary_nodes.dedup();
            self.exterior_nodes = exterior_nodes;
            self.interface_nodes = interface_nodes;
            self.interior_nodes = interior_nodes;
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mNodal hierarchy\x1b[0m {:?} ",
                time.elapsed()
            );
            Ok(())
        } else {
            Err("Need to calculate the node-to-element connectivity first")
        }
    }
    fn node_element_connectivity(&mut self) -> Result<(), &str> {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let number_of_nodes = self.get_nodal_coordinates().len();
        let mut node_element_connectivity = vec![vec![]; number_of_nodes];
        self.get_element_node_connectivity()
            .iter()
            .enumerate()
            .for_each(|(element, connectivity)| {
                connectivity.iter().for_each(|node| {
                    node_element_connectivity[node - NODE_NUMBERING_OFFSET]
                        .push(element + ELEMENT_NUMBERING_OFFSET)
                })
            });
        self.node_element_connectivity = node_element_connectivity;
        #[cfg(feature = "profile")]
        println!(
            "           \x1b[1;93m⤷ Node-to-element connectivity\x1b[0m {:?} ",
            time.elapsed()
        );
        Ok(())
    }
    fn node_node_connectivity(&mut self) -> Result<(), &str> {
        let node_element_connectivity = self.get_node_element_connectivity();
        if !node_element_connectivity.is_empty() {
            #[cfg(feature = "profile")]
            let time = Instant::now();
            let mut element_connectivity = [0; N];
            let element_node_connectivity = self.get_element_node_connectivity();
            let number_of_nodes = self.get_nodal_coordinates().len();
            let mut node_node_connectivity: VecConnectivity = vec![vec![]; number_of_nodes];
            node_node_connectivity
                .iter_mut()
                .zip(node_element_connectivity.iter().enumerate())
                .try_for_each(|(connectivity, (node, node_connectivity))| {
                    node_connectivity.iter().try_for_each(|element| {
                        element_connectivity.clone_from(
                            &element_node_connectivity[element - ELEMENT_NUMBERING_OFFSET],
                        );
                        if let Some(neighbors) = element_connectivity
                            .iter()
                            .position(|&n| n == node + NODE_NUMBERING_OFFSET)
                        {
                            Self::connected_nodes(&neighbors)
                                .iter()
                                .for_each(|&neighbor| {
                                    connectivity.push(element_connectivity[neighbor])
                                });
                            Ok(())
                        } else {
                            Err("The element-to-node connectivity has been incorrectly calculated")
                        }
                    })
                })?;
            node_node_connectivity.iter_mut().for_each(|connectivity| {
                connectivity.sort();
                connectivity.dedup();
            });
            self.node_node_connectivity = node_node_connectivity;
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mNode-to-node connectivity\x1b[0m {:?} ",
                time.elapsed()
            );
            Ok(())
        } else {
            Err("Need to calculate the node-to-element connectivity first")
        }
    }
    fn smooth(&mut self, method: Smoothing) -> Result<(), &str> {
        if !self.get_node_node_connectivity().is_empty() {
            let smoothing_iterations;
            let smoothing_scale_deflate;
            let mut smoothing_scale_inflate = 0.0;
            match method {
                Smoothing::Laplacian(iterations, scale) => {
                    if scale <= 0.0 || scale >= 1.0 {
                        return Err("Need to specify 0.0 < scale < 1.0");
                    } else {
                        smoothing_iterations = iterations;
                        smoothing_scale_deflate = scale;
                    }
                }
                Smoothing::Taubin(iterations, pass_band, scale) => {
                    if pass_band <= 0.0 || pass_band >= 1.0 {
                        return Err("Need to specify 0.0 < pass-band < 1.0");
                    } else if scale <= 0.0 || scale >= 1.0 {
                        return Err("Need to specify 0.0 < scale < 1.0");
                    } else {
                        smoothing_iterations = iterations;
                        smoothing_scale_deflate = scale;
                        smoothing_scale_inflate = scale / (pass_band * scale - 1.0);
                        if smoothing_scale_deflate >= -smoothing_scale_inflate {
                            return Err("Inflation scale must be larger than deflation scale");
                        }
                    }
                }
            }
            let prescribed_nodes_inhomogeneous = self.get_prescribed_nodes_inhomogeneous().clone();
            let prescribed_nodes_inhomogeneous_coordinates: Coordinates = self
                .get_prescribed_nodes_inhomogeneous_coordinates()
                .iter()
                .cloned()
                .collect();
            let nodal_coordinates_mut = self.get_nodal_coordinates_mut();
            prescribed_nodes_inhomogeneous
                .iter()
                .zip(prescribed_nodes_inhomogeneous_coordinates.iter())
                .for_each(|(node, coordinates)| {
                    nodal_coordinates_mut[node - NODE_NUMBERING_OFFSET] = coordinates.clone()
                });
            let mut iteration = 1;
            let mut laplacian;
            let mut scale;
            #[cfg(feature = "profile")]
            let mut frequency = 1;
            #[cfg(feature = "profile")]
            while (smoothing_iterations / frequency) > 10 {
                frequency *= 10;
            }
            #[cfg(feature = "profile")]
            let remainder = (smoothing_iterations as f64 / frequency as f64 == 10.0) as usize;
            #[cfg(feature = "profile")]
            let width = smoothing_iterations.to_string().len();
            #[cfg(feature = "profile")]
            let mut time = Instant::now();
            while iteration <= smoothing_iterations {
                scale = if smoothing_scale_inflate < 0.0 && iteration % 2 == 1 {
                    smoothing_scale_inflate
                } else {
                    smoothing_scale_deflate
                };
                laplacian = self.laplacian(self.get_nodal_influencers());
                self.get_nodal_coordinates_mut()
                    .iter_mut()
                    .zip(laplacian.iter())
                    .for_each(|(coordinate, entry)| *coordinate += entry * scale);
                #[cfg(feature = "profile")]
                if frequency == 1 {
                    println!(
                        "             \x1b[1;93mSmoothing iteration {}\x1b[0m {:?} ",
                        format_args!("{:width$}", iteration, width = width),
                        time.elapsed()
                    );
                    time = Instant::now();
                } else if iteration % frequency == 0 {
                    println!(
                        "             \x1b[1;93mSmoothing iterations {}..{}\x1b[0m {:?} ",
                        format_args!(
                            "{:width$}",
                            iteration - frequency + 1,
                            width = width - remainder
                        ),
                        format_args!("{:.>width$}", iteration, width = width),
                        time.elapsed()
                    );
                    time = Instant::now();
                }
                iteration += 1;
            }
            #[cfg(feature = "profile")]
            if smoothing_iterations % frequency != 0 {
                println!(
                    "             \x1b[1;93mSmoothing iterations {}..{}\x1b[0m {:?} ",
                    format_args!(
                        "{:width$}",
                        iteration - 1 - (smoothing_iterations % frequency),
                        width = width - remainder
                    ),
                    format_args!("{:.>width$}", smoothing_iterations, width = width),
                    time.elapsed()
                );
            }
            Ok(())
        } else {
            Err("Need to calculate the node-to-node connectivity first")
        }
    }
    fn write_exo(&self, file_path: &str) -> Result<(), ErrorNetCDF> {
        write_finite_elements_to_exodus(
            file_path,
            self.get_element_blocks(),
            self.get_element_node_connectivity(),
            self.get_nodal_coordinates(),
        )
    }
    fn write_inp(&self, file_path: &str) -> Result<(), ErrorIO> {
        write_finite_elements_to_abaqus(
            file_path,
            self.get_element_blocks(),
            self.get_element_node_connectivity(),
            self.get_nodal_coordinates(),
        )
    }
    fn write_mesh(&self, file_path: &str) -> Result<(), ErrorIO> {
        write_finite_elements_to_mesh(
            file_path,
            self.get_element_blocks(),
            self.get_element_node_connectivity(),
            self.get_nodal_coordinates(),
        )
    }
    fn write_metrics(&self, file_path: &str) -> Result<(), ErrorIO> {
        write_finite_elements_metrics(
            file_path,
            self.get_element_node_connectivity(),
            self.get_nodal_coordinates(),
        )
    }
    fn write_vtk(&self, file_path: &str) -> Result<(), ErrorVtk> {
        write_finite_elements_to_vtk(
            file_path,
            self.get_element_blocks(),
            self.get_element_node_connectivity(),
            self.get_nodal_coordinates(),
        )
    }
    fn get_boundary_nodes(&self) -> &Nodes {
        &self.boundary_nodes
    }
    fn get_element_blocks(&self) -> &Blocks {
        &self.element_blocks
    }
    fn get_element_node_connectivity(&self) -> &Connectivity<N> {
        &self.element_node_connectivity
    }
    fn get_exterior_nodes(&self) -> &Nodes {
        &self.exterior_nodes
    }
    fn get_interface_nodes(&self) -> &Nodes {
        &self.interface_nodes
    }
    fn get_interior_nodes(&self) -> &Nodes {
        &self.interior_nodes
    }
    fn get_nodal_coordinates(&self) -> &Coordinates {
        &self.nodal_coordinates
    }
    fn get_nodal_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.nodal_coordinates
    }
    fn get_nodal_influencers(&self) -> &VecConnectivity {
        &self.nodal_influencers
    }
    fn get_node_element_connectivity(&self) -> &VecConnectivity {
        &self.node_element_connectivity
    }
    fn get_node_node_connectivity(&self) -> &VecConnectivity {
        &self.node_node_connectivity
    }
    fn get_prescribed_nodes(&self) -> &Nodes {
        &self.prescribed_nodes
    }
    fn get_prescribed_nodes_homogeneous(&self) -> &Nodes {
        &self.prescribed_nodes_homogeneous
    }
    fn get_prescribed_nodes_inhomogeneous(&self) -> &Nodes {
        &self.prescribed_nodes_inhomogeneous
    }
    fn get_prescribed_nodes_inhomogeneous_coordinates(&self) -> &Coordinates {
        &self.prescribed_nodes_inhomogeneous_coordinates
    }
    fn set_prescribed_nodes(
        &mut self,
        homogeneous: Option<Nodes>,
        inhomogeneous: Option<(Coordinates, Nodes)>,
    ) -> Result<(), &str> {
        if let Some(homogeneous_nodes) = homogeneous {
            self.prescribed_nodes_homogeneous = homogeneous_nodes;
            self.prescribed_nodes_homogeneous.sort();
            self.prescribed_nodes_homogeneous.dedup();
        }
        if let Some(inhomogeneous_nodes) = inhomogeneous {
            self.prescribed_nodes_inhomogeneous = inhomogeneous_nodes.1;
            self.prescribed_nodes_inhomogeneous_coordinates = inhomogeneous_nodes.0;
            let mut sorted_unique = self.prescribed_nodes_inhomogeneous.clone();
            sorted_unique.sort();
            sorted_unique.dedup();
            if sorted_unique != self.prescribed_nodes_inhomogeneous {
                return Err("Inhomogeneously-prescribed nodes must be sorted and unique.");
            }
        }
        self.prescribed_nodes = self
            .prescribed_nodes_homogeneous
            .clone()
            .into_iter()
            .chain(self.prescribed_nodes_inhomogeneous.clone())
            .collect();
        Ok(())
    }
}

/// Methods specific to each finite element type.
pub trait FiniteElementSpecifics {
    /// Returns the nodes connected to the given node within an element.
    fn connected_nodes(node: &usize) -> Vec<usize>;
    /// Converts the finite elements into a tessellation, consuming the finite elements.
    fn into_tesselation(self) -> Tessellation;
}

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

fn reorder_connectivity<const N: usize>(
    element_blocks: &Blocks,
    element_blocks_unique: &Blocks,
    element_node_connectivity: &Connectivity<N>,
) -> ReorderedConnectivity {
    element_blocks_unique
        .iter()
        .map(|unique_block| {
            element_blocks
                .iter()
                .enumerate()
                .filter(|&(_, &block)| &block == unique_block)
                .flat_map(|(element, _)| {
                    element_node_connectivity[element]
                        .iter()
                        .map(|entry| *entry as i32)
                        .collect::<Vec<i32>>()
                })
                .collect::<Vec<i32>>()
        })
        .collect()
}

fn automesh_header() -> String {
    format!(
        "autotwin.automesh, version {}, autogenerated on {}",
        env!("CARGO_PKG_VERSION"),
        Utc::now()
    )
}

fn finite_element_data_from_inp<const N: usize>(
    file_path: &str,
) -> Result<(Blocks, Connectivity<N>, Coordinates), ErrorIO> {
    let inp_file = File::open(file_path)?;
    let mut file = BufReader::new(inp_file);
    let mut buffer = String::new();
    while buffer != "*NODE, NSET=ALLNODES\n" {
        buffer.clear();
        file.read_line(&mut buffer)?;
    }
    buffer.clear();
    file.read_line(&mut buffer)?;
    let mut nodal_coordinates = Coordinates::zero(0);
    let mut inverse_mapping: Vec<usize> = vec![];
    while buffer != "**\n" {
        inverse_mapping.push(
            buffer
                .trim()
                .split(",")
                .take(1)
                .next()
                .unwrap()
                .trim()
                .parse()
                .unwrap(),
        );
        nodal_coordinates.push(
            buffer
                .trim()
                .split(",")
                .skip(1)
                .map(|entry| entry.trim().parse().unwrap())
                .collect(),
        );
        buffer.clear();
        file.read_line(&mut buffer)?;
    }
    let mut mapping = vec![0_usize; *inverse_mapping.iter().max().unwrap()];
    inverse_mapping
        .iter()
        .enumerate()
        .for_each(|(new, old)| mapping[old - NODE_NUMBERING_OFFSET] = new + NODE_NUMBERING_OFFSET);
    buffer.clear();
    file.read_line(&mut buffer)?;
    buffer.clear();
    file.read_line(&mut buffer)?;
    let mut current_block = 0;
    let mut element_blocks: Blocks = vec![];
    let mut element_node_connectivity: Connectivity<N> = vec![];
    let mut element_numbers = vec![];
    while buffer != "**\n" {
        if buffer.trim().chars().take(8).collect::<String>() == "*ELEMENT" {
            current_block = buffer.trim().chars().last().unwrap().to_digit(10).unwrap() as u8;
        } else {
            element_blocks.push(current_block);
            element_node_connectivity.push(
                buffer
                    .trim()
                    .split(",")
                    .skip(1)
                    .map(|entry| entry.trim().parse::<usize>().unwrap())
                    .collect::<Vec<usize>>()
                    .try_into()
                    .unwrap(),
            );
            element_numbers.push(
                buffer
                    .trim()
                    .split(",")
                    .take(1)
                    .next()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
            );
        }
        buffer.clear();
        file.read_line(&mut buffer)?;
    }
    element_node_connectivity
        .iter_mut()
        .for_each(|connectivity| {
            connectivity
                .iter_mut()
                .for_each(|node| *node = mapping[*node - NODE_NUMBERING_OFFSET])
        });
    Ok((element_blocks, element_node_connectivity, nodal_coordinates))
}

fn write_finite_elements_to_exodus<const N: usize>(
    file_path: &str,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Result<(), ErrorNetCDF> {
    let mut file = create(file_path)?;
    file.add_attribute::<f32>("api_version", 8.25)?;
    file.add_attribute::<i32>("file_size", 1)?;
    file.add_attribute::<i32>("floating_point_word_size", 8)?;
    file.add_attribute::<String>("title", automesh_header())?;
    file.add_attribute::<f32>("version", 8.25)?;
    let mut element_blocks_unique = element_blocks.clone();
    element_blocks_unique.sort();
    element_blocks_unique.dedup();
    file.add_dimension("num_dim", NSD)?;
    file.add_dimension("num_elem", element_blocks.len())?;
    file.add_dimension("num_el_blk", element_blocks_unique.len())?;
    let mut eb_prop1 = file.add_variable::<i32>("eb_prop1", &["num_el_blk"])?;
    element_blocks_unique
        .iter()
        .enumerate()
        .try_for_each(|(index, unique_block)| eb_prop1.put_value(*unique_block as i32, index))?;
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let block_connectivities = reorder_connectivity(
        element_blocks,
        &element_blocks_unique,
        element_node_connectivity,
    );
    let mut current_block = 0;
    let mut number_of_elements = 0;
    element_blocks_unique
        .iter()
        .zip(block_connectivities.into_iter())
        .try_for_each(|(unique_block, block_connectivity)| {
            current_block += 1;
            number_of_elements = element_blocks
                .iter()
                .filter(|&block| block == unique_block)
                .count();
            file.add_dimension(
                format!("num_el_in_blk{}", current_block).as_str(),
                number_of_elements,
            )?;
            file.add_dimension(format!("num_nod_per_el{}", current_block).as_str(), N)?;
            let mut connectivities = file.add_variable::<i32>(
                format!("connect{}", current_block).as_str(),
                &[
                    format!("num_el_in_blk{}", current_block).as_str(),
                    format!("num_nod_per_el{}", current_block).as_str(),
                ],
            )?;
            match N {
                HEX => connectivities.put_attribute("elem_type", "HEX8")?,
                TRI => connectivities.put_attribute("elem_type", "TRI3")?,
                _ => panic!(),
            };
            connectivities.put_values(&block_connectivity, (.., ..))?;
            Ok::<_, ErrorNetCDF>(())
        })?;
    #[cfg(feature = "profile")]
    println!(
        "           \x1b[1;93m⤷ Element-to-node connectivity\x1b[0m {:?}",
        time.elapsed()
    );
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let xs: Vec<f64> = nodal_coordinates.iter().map(|coords| coords[0]).collect();
    let ys: Vec<f64> = nodal_coordinates.iter().map(|coords| coords[1]).collect();
    let zs: Vec<f64> = nodal_coordinates.iter().map(|coords| coords[2]).collect();
    file.add_dimension("num_nodes", nodal_coordinates.len())?;
    file.add_variable::<f64>("coordx", &["num_nodes"])?
        .put_values(&xs, 0..)?;
    file.add_variable::<f64>("coordy", &["num_nodes"])?
        .put_values(&ys, 0..)?;
    file.add_variable::<f64>("coordz", &["num_nodes"])?
        .put_values(&zs, 0..)?;
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mNodal coordinates\x1b[0m {:?}",
        time.elapsed()
    );
    Ok(())
}

fn write_finite_elements_to_abaqus<const N: usize>(
    file_path: &str,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Result<(), ErrorIO> {
    let element_number_width = get_width(element_node_connectivity);
    let node_number_width = nodal_coordinates.len().to_string().chars().count();
    let inp_file = File::create(file_path)?;
    let mut file = BufWriter::new(inp_file);
    write_heading_to_inp(&mut file)?;
    write_nodal_coordinates_to_inp(&mut file, nodal_coordinates, &node_number_width)?;
    write_element_node_connectivity_to_inp(
        &mut file,
        element_blocks,
        element_node_connectivity,
        &element_number_width,
        &node_number_width,
    )?;
    file.flush()
}

fn write_heading_to_inp(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    let prefix = "*HEADING\n";
    let middle = automesh_header().replace(", ", "\n");
    let postfix = "**\n";
    let heading = format!("{}{}{}", prefix, middle, postfix);
    file.write_all(heading.as_bytes())?;
    end_section(file)
}

fn write_nodal_coordinates_to_inp(
    file: &mut BufWriter<File>,
    nodal_coordinates: &Coordinates,
    node_number_width: &usize,
) -> Result<(), ErrorIO> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    file.write_all(
        "********************************** N O D E S **********************************\n"
            .as_bytes(),
    )?;
    file.write_all("*NODE, NSET=ALLNODES".as_bytes())?;
    nodal_coordinates
        .iter()
        .enumerate()
        .try_for_each(|(node, coordinates)| {
            indent(file)?;
            file.write_all(
                format!(
                    "{:>width$}",
                    node + NODE_NUMBERING_OFFSET,
                    width = node_number_width
                )
                .as_bytes(),
            )?;
            coordinates.iter().try_for_each(|coordinate| {
                delimiter(file)?;
                file.write_all(format!("{:>15.6e}", coordinate).as_bytes())
            })
        })?;
    newline(file)?;
    let result = end_section(file);
    #[cfg(feature = "profile")]
    println!(
        "           \x1b[1;93m⤷ Nodal coordinates\x1b[0m {:?}",
        time.elapsed()
    );
    result
}

fn write_element_node_connectivity_to_inp<const N: usize>(
    file: &mut BufWriter<File>,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity<N>,
    element_number_width: &usize,
    node_number_width: &usize,
) -> Result<(), ErrorIO> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let element_type = match N {
        HEX => "C3D8R",
        TRI => "TRI3",
        _ => panic!(),
    };
    file.write_all(
        "********************************** E L E M E N T S ****************************\n"
            .as_bytes(),
    )?;
    let mut element_blocks_unique = element_blocks.clone();
    element_blocks_unique.sort();
    element_blocks_unique.dedup();
    element_blocks_unique
        .iter()
        .clone()
        .try_for_each(|current_block| {
            file.write_all(
                format!("*ELEMENT, TYPE={}, ELSET=EB{}", element_type, current_block).as_bytes(),
            )?;
            element_blocks
                .iter()
                .enumerate()
                .filter(|(_, block)| block == &current_block)
                .try_for_each(|(element, _)| {
                    indent(file)?;
                    file.write_all(
                        format!(
                            "{:>width$}",
                            element + ELEMENT_NUMBERING_OFFSET,
                            width = element_number_width
                        )
                        .as_bytes(),
                    )?;
                    element_node_connectivity[element]
                        .iter()
                        .try_for_each(|entry| {
                            delimiter(file)?;
                            file.write_all(
                                format!("{:>width$}", entry, width = node_number_width + 3)
                                    .as_bytes(),
                            )
                        })
                })?;
            newline(file)
        })?;
    end_section(file)?;
    let result = element_blocks_unique.iter().try_for_each(|block| {
        file.write_all(
            format!(
                "*SOLID SECTION, ELSET=EB{}, MATERIAL=Default-Steel\n",
                block
            )
            .as_bytes(),
        )
    });
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mElement-to-node connectivity\x1b[0m {:?}",
        time.elapsed()
    );
    result
}

fn end_section(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    file.write_all(&[42, 42, 10])
}

fn delimiter(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    file.write_all(&[44, 32])
}

fn indent(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    file.write_all(&[10, 32, 32, 32, 32])
}

fn newline(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    file.write_all(&[10])
}

fn get_width<T>(input: &[T]) -> usize {
    input.len().to_string().chars().count()
}

fn write_finite_elements_to_mesh<const N: usize>(
    file_path: &str,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Result<(), ErrorIO> {
    let mesh_file = File::create(file_path)?;
    let mut file = BufWriter::new(mesh_file);
    file.write_all(b"MeshVersionFormatted 1\nDimension 3\nVertices\n")?;
    file.write_all(format!("{}\n", nodal_coordinates.len()).as_bytes())?;
    nodal_coordinates.iter().try_for_each(|coordinates| {
        coordinates
            .iter()
            .try_for_each(|coordinate| file.write_all(format!("{} ", coordinate).as_bytes()))?;
        file.write_all(b"0\n")
    })?;
    match N {
        HEX => file.write_all(b"Hexahedra\n")?,
        TRI => file.write_all(b"Triangles\n")?,
        _ => panic!(),
    };
    file.write_all(format!("{}\n", element_blocks.len()).as_bytes())?;
    element_node_connectivity
        .iter()
        .try_for_each(|connectivity| {
            connectivity
                .iter()
                .try_for_each(|node| file.write_all(format!("{} ", node).as_bytes()))?;
            file.write_all(b"0\n")
        })?;
    file.write_all(b"End")?;
    file.flush()
}

fn write_finite_elements_to_vtk<const N: usize>(
    file_path: &str,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Result<(), ErrorVtk> {
    let connectivity = element_node_connectivity
        .iter()
        .flatten()
        .map(|node| (node - NODE_NUMBERING_OFFSET) as u64)
        .collect();
    let nodal_coordinates_flattened = nodal_coordinates
        .iter()
        .flat_map(|entry| entry.iter())
        .copied()
        .collect();
    let number_of_cells = element_blocks.len();
    let offsets = (0..number_of_cells)
        .map(|cell| ((cell + 1) * N) as u64)
        .collect();
    let types = match N {
        HEX => vec![CellType::Hexahedron; number_of_cells],
        TRI => vec![CellType::Triangle; number_of_cells],
        _ => panic!(),
    };
    let file = PathBuf::from(file_path);
    Vtk {
        version: Version { major: 4, minor: 2 },
        title: automesh_header(),
        byte_order: ByteOrder::BigEndian,
        file_path: None,
        data: DataSet::inline(UnstructuredGridPiece {
            points: IOBuffer::F64(nodal_coordinates_flattened),
            cells: Cells {
                cell_verts: VertexNumbers::XML {
                    connectivity,
                    offsets,
                },
                types,
            },
            data: Attributes {
                ..Default::default()
            },
        }),
    }
    .export_be(&file)
}

fn metrics_headers<const N: usize>() -> String {
    match N {
        HEX => {
            "maximum edge ratio,minimum scaled jacobian,maximum skew,element volume\n".to_string()
        }
        TRI => {
            "maximum edge ratio,minimum scaled jacobian,maximum skew,element area,minimum angle\n"
                .to_string()
        }
        _ => panic!(),
    }
}

fn write_finite_elements_metrics<const N: usize>(
    file_path: &str,
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Result<(), ErrorIO> {
    match N {
        HEX => write_finite_elements_metrics_hex(
            file_path,
            element_node_connectivity,
            nodal_coordinates,
        ),
        TRI => write_finite_elements_metrics_tri(
            file_path,
            element_node_connectivity,
            nodal_coordinates,
        ),
        _ => panic!(),
    }
}

fn write_finite_elements_metrics_hex<const N: usize>(
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

fn calculate_maximum_edge_ratios<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let maximum_edge_ratios = match N {
        HEX => calculate_maximum_edge_ratios_hex(element_node_connectivity, nodal_coordinates),
        TRI => calculate_maximum_edge_ratios_tri(element_node_connectivity, nodal_coordinates),
        _ => panic!(),
    };
    #[cfg(feature = "profile")]
    println!(
        "           \x1b[1;93m⤷ Maximum edge ratios\x1b[0m {:?}",
        time.elapsed()
    );
    maximum_edge_ratios
}

fn calculate_minimum_scaled_jacobians<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let minimum_scaled_jacobians = match N {
        HEX => calculate_minimum_scaled_jacobians_hex(element_node_connectivity, nodal_coordinates),
        TRI => calculate_minimum_scaled_jacobians_tri(element_node_connectivity, nodal_coordinates),
        _ => panic!(),
    };
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mMinimum scaled Jacobians\x1b[0m {:?}",
        time.elapsed()
    );
    minimum_scaled_jacobians
}

fn calculate_maximum_skews<const N: usize>(
    element_node_connectivity: &Connectivity<N>,
    nodal_coordinates: &Coordinates,
) -> Metrics {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let maximum_skews = match N {
        HEX => calculate_maximum_skews_hex(element_node_connectivity, nodal_coordinates),
        TRI => calculate_maximum_skews_tri(element_node_connectivity, nodal_coordinates),
        _ => panic!(),
    };
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mMaximum edge skews\x1b[0m {:?}",
        time.elapsed()
    );
    maximum_skews
}

fn calculate_maximum_edge_ratios_hex<const N: usize>(
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

fn calculate_minimum_scaled_jacobians_hex<const N: usize>(
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

fn calculate_element_principal_axes<const N: usize>(
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

fn calculate_maximum_skews_hex<const N: usize>(
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

fn calculate_element_volumes_hex<const N: usize>(
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
