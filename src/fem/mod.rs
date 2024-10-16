#[cfg(feature = "python")]
pub mod py;

#[cfg(test)]
pub mod test;

#[cfg(feature = "profile")]
use std::time::Instant;

use super::{abaqus::Abaqus, ELEMENT_NUMBERING_OFFSET, NODE_NUMBERING_OFFSET, NSD};
use chrono::Utc;
use std::{
    fs::File,
    io::{BufWriter, Error, Write},
};

const ELEMENT_TYPE: &str = "C3D8R";
const ELEMENT_NUM_NODES: usize = 8;
const EMPTY_CONNECTIVITY: Connectivity = vec![];
const EMPTY_NODES: Nodes = vec![];

pub type Blocks = Vec<usize>;
pub type Connectivity = Vec<Vec<usize>>;
pub type Coordinates = Vec<Vec<f64>>;
pub type Nodes = Vec<usize>;

/// The finite elements type.
pub struct FiniteElements {
    boundary_nodes: Nodes,
    element_blocks: Blocks,
    element_node_connectivity: Connectivity,
    exterior_nodes: Nodes,
    interface_nodes: Nodes,
    interior_nodes: Nodes,
    nodal_coordinates: Coordinates,
    node_element_connectivity: Connectivity,
    node_node_connectivity: Connectivity,
    node_node_connectivity_boundary: Connectivity,
    node_node_connectivity_interior: Connectivity,
}

/// Possible smoothing methods.
pub enum Smoothing {
    Laplacian(usize, f64),
}

