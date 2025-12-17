#[cfg(feature = "python")]
pub mod py;

pub mod hex;
pub mod tet;
pub mod tri;
pub use hex::{HEX, HexahedralFiniteElements};
pub use tet::{TET, TetrahedralFiniteElements};
pub use tri::{TRI, TriangularFiniteElements};

#[cfg(feature = "profile")]
use std::time::Instant;

use crate::{
    Coordinate, Coordinates, NSD, Tessellation, Vector,
    tree::{HexesAndCoords, Octree, Samples, octree_from_surface},
};
use chrono::Utc;
use conspire::{
    constitutive::solid::hyperelastic::NeoHookean,
    fem::{
        ElementBlock, FiniteElementBlock, FirstOrderMinimize, LinearHexahedron, LinearTetrahedron,
    },
    math::{
        Scalar, Tensor, TensorArray, TensorRank1List, TensorVec,
        optimize::{EqualityConstraint, GradientDescent, LineSearch},
    },
};
use ndarray::{Array1, parallel::prelude::*};
use netcdf::{Error as ErrorNetCDF, create, open};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Error as ErrorIO, Write},
    path::PathBuf,
};
use vtkio::{
    Error as ErrorVtk,
    model::{
        Attributes, ByteOrder, CellType, Cells, DataSet, IOBuffer, UnstructuredGridPiece, Version,
        VertexNumbers, Vtk,
    },
};

const ELEMENT_NUMBERING_OFFSET: usize = 1;
const NODE_NUMBERING_OFFSET: usize = 1;

/// A bounding box described by two opposing corners.
pub type BoundingBox = TensorRank1List<3, 1, 2>;

/// A vector of finite element block IDs.
pub type Blocks = Vec<u8>;

/// An element-to-node connectivity.
pub type Connectivity<const N: usize> = Vec<[usize; N]>;

pub type Data<const N: usize> = (Blocks, Connectivity<N>, Coordinates);
pub type Metrics = Array1<Scalar>;
pub type Nodes = Vec<usize>;
pub type ReorderedConnectivity = Vec<Vec<u32>>;
pub type Size = Option<Scalar>;
pub type VecConnectivity = Vec<Vec<usize>>;