/// Inherent implementation of the finite elements type.
impl FiniteElements {
    /// Constructs and returns a new finite elements type from data.
    pub fn from_data(
        element_blocks: Blocks,
        element_node_connectivity: Connectivity,
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
            node_element_connectivity: vec![],
            node_node_connectivity: vec![],
            node_node_connectivity_boundary: vec![],
            node_node_connectivity_interior: vec![],
        }
    }
    /// Calculates the discrete Laplacian for the given node-to-node connectivity.
    pub fn calculate_laplacian(&self, node_node_connectivity: &Connectivity) -> Coordinates {
        let nodal_coordinates = self.get_nodal_coordinates();
        node_node_connectivity
            .iter()
            .enumerate()
            .map(|(node, connectivity)| {
                (0..NSD)
                    .map(|i| {
                        connectivity
                            .iter()
                            .map(|neighbor| nodal_coordinates[neighbor - NODE_NUMBERING_OFFSET][i])
                            .sum::<f64>()
                            / (connectivity.len() as f64)
                            - nodal_coordinates[node][i]
                    })
                    .collect()
            })
            .collect()
    }
    /// Calculates and sets the nodal hierarchy.
    pub fn calculate_nodal_hierarchy(&mut self) -> Result<(), &str> {
        let node_element_connectivity = self.get_node_element_connectivity();
        if node_element_connectivity != &EMPTY_CONNECTIVITY {
            #[cfg(feature = "profile")]
            let time = Instant::now();
            let element_blocks = self.get_element_blocks();
            let mut connected_blocks: Vec<usize> = vec![];
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
                        if number_of_connected_elements < 8 {
                            exterior_nodes.push(node + NODE_NUMBERING_OFFSET);
                        }
                    } else if number_of_connected_elements < 8 {
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
            Err("Need to calculate and set the node-to-element connectivity first.")
        }
    }
    /// Calculates and sets the node-to-element connectivity.
    pub fn calculate_node_element_connectivity(&mut self) -> Result<(), &str> {
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
    /// Calculates and sets the node-to-node connectivity.
    pub fn calculate_node_node_connectivity(&mut self) -> Result<(), &str> {
        let node_element_connectivity = self.get_node_element_connectivity();
        if node_element_connectivity != &EMPTY_CONNECTIVITY {
            #[cfg(feature = "profile")]
            let time = Instant::now();
            let mut element_connectivity = vec![0; ELEMENT_NUM_NODES];
            let element_node_connectivity = self.get_element_node_connectivity();
            let number_of_nodes = self.get_nodal_coordinates().len();
            let mut node_node_connectivity = vec![vec![]; number_of_nodes];
            node_node_connectivity
                .iter_mut()
                .zip(node_element_connectivity.iter().enumerate())
                .try_for_each(|(connectivity, (node, node_connectivity))| {
                    node_connectivity.iter().try_for_each(|element| {
                        element_connectivity.clone_from(
                            &element_node_connectivity[element - ELEMENT_NUMBERING_OFFSET],
                        );
                        match element_connectivity
                            .iter()
                            .position(|&n| n == node + NODE_NUMBERING_OFFSET)
                        {
                            Some(0) => {
                                connectivity.push(element_connectivity[1]);
                                connectivity.push(element_connectivity[3]);
                                connectivity.push(element_connectivity[4]);
                                Ok(())
                            }
                            Some(1) => {
                                connectivity.push(element_connectivity[0]);
                                connectivity.push(element_connectivity[2]);
                                connectivity.push(element_connectivity[5]);
                                Ok(())
                            }
                            Some(2) => {
                                connectivity.push(element_connectivity[1]);
                                connectivity.push(element_connectivity[3]);
                                connectivity.push(element_connectivity[6]);
                                Ok(())
                            }
                            Some(3) => {
                                connectivity.push(element_connectivity[0]);
                                connectivity.push(element_connectivity[2]);
                                connectivity.push(element_connectivity[7]);
                                Ok(())
                            }
                            Some(4) => {
                                connectivity.push(element_connectivity[0]);
                                connectivity.push(element_connectivity[5]);
                                connectivity.push(element_connectivity[7]);
                                Ok(())
                            }
                            Some(5) => {
                                connectivity.push(element_connectivity[1]);
                                connectivity.push(element_connectivity[4]);
                                connectivity.push(element_connectivity[6]);
                                Ok(())
                            }
                            Some(6) => {
                                connectivity.push(element_connectivity[2]);
                                connectivity.push(element_connectivity[5]);
                                connectivity.push(element_connectivity[7]);
                                Ok(())
                            }
                            Some(7) => {
                                connectivity.push(element_connectivity[3]);
                                connectivity.push(element_connectivity[4]);
                                connectivity.push(element_connectivity[6]);
                                Ok(())
                            }
                            Some(8..) => Err(
                                "The element-to-node connectivity has been incorrectly calculated.",
                            ),
                            None => Err(
                                "The node-to-element connectivity has been incorrectly calculated.",
                            ),
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
            Err("Need to calculate and set the node-to-element connectivity first.")
        }
    }
    /// Calculates and sets the node-to-node connectivity for boundary nodes.
    pub fn calculate_node_node_connectivity_boundary(&mut self) -> Result<(), &str> {
        let exterior_nodes = self.get_exterior_nodes();
        if exterior_nodes != &EMPTY_NODES {
            let boundary_nodes = self.get_boundary_nodes();
            let node_node_connectivity = self.get_node_node_connectivity();
            self.node_node_connectivity_boundary = boundary_nodes
                .iter()
                .map(|boundary_node| {
                    node_node_connectivity[boundary_node - NODE_NUMBERING_OFFSET]
                        .clone()
                        .into_iter()
                        .filter(|&node| boundary_nodes.contains(&node))
                        .collect()
                })
                .collect();
            Ok(())
        } else {
            Err("Need to calculate and set the nodal hierarchy first.")
        }
    }
    /// Calculates and sets the node-to-node connectivity for interior nodes.
    pub fn calculate_node_node_connectivity_interior(&mut self) -> Result<(), &str> {
        if self.get_exterior_nodes() != &EMPTY_NODES {
            let node_node_connectivity = self.get_node_node_connectivity();
            self.node_node_connectivity_interior = self
                .get_interior_nodes()
                .iter()
                .map(|interior_node| {
                    node_node_connectivity[interior_node - NODE_NUMBERING_OFFSET].clone()
                })
                .collect();
            Ok(())
        } else {
            Err("Need to calculate and set the nodal hierarchy first.")
        }
    }
    /// Returns a reference to the boundary nodes.
    pub fn get_boundary_nodes(&self) -> &Nodes {
        &self.boundary_nodes
    }
    /// Returns a reference to the element blocks.
    pub fn get_element_blocks(&self) -> &Blocks {
        &self.element_blocks
    }
    /// Returns a reference to the element-to-node connectivity.
    pub fn get_element_node_connectivity(&self) -> &Connectivity {
        &self.element_node_connectivity
    }
    /// Returns a reference to the exterior nodes.
    pub fn get_exterior_nodes(&self) -> &Nodes {
        &self.exterior_nodes
    }
    /// Returns a reference to the interface nodes.
    pub fn get_interface_nodes(&self) -> &Nodes {
        &self.interface_nodes
    }
    /// Returns a reference to the interior nodes.
    pub fn get_interior_nodes(&self) -> &Nodes {
        &self.interior_nodes
    }
    /// Returns a reference to the nodal coordinates.
    pub fn get_nodal_coordinates(&self) -> &Coordinates {
        &self.nodal_coordinates
    }
    /// Returns a mutable reference to the nodal coordinates.
    pub fn get_nodal_coordinates_mut(&mut self) -> &mut Coordinates {
        &mut self.nodal_coordinates
    }
    /// Returns a reference to the node-to-element connectivity.
    pub fn get_node_element_connectivity(&self) -> &Connectivity {
        &self.node_element_connectivity
    }
    /// Returns a reference to the node-to-node connectivity.
    pub fn get_node_node_connectivity(&self) -> &Connectivity {
        &self.node_node_connectivity
    }
    /// Returns a reference to the node-to-node connectivity for boundary nodes.
    pub fn get_node_node_connectivity_boundary(&self) -> &Connectivity {
        &self.node_node_connectivity_boundary
    }
    /// Returns a reference to the node-to-node connectivity for interior nodes.
    pub fn get_node_node_connectivity_interior(&self) -> &Connectivity {
        &self.node_node_connectivity_interior
    }
    /// ???
    pub fn smooth(&mut self, method: Smoothing) {
        match method {
            Smoothing::Laplacian(iterations, scale) => {
                for _ in 0..iterations {
                    let laplacian = self.calculate_laplacian(self.get_node_node_connectivity());
                    self.get_nodal_coordinates_mut()
                        .iter_mut()
                        .flatten()
                        .zip(laplacian.iter().flatten())
                        .for_each(|(coordinate, entry)| *coordinate += scale * entry)
                }
            }
        }
    }
}

/// Abaqus implementation of the finite elements type.
impl Abaqus for FiniteElements {
    fn write_inp(&self, file_path: &str) -> Result<(), Error> {
        write_fem_to_inp(
            file_path,
            self.get_element_blocks(),
            self.get_element_node_connectivity(),
            self.get_nodal_coordinates(),
        )
    }
}

fn write_fem_to_inp(
    file_path: &str,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity,
    nodal_coordinates: &Coordinates,
) -> Result<(), Error> {
    let element_number_width = get_width(element_node_connectivity);
    let node_number_width = get_width(nodal_coordinates);
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

fn write_heading_to_inp(file: &mut BufWriter<File>) -> Result<(), Error> {
    let heading = format!(
        "*HEADING\nautotwin.automesh\nversion {}\nautogenerated on {}",
        env!("CARGO_PKG_VERSION"),
        Utc::now()
    );
    file.write_all(heading.as_bytes())?;
    end_section(file)
}

fn write_nodal_coordinates_to_inp(
    file: &mut BufWriter<File>,
    nodal_coordinates: &Coordinates,
    node_number_width: &usize,
) -> Result<(), Error> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
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
    let result = end_section(file);
    #[cfg(feature = "profile")]
    println!(
        "           \x1b[1;93m⤷ Nodal coordinates\x1b[0m {:?}",
        time.elapsed()
    );
    result
}

fn write_element_node_connectivity_to_inp(
    file: &mut BufWriter<File>,
    element_blocks: &Blocks,
    element_node_connectivity: &Connectivity,
    element_number_width: &usize,
    node_number_width: &usize,
) -> Result<(), Error> {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let mut unique_element_blocks = element_blocks.clone();
    unique_element_blocks.sort();
    unique_element_blocks.dedup();
    unique_element_blocks
        .iter()
        .clone()
        .try_for_each(|current_block| {
            file.write_all(
                format!("*ELEMENT, TYPE={}, ELSET=EB{}", ELEMENT_TYPE, current_block).as_bytes(),
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
            end_section(file)
        })?;
    let result = unique_element_blocks.iter().try_for_each(|block| {
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

fn end_section(file: &mut BufWriter<File>) -> Result<(), Error> {
    file.write_all(&[10, 42, 42, 10])
}

fn delimiter(file: &mut BufWriter<File>) -> Result<(), Error> {
    file.write_all(&[44, 32])
}

fn indent(file: &mut BufWriter<File>) -> Result<(), Error> {
    file.write_all(&[10, 32, 32, 32, 32])
}

fn get_width<T>(input: &[T]) -> usize {
    input.len().to_string().chars().count()
}