/// Possible smoothing methods.
pub enum Smoothing {
    Energetic,
    Laplacian(usize, Scalar),
    Taubin(usize, Scalar, Scalar),
    None,
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

/// Methods common to all finite element types.
pub trait FiniteElementMethods<const M: usize, const N: usize>
where
    Self: FiniteElementSpecifics<M> + Sized,
{
    /// Calculates and returns the bounding box.
    fn bounding_box(&self) -> BoundingBox;
    /// Calculates and returns the coordinates of the centroids.
    fn centroids(&self) -> Coordinates;
    /// Returns the centroid for each exterior face.
    fn exterior_faces_centroids(&self) -> Coordinates;
    /// Constructs and returns a new finite elements type from an Exodus input file.
    fn from_exo(file_path: &str) -> Result<Self, ErrorNetCDF>;
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
    /// Remove nodes and elements that connect to them.
    fn remove_nodes(self, removed_nodes: Nodes) -> Self;
    /// Removes nodes that are not connected to any elements.
    fn remove_orphan_nodes(self) -> Result<Self, String>;
    /// Smooths the nodal coordinates according to the provided smoothing method.
    fn smooth(&mut self, method: &Smoothing) -> Result<(), String>;
    /// Writes the finite elements data to a new Exodus file.
    fn write_exo(&self, file_path: &str) -> Result<(), ErrorNetCDF>;
    /// Writes the finite elements data to a new Abaqus file.
    fn write_inp(&self, file_path: &str) -> Result<(), ErrorIO>;
    /// Writes the finite elements data to a new Mesh file.
    fn write_mesh(&self, file_path: &str) -> Result<(), ErrorIO>;
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

impl<const M: usize, const N: usize> FiniteElementMethods<M, N> for FiniteElements<N>
where
    Self: FiniteElementSpecifics<M> + Sized,
{
    fn bounding_box(&self) -> BoundingBox {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let (minimum, maximum) = self.get_nodal_coordinates().iter().fold(
            (
                Coordinate::new([f64::INFINITY; NSD]),
                Coordinate::new([f64::NEG_INFINITY; NSD]),
            ),
            |(mut minimum, mut maximum), coordinate| {
                minimum
                    .iter_mut()
                    .zip(maximum.iter_mut().zip(coordinate.iter()))
                    .for_each(|(min, (max, &coord))| {
                        *min = min.min(coord);
                        *max = max.max(coord);
                    });
                (minimum, maximum)
            },
        );
        let bounding_box = BoundingBox::new([minimum.into(), maximum.into()]);
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mBounding box function\x1b[0m {:?}",
            time.elapsed()
        );
        bounding_box
    }
    fn centroids(&self) -> Coordinates {
        let coordinates = self.get_nodal_coordinates();
        let number_of_nodes = N as f64;
        self.get_element_node_connectivity()
            .iter()
            .map(|nodes| {
                nodes
                    .iter()
                    .map(|&node| coordinates[node].clone())
                    .sum::<Coordinate>()
                    / number_of_nodes
            })
            .collect()
    }
    fn exterior_faces_centroids(&self) -> Coordinates {
        let coordinates = self.get_nodal_coordinates();
        let number_of_nodes = M as f64;
        self.exterior_faces()
            .iter()
            .map(|face| {
                face.iter()
                    .map(|&node| coordinates[node].clone())
                    .sum::<Coordinate>()
                    / number_of_nodes
            })
            .collect()
    }
    fn from_exo(file_path: &str) -> Result<Self, ErrorNetCDF> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_exo(file_path)?;
        Ok(Self::from((
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        )))
    }
    fn from_inp(file_path: &str) -> Result<Self, ErrorIO> {
        let (element_blocks, element_node_connectivity, nodal_coordinates) =
            finite_element_data_from_inp(file_path)?;
        Ok(Self::from((
            element_blocks,
            element_node_connectivity,
            nodal_coordinates,
        )))
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
                        .map(|&node_j| nodal_coordinates[node_j].clone())
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
            boundary_nodes.iter().for_each(|&boundary_node| {
                nodal_influencers[boundary_node].retain(|node| {
                    boundary_nodes.binary_search(node).is_ok()
                        || prescribed_nodes.binary_search(node).is_ok()
                })
            });
        }
        prescribed_nodes
            .iter()
            .for_each(|&prescribed_node| nodal_influencers[prescribed_node].clear());
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
                        .map(|&element| element_blocks[element])
                        .collect();
                    connected_blocks.sort();
                    connected_blocks.dedup();
                    number_of_connected_blocks = connected_blocks.len();
                    number_of_connected_elements = connected_elements.len();
                    if number_of_connected_blocks > 1 {
                        interface_nodes.push(node);
                        //
                        // THIS IS WHERE IT IS ASSUMED THAT THE MESH IS PERFECTLY STRUCTURED
                        // ONLY AFFECTS HIERARCHICAL SMOOTHING
                        //
                        if number_of_connected_elements < HEX {
                            exterior_nodes.push(node);
                        }
                    } else if number_of_connected_elements < HEX {
                        exterior_nodes.push(node);
                    } else {
                        interior_nodes.push(node);
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
                connectivity
                    .iter()
                    .for_each(|&node| node_element_connectivity[node].push(element))
            });
        self.node_element_connectivity = node_element_connectivity;
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mInverted connectivity\x1b[0m {:?} ",
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
                    node_connectivity.iter().try_for_each(|&element| {
                        element_connectivity.clone_from(&element_node_connectivity[element]);
                        if let Some(neighbors) =
                            element_connectivity.iter().position(|n| n == &node)
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
                "             \x1b[1;93mNode-node connections\x1b[0m {:?} ",
                time.elapsed()
            );
            Ok(())
        } else {
            Err("Need to calculate the node-to-element connectivity first")
        }
    }
    fn remove_nodes(self, removed_nodes: Nodes) -> Self {
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let mut is_removed_node = vec![false; self.nodal_coordinates.len()];
        removed_nodes
            .iter()
            .for_each(|&removed_node| is_removed_node[removed_node] = true);
        let mut shift = 0;
        let decrements: Nodes = is_removed_node
            .iter()
            .map(|&node_is_removed| {
                if node_is_removed {
                    shift += 1;
                }
                shift
            })
            .collect();
        let mut is_removed_elements = vec![false; self.nodal_coordinates.len()];
        self.element_node_connectivity
            .iter()
            .enumerate()
            .filter(|(_, nodes)| {
                nodes
                    .iter()
                    .any(|node| removed_nodes.binary_search(node).is_ok())
            })
            .for_each(|(removed_element, _)| is_removed_elements[removed_element] = true);
        let element_blocks = self
            .element_blocks
            .into_iter()
            .zip(is_removed_elements.iter())
            .filter_map(|(block, &element_is_removed)| {
                if element_is_removed {
                    None
                } else {
                    Some(block)
                }
            })
            .collect();
        let mut element_node_connectivity: Connectivity<N> = self
            .element_node_connectivity
            .into_iter()
            .zip(is_removed_elements)
            .filter_map(|(connectivity, element_is_removed)| {
                if element_is_removed {
                    None
                } else {
                    Some(connectivity)
                }
            })
            .collect();
        element_node_connectivity
            .iter_mut()
            .for_each(|nodes| nodes.iter_mut().for_each(|node| *node -= decrements[*node]));
        let nodal_coordinates = self
            .nodal_coordinates
            .into_iter()
            .zip(is_removed_node)
            .filter_map(|(coordinates, node_is_removed)| {
                if node_is_removed {
                    None
                } else {
                    Some(coordinates)
                }
            })
            .collect();
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mRemoving marked nodes\x1b[0m {:?}",
            time.elapsed()
        );
        Self::from((element_blocks, element_node_connectivity, nodal_coordinates))
    }
    fn remove_orphan_nodes(mut self) -> Result<Self, String> {
        self.node_element_connectivity()?;
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let orphan_nodes: Vec<usize> = self
            .get_node_element_connectivity()
            .iter()
            .enumerate()
            .filter_map(|(node, elements)| {
                if elements.is_empty() {
                    Some(node)
                } else {
                    None
                }
            })
            .collect();
        let mut is_orphan_node = vec![false; self.nodal_coordinates.len()];
        orphan_nodes
            .iter()
            .for_each(|&orphan_node| is_orphan_node[orphan_node] = true);
        let mut shift = 0;
        let decrements: Nodes = is_orphan_node
            .iter()
            .map(|&node_is_orphan| {
                if node_is_orphan {
                    shift += 1;
                }
                shift
            })
            .collect();
        self.element_node_connectivity
            .iter_mut()
            .for_each(|nodes| nodes.iter_mut().for_each(|node| *node -= decrements[*node]));
        let nodal_coordinates = self
            .nodal_coordinates
            .into_iter()
            .zip(is_orphan_node)
            .filter_map(|(coordinates, node_is_orphan)| {
                if node_is_orphan {
                    None
                } else {
                    Some(coordinates)
                }
            })
            .collect();
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mRemoving orphan nodes\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(Self::from((
            self.element_blocks,
            self.element_node_connectivity,
            nodal_coordinates,
        )))
    }
    fn smooth(&mut self, method: &Smoothing) -> Result<(), String> {
        if !self.get_node_node_connectivity().is_empty() {
            let smoothing_iterations;
            let smoothing_scale_deflate;
            let mut smoothing_scale_inflate = 0.0;
            match *method {
                Smoothing::Energetic => {
                    let mut nodes: Nodes = self.exterior_faces().into_iter().flatten().collect();
                    nodes.sort();
                    nodes.dedup();
                    let solver = GradientDescent {
                        abs_tol: 1e-6,
                        dual: false,
                        line_search: LineSearch::Armijo {
                            control: 1e-3,
                            cut_back: 0.9,
                            max_steps: 100,
                        },
                        max_steps: 1000,
                        rel_tol: Some(1e-2),
                    };
                    let indices = nodes
                        .into_iter()
                        .flat_map(|node: usize| [NSD * node, NSD * node + 1, NSD * node + 2])
                        .collect();
                    self.nodal_coordinates = match N {
                        HEX => {
                            let connectivity = self
                                .get_element_node_connectivity()
                                .iter()
                                .map(|entry| {
                                    entry.iter().copied().collect::<Nodes>().try_into().unwrap()
                                })
                                .collect();
                            let mut block = ElementBlock::<_, LinearHexahedron, _>::new(
                                NeoHookean {
                                    bulk_modulus: 0.0,
                                    shear_modulus: 1.0,
                                },
                                connectivity,
                                self.get_nodal_coordinates().clone().into(),
                            );
                            block.reset();
                            block.minimize(EqualityConstraint::Fixed(indices), solver)?
                        }
                        TET => {
                            let connectivity = self
                                .get_element_node_connectivity()
                                .iter()
                                .map(|entry| {
                                    entry.iter().copied().collect::<Nodes>().try_into().unwrap()
                                })
                                .collect();
                            let mut block = ElementBlock::<_, LinearTetrahedron, _>::new(
                                NeoHookean {
                                    bulk_modulus: 0.0,
                                    shear_modulus: 1.0,
                                },
                                connectivity,
                                self.get_nodal_coordinates().clone().into(),
                            );
                            block.reset();
                            block.minimize(EqualityConstraint::Fixed(indices), solver)?
                        }
                        _ => panic!(),
                    };
                    return Ok(());
                }
                Smoothing::Laplacian(iterations, scale) => {
                    if scale <= 0.0 || scale >= 1.0 {
                        return Err("Need to specify 0.0 < scale < 1.0".to_string());
                    } else {
                        smoothing_iterations = iterations;
                        smoothing_scale_deflate = scale;
                    }
                }
                Smoothing::Taubin(iterations, pass_band, scale) => {
                    if pass_band <= 0.0 || pass_band >= 1.0 {
                        return Err("Need to specify 0.0 < pass-band < 1.0".to_string());
                    } else if scale <= 0.0 || scale >= 1.0 {
                        return Err("Need to specify 0.0 < scale < 1.0".to_string());
                    } else {
                        smoothing_iterations = iterations;
                        smoothing_scale_deflate = scale;
                        smoothing_scale_inflate = scale / (pass_band * scale - 1.0);
                        if smoothing_scale_deflate >= -smoothing_scale_inflate {
                            return Err(
                                "Inflation scale must be larger than deflation scale".to_string()
                            );
                        }
                    }
                }
                Smoothing::None => return Ok(()),
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
                .for_each(|(&node, coordinates)| nodal_coordinates_mut[node] = coordinates.clone());
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
            Err("Need to calculate the node-to-node connectivity first".to_string())
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

impl<const N: usize> From<FiniteElements<N>> for Data<N> {
    fn from(finite_elements: FiniteElements<N>) -> Self {
        (
            finite_elements.element_blocks,
            finite_elements.element_node_connectivity,
            finite_elements.nodal_coordinates,
        )
    }
}

impl<const N: usize> From<FiniteElements<N>>
    for (Blocks, Connectivity<N>, Coordinates, VecConnectivity)
{
    fn from(finite_elements: FiniteElements<N>) -> Self {
        (
            finite_elements.element_blocks,
            finite_elements.element_node_connectivity,
            finite_elements.nodal_coordinates,
            finite_elements.node_element_connectivity,
        )
    }
}

impl<const N: usize> From<Data<N>> for FiniteElements<N> {
    fn from((element_blocks, element_node_connectivity, nodal_coordinates): Data<N>) -> Self {
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
}

impl TryFrom<(Tessellation, Size)> for HexahedralFiniteElements {
    type Error = String;
    fn try_from((tessellation, size): (Tessellation, Size)) -> Result<Self, Self::Error> {
        let mut triangular_finite_elements = TriangularFiniteElements::from(tessellation);
        triangular_finite_elements.node_element_connectivity()?;
        triangular_finite_elements.node_node_connectivity()?;
        triangular_finite_elements.refine(size.unwrap());
        let surface_nodal_coordinates = triangular_finite_elements.get_nodal_coordinates().clone();
        let (
            tree,
            samples,
            surface_element_node_connectivity,
            surface_node_element_connectivity,
            bins,
        ) = octree_from_surface(triangular_finite_elements, size);
        let (hexahedral_finite_elements, coordinates) = HexesAndCoords::from(&tree).into();
        let removed_nodes = mark_outside_nodes(coordinates, samples, &tree);
        let mut finite_elements = hexahedral_finite_elements
            .remove_nodes(removed_nodes)
            .remove_orphan_nodes()?;
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let rounded_coordinates: Vec<_> = finite_elements
            .get_nodal_coordinates()
            .iter()
            .map(|coordinate| {
                [
                    ((coordinate[0] - tree.translate().x()) / tree.scale().x()).floor() as i16,
                    ((coordinate[1] - tree.translate().y()) / tree.scale().y()).floor() as i16,
                    ((coordinate[2] - tree.translate().z()) / tree.scale().z()).floor() as i16,
                ]
            })
            .collect();
        let voxel_grid: Vec<_> = (-2..=2)
            .flat_map(|i| (-2..=2).flat_map(move |j| (-2..=2).map(move |k| [i, j, k])))
            .collect();
        let (exterior_faces, exterior_nodes) = finite_elements.exterior_faces_and_nodes();
        let new_points: Coordinates = exterior_nodes
            .iter()
            .map(|&exterior_node| {
                let [i, j, k] = rounded_coordinates[exterior_node];
                let mut nearby_surface_nodes: Nodes = voxel_grid
                    .iter()
                    .filter_map(|[i0, j0, k0]| {
                        bins.get(&[(i + i0) as usize, (j + j0) as usize, (k + k0) as usize])
                    })
                    .flatten()
                    .copied()
                    .collect();
                assert!(!nearby_surface_nodes.is_empty());
                nearby_surface_nodes.sort();
                nearby_surface_nodes.dedup();
                let exterior_node_coordinates =
                    &finite_elements.get_nodal_coordinates()[exterior_node];
                let (closest_node, _) = nearby_surface_nodes.iter().fold(
                    (usize::MAX, f64::MAX),
                    |(closest_node, minimum_distance_squared), &surface_node| {
                        let distance_squared = (&surface_nodal_coordinates[surface_node]
                            - exterior_node_coordinates)
                            .norm_squared();
                        if distance_squared < minimum_distance_squared {
                            (surface_node, distance_squared)
                        } else {
                            (closest_node, minimum_distance_squared)
                        }
                    },
                );
                let (closest_point, _) =
                    surface_node_element_connectivity[closest_node].iter().fold(
                        (Coordinate::new([f64::MAX; NSD]), f64::MAX),
                        |(closest_point, minimum_distance_squared), &triangle| {
                            let point = TriangularFiniteElements::closest_point(
                                exterior_node_coordinates,
                                &surface_nodal_coordinates,
                                surface_element_node_connectivity[triangle],
                            );
                            let distance_squared =
                                (exterior_node_coordinates - &point).norm_squared();
                            if distance_squared < minimum_distance_squared {
                                (point, distance_squared)
                            } else {
                                (closest_point, minimum_distance_squared)
                            }
                        },
                    );
                closest_point
            })
            .collect();
        let numbering_offset = rounded_coordinates.len();
        let mut surface_nodes_map = vec![0; exterior_nodes.iter().max().unwrap() + 1];
        exterior_nodes
            .iter()
            .enumerate()
            .for_each(|(surface_node, &exterior_node)| {
                surface_nodes_map[exterior_node] = surface_node + numbering_offset
            });
        finite_elements.nodal_coordinates.extend(new_points);
        let new_hexes: Connectivity<HEX> = exterior_faces
            .into_iter()
            .map(|[node_0, node_1, node_2, node_3]| {
                [
                    node_0,
                    node_1,
                    node_2,
                    node_3,
                    surface_nodes_map[node_0],
                    surface_nodes_map[node_1],
                    surface_nodes_map[node_2],
                    surface_nodes_map[node_3],
                ]
            })
            .collect();
        finite_elements
            .element_blocks
            .extend(vec![finite_elements.element_blocks[0]; new_hexes.len()]);
        finite_elements.element_node_connectivity.extend(new_hexes);
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mConforming to surface\x1b[0m {:?}",
            time.elapsed()
        );
        Ok(finite_elements)
    }
}

fn mark_outside_nodes(coordinates: Coordinates, mut samples: Samples, tree: &Octree) -> Nodes {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut i;
    let mut j;
    let mut k;
    let mut index = 0;
    let mut indices = vec![[0, 0, 0]];
    let lim = (tree.nel().x() - 1) as u16;
    while index < indices.len() {
        [i, j, k] = indices[index];
        if i > 0 && !samples[(i - 1) as usize][j as usize][k as usize] {
            samples[(i - 1) as usize][j as usize][k as usize] = true;
            indices.push([i - 1, j, k]);
        }
        if i < lim && !samples[(i + 1) as usize][j as usize][k as usize] {
            samples[(i + 1) as usize][j as usize][k as usize] = true;
            indices.push([i + 1, j, k]);
        }
        if j > 0 && !samples[i as usize][(j - 1) as usize][k as usize] {
            samples[i as usize][(j - 1) as usize][k as usize] = true;
            indices.push([i, j - 1, k]);
        }
        if j < lim && !samples[i as usize][(j + 1) as usize][k as usize] {
            samples[i as usize][(j + 1) as usize][k as usize] = true;
            indices.push([i, j + 1, k]);
        }
        if k > 0 && !samples[i as usize][j as usize][(k - 1) as usize] {
            samples[i as usize][j as usize][(k - 1) as usize] = true;
            indices.push([i, j, k - 1]);
        }
        if k < lim && !samples[i as usize][j as usize][(k + 1) as usize] {
            samples[i as usize][j as usize][(k + 1) as usize] = true;
            indices.push([i, j, k + 1]);
        }
        index += 1
    }
    let removed_nodes = coordinates
        .into_iter()
        .enumerate()
        .filter_map(|(node, coordinate)| {
            if samples[coordinate[0].floor() as usize][coordinate[1].floor() as usize]
                [coordinate[2].floor() as usize]
            {
                Some(node)
            } else {
                None
            }
        })
        .collect();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mMarking outside nodes\x1b[0m {:?}",
        time.elapsed()
    );
    removed_nodes
}

impl TryFrom<(Tessellation, Size)> for TetrahedralFiniteElements {
    type Error = String;
    fn try_from((_tessellation, _size): (Tessellation, Size)) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

impl TryFrom<(Tessellation, Size)> for TriangularFiniteElements {
    type Error = String;
    fn try_from((_tessellation, _size): (Tessellation, Size)) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

/// Methods specific to each finite element type.
pub trait FiniteElementSpecifics<const M: usize> {
    /// Returns the nodes connected to the given node within an element.
    fn connected_nodes(node: &usize) -> Vec<usize>;
    /// Returns the exterior faces.
    fn exterior_faces(&self) -> Connectivity<M>;
    /// Calculates evenly-spaced points interior to each exterior face.
    fn exterior_faces_interior_points(&self, grid_length: usize) -> Coordinates;
    /// Returns the exterior faces and nodes.
    fn exterior_faces_and_nodes(&self) -> (Connectivity<M>, Nodes) {
        let faces = self.exterior_faces();
        let mut nodes: Nodes = faces.iter().flatten().copied().collect();
        nodes.sort();
        nodes.dedup();
        (faces, nodes)
    }
    /// Returns the faces.
    fn faces(&self) -> Connectivity<M>;
    /// Calculates evenly-spaced points interior to each element.
    fn interior_points(&self, grid_length: usize) -> Coordinates;
    /// Calculates the maximum edge ratios.
    fn maximum_edge_ratios(&self) -> Metrics;
    /// Calculates the maximum skews.
    fn maximum_skews(&self) -> Metrics;
    /// Calculates the minimum scaled Jacobians.
    fn minimum_scaled_jacobians(&self) -> Metrics;
    /// Isotropic remeshing of the finite elements.
    fn remesh(&mut self, iterations: usize, smoothing_method: &Smoothing, size: Size);
    /// Writes the finite elements quality metrics to a new file.
    fn write_metrics(&self, file_path: &str) -> Result<(), ErrorIO>;
}

fn reorder_connectivity<const N: usize>(
    element_blocks: &Blocks,
    element_blocks_unique: &Blocks,
    element_node_connectivity: &Connectivity<N>,
) -> ReorderedConnectivity {
    element_blocks_unique
        .par_iter()
        .map(|unique_block| {
            element_blocks
                .iter()
                .enumerate()
                .filter(|&(_, block)| block == unique_block)
                .flat_map(|(element, _)| {
                    element_node_connectivity[element]
                        .iter()
                        .map(|&entry| (entry + NODE_NUMBERING_OFFSET) as u32)
                })
                .collect()
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

fn finite_element_data_from_exo<const N: usize>(file_path: &str) -> Result<Data<N>, ErrorNetCDF> {
    let file = open(file_path)?;
    let mut blocks = vec![];
    let connectivity = file
        .variables()
        .filter(|variable| variable.name().starts_with("connect"))
        .flat_map(|variable| {
            let connect = variable
                .get_values::<u32, _>(..)
                .expect("Error getting block connectivity")
                .chunks(N)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|&node| node as usize - NODE_NUMBERING_OFFSET)
                        .collect::<Vec<usize>>()
                        .try_into()
                        .expect("Error getting element connectivity")
                })
                .collect::<Connectivity<N>>();
            blocks.extend(vec![
                variable.name()["connect".len()..]
                    .parse::<u8>()
                    .expect("Error getting block index");
                connect.len()
            ]);
            connect
        })
        .collect();
    let coordinates = file
        .variable("coordx")
        .expect("Coordinates x not found")
        .get_values(..)?
        .into_iter()
        .zip(
            file.variable("coordy")
                .expect("Coordinates y not found")
                .get_values(..)?
                .into_iter()
                .zip(
                    file.variable("coordz")
                        .expect("Coordinates z not found")
                        .get_values(..)?,
                ),
        )
        .map(|(x, (y, z))| [x, y, z].into())
        .collect();
    Ok((blocks, connectivity, coordinates))
}

fn finite_element_data_from_inp<const N: usize>(file_path: &str) -> Result<Data<N>, ErrorIO> {
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
                .parse::<usize>()
                .unwrap()
                - NODE_NUMBERING_OFFSET,
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
    let mut mapping = vec![0_usize; *inverse_mapping.iter().max().unwrap() + NODE_NUMBERING_OFFSET];
    inverse_mapping
        .iter()
        .enumerate()
        .for_each(|(new, &old)| mapping[old] = new);
    buffer.clear();
    file.read_line(&mut buffer)?;
    buffer.clear();
    file.read_line(&mut buffer)?;
    let mut current_block = 0;
    let mut element_blocks: Blocks = vec![];
    let mut element_node_connectivity: Connectivity<N> = vec![];
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
                    .map(|entry| entry.trim().parse::<usize>().unwrap() - NODE_NUMBERING_OFFSET)
                    .collect::<Vec<usize>>()
                    .try_into()
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
                .for_each(|node| *node = mapping[*node])
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
    file.add_attribute::<u32>("file_size", 1)?;
    file.add_attribute::<u32>("floating_point_word_size", 8)?;
    file.add_attribute::<String>("title", automesh_header())?;
    file.add_attribute::<f32>("version", 8.25)?;
    let mut element_blocks_unique = element_blocks.clone();
    element_blocks_unique.sort();
    element_blocks_unique.dedup();
    file.add_dimension("time_step", 0)?;
    file.add_dimension("num_dim", NSD)?;
    file.add_dimension("num_elem", element_blocks.len())?;
    file.add_dimension("num_el_blk", element_blocks_unique.len())?;
    let mut eb_prop1 = file.add_variable::<u32>("eb_prop1", &["num_el_blk"])?;
    element_blocks_unique
        .iter()
        .enumerate()
        .try_for_each(|(index, unique_block)| eb_prop1.put_value(*unique_block as u32, index))?;
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
                format!("num_el_in_blk{current_block}").as_str(),
                number_of_elements,
            )?;
            file.add_dimension(format!("num_nod_per_el{current_block}").as_str(), N)?;
            let mut connectivities = file.add_variable::<u32>(
                format!("connect{current_block}").as_str(),
                &[
                    format!("num_el_in_blk{current_block}").as_str(),
                    format!("num_nod_per_el{current_block}").as_str(),
                ],
            )?;
            match N {
                HEX => connectivities.put_attribute("elem_type", "HEX8")?,
                TET => connectivities.put_attribute("elem_type", "TET4")?,
                TRI => connectivities.put_attribute("elem_type", "TRI3")?,
                _ => panic!(),
            };
            connectivities.put_values(&block_connectivity, (.., ..))?;
            Ok::<_, ErrorNetCDF>(())
        })?;
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mElement-to-node connectivity\x1b[0m {:?}",
        time.elapsed()
    );
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let (xs, (ys, zs)): (Vec<f64>, (Vec<f64>, Vec<f64>)) = nodal_coordinates
        .iter()
        .map(|coords| (coords[0], (coords[1], coords[2])))
        .unzip();
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
    let postfix = "\n";
    let middle = automesh_header().replace(", ", "\n** ");
    let heading = format!("** {middle}{postfix}");
    file.write_all(heading.as_bytes())
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
                file.write_all(format!("{coordinate:>15.6e}").as_bytes())
            })
        })?;
    newline(file)?;
    end_section(file)?;
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mNodal coordinates\x1b[0m {:?}",
        time.elapsed()
    );
    Ok(())
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
        TET => "C3D4",
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
                format!("*ELEMENT, TYPE={element_type}, ELSET=EB{current_block}").as_bytes(),
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
                                format!(
                                    "{:>width$}",
                                    entry + NODE_NUMBERING_OFFSET,
                                    width = node_number_width + 3
                                )
                                .as_bytes(),
                            )
                        })
                })?;
            newline(file)
        })?;
    end_section(file)?;
    file.write_all(
        "********************************** P R O P E R T I E S ************************\n"
            .as_bytes(),
    )?;
    element_blocks_unique
        .into_iter()
        .try_for_each(|current_block| {
            file.write_all(
                format!("*SOLID SECTION, ELSET=EB{current_block}, MATERIAL=Default-Steel\n")
                    .as_bytes(),
            )
        })?;
    end_file(file)?;
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mElement-to-node connectivity\x1b[0m {:?}",
        time.elapsed()
    );
    Ok(())
}

fn end_section(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    file.write_all(&[42, 42, 10])
}

fn end_file(file: &mut BufWriter<File>) -> Result<(), ErrorIO> {
    file.write_all(&[42, 42])
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
            .try_for_each(|coordinate| file.write_all(format!("{coordinate} ").as_bytes()))?;
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
                .try_for_each(|node| file.write_all(format!("{node} ").as_bytes()))?;
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
        .map(|&node| node as u64)
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
