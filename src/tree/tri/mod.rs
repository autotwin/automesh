#[cfg(feature = "profile")]
use std::time::Instant;

use crate::{
    Blocks, Coordinate, Coordinates,
    fem::TriangularFiniteElements,
    tree::{Edges, Faces, NUM_FACES, NUM_NODES_FACE, Octree, PADDING, mirror_face},
};
use conspire::math::{Tensor, TensorArray, TensorVec};

impl From<Octree> for TriangularFiniteElements {
    fn from(mut tree: Octree) -> Self {
        let mut removed_data: Blocks = (&tree.remove).into();
        removed_data.push(PADDING);
        tree.boundaries();
        let clusters = tree.clusters(Some(&removed_data), None);
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let blocks = clusters
            .iter()
            .map(|cluster: &Vec<usize>| tree[cluster[0]].get_block())
            .collect::<Blocks>();
        let default_face_info = [None; NUM_FACES];
        let mut faces_info = default_face_info;
        let boundaries_cells_faces = blocks
            .iter()
            .zip(clusters.iter())
            .map(|(&block, cluster)| {
                cluster
                    .iter()
                    .filter(|&&cell| tree[cell].is_voxel())
                    .filter_map(|&cell| {
                        faces_info = default_face_info;
                        faces_info
                            .iter_mut()
                            .enumerate()
                            .zip(tree[cell].get_faces().iter())
                            .for_each(|((face_index, face_info), &face)| {
                                if let Some(face_cell) = face {
                                    if tree[face_cell].get_block() != block {
                                        *face_info = Some(face_cell)
                                    }
                                } else if tree[cell]
                                    .is_face_on_octree_boundary(&face_index, tree.nel())
                                {
                                    *face_info = Some(usize::MAX)
                                }
                            });
                        if faces_info.iter().all(|face_info| face_info.is_none()) {
                            None
                        } else {
                            Some((cell, faces_info))
                        }
                    })
                    .collect()
            })
            .collect::<Vec<Vec<(usize, Faces)>>>();
        let mut max_cell_id = 0;
        let mut boundaries_face_from_cell = boundaries_cells_faces
            .iter()
            .map(|boundary_cells_faces| {
                (max_cell_id, _) = *boundary_cells_faces
                    .iter()
                    .max_by(|(cell_a, _), (cell_b, _)| cell_a.cmp(cell_b))
                    .unwrap();
                vec![[false; NUM_FACES]; max_cell_id + 1]
            })
            .collect::<Vec<Vec<[bool; NUM_FACES]>>>();
        max_cell_id = 0;
        boundaries_cells_faces
            .iter()
            .for_each(|boundary_cells_faces| {
                boundary_cells_faces.iter().for_each(|(cell, _)| {
                    if cell > &max_cell_id {
                        max_cell_id = *cell
                    }
                })
            });
        let mut boundary_from_cell = vec![None; max_cell_id + 1];
        boundaries_cells_faces
            .iter()
            .enumerate()
            .for_each(|(boundary, boundary_cells_faces)| {
                boundary_cells_faces
                    .iter()
                    .for_each(|(cell, _)| boundary_from_cell[*cell] = Some(boundary))
            });
        let mut cell_face_index = 0;
        let mut cell_faces = vec![vec![]; max_cell_id + 1];
        let mut face_blocks: Vec<u8> = vec![];
        let mut face_cells: Vec<usize> = vec![];
        let mut face_connectivity = [0; NUM_NODES_FACE];
        let mut faces_connectivity = vec![];
        let mut nodal_coordinates = Coordinates::zero(0);
        let mut node_new = 0;
        let nodes_len = (tree[0].get_lngth() + 1) as usize;
        let mut nodes = vec![vec![vec![None; nodes_len]; nodes_len]; nodes_len];
        (0..boundaries_cells_faces.len()).for_each(|boundary| {
            boundaries_cells_faces[boundary]
                .iter()
                .for_each(|(cell, faces)| {
                    faces.iter().enumerate().for_each(|(face_index, face)| {
                        if let Some(face_cell) = face
                            && !boundaries_face_from_cell[boundary][*cell][face_index]
                        {
                            boundaries_face_from_cell[boundary][*cell][face_index] = true;
                            #[allow(clippy::collapsible_if)]
                            if face_cell != &usize::MAX {
                                if removed_data
                                    .binary_search(&tree[*face_cell].get_block())
                                    .is_err()
                                {
                                    if let Some(opposing_boundary) = boundary_from_cell[*face_cell]
                                    {
                                        boundaries_face_from_cell[opposing_boundary][*face_cell]
                                            [mirror_face(face_index)] = true;
                                    }
                                }
                            }
                            tree[*cell]
                                .get_nodal_indices_face(&face_index)
                                .iter()
                                .zip(face_connectivity.iter_mut())
                                .for_each(|(nodal_indices, face_node)| {
                                    if let Some(node) = nodes[nodal_indices[0] as usize]
                                        [nodal_indices[1] as usize]
                                        [nodal_indices[2] as usize]
                                    {
                                        *face_node = node
                                    } else {
                                        nodal_coordinates.push(Coordinate::new([
                                            nodal_indices[0] as f64 * tree.scale.x()
                                                + tree.translate.x(),
                                            nodal_indices[1] as f64 * tree.scale.y()
                                                + tree.translate.y(),
                                            nodal_indices[2] as f64 * tree.scale.z()
                                                + tree.translate.z(),
                                        ]));
                                        *face_node = node_new;
                                        nodes[nodal_indices[0] as usize]
                                            [nodal_indices[1] as usize]
                                            [nodal_indices[2] as usize] = Some(node_new);
                                        node_new += 1;
                                    }
                                });
                            cell_faces[*cell].push(cell_face_index);
                            cell_face_index += 1;
                            face_blocks.push(boundary as u8 + 1);
                            face_cells.push(*cell);
                            faces_connectivity.push(face_connectivity)
                        }
                    })
                })
        });
        let node_face_connectivity =
            invert_connectivity(&faces_connectivity, nodal_coordinates.len());
        let mut non_manifold_edges = non_manifold(&faces_connectivity, &node_face_connectivity);
        let consecutive_non_manifold_edges: Edges = non_manifold_edges
            .iter()
            .filter(|non_manifold_edge_a| {
                non_manifold_edges.iter().any(|non_manifold_edge_b| {
                    non_manifold_edge_a
                        .iter()
                        .filter(|node_a| non_manifold_edge_b.contains(node_a))
                        .count()
                        == 1
                })
            })
            .copied()
            .collect();
        non_manifold_edges.retain(|non_manifold_edge| {
            consecutive_non_manifold_edges
                .binary_search(non_manifold_edge)
                .is_err()
        });
        let mut consecutive_non_manifold_nodes: Vec<usize> = consecutive_non_manifold_edges
            .into_iter()
            .flatten()
            .collect();
        consecutive_non_manifold_nodes.sort();
        consecutive_non_manifold_nodes.dedup();
        let mut non_manifold_node_position = 0;
        consecutive_non_manifold_nodes
            .into_iter()
            .for_each(|non_manifold_node| {
                node_face_connectivity[non_manifold_node]
                    .iter()
                    .for_each(|&non_manifold_face| {
                        non_manifold_node_position = faces_connectivity[non_manifold_face]
                            .iter()
                            .position(|node| node == &non_manifold_node)
                            .expect("Position of node not found");
                        faces_connectivity[non_manifold_face][non_manifold_node_position] =
                            nodal_coordinates.len();
                        nodal_coordinates.push(nodal_coordinates[non_manifold_node].clone());
                    })
            });
        non_manifold_edges.iter().for_each(|edge| {
            let non_manifold_faces: Vec<usize> = node_face_connectivity[edge[0]]
                .iter()
                .filter(|face_a| node_face_connectivity[edge[1]].contains(face_a))
                .copied()
                .collect();
            let mut non_manifold_cells_vec: Vec<usize> = non_manifold_faces
                .iter()
                .map(|&non_manifold_face| face_cells[non_manifold_face])
                .collect();
            non_manifold_cells_vec.sort();
            non_manifold_cells_vec.dedup();
            let non_manifold_cells: [usize; 2] = non_manifold_cells_vec
                .try_into()
                .expect("There should be two non-manifold cells.");
            let non_manifold_cells_non_manifold_faces: [[usize; 2]; 2] = non_manifold_cells
                .iter()
                .map(|&non_manifold_cell| {
                    cell_faces[non_manifold_cell]
                        .iter()
                        .filter(|cell_face| non_manifold_faces.contains(cell_face))
                        .copied()
                        .collect::<Vec<usize>>()
                        .try_into()
                        .expect("There should be two non-manifold faces per non-manifold cell.")
                })
                .collect::<Vec<[usize; 2]>>()
                .try_into()
                .expect("There should be two non-manifold cells.");

            let non_manifold_cells_other_faces: Vec<Vec<usize>> = non_manifold_cells
                .iter()
                .map(|&non_manifold_cell| {
                    cell_faces[non_manifold_cell]
                        .iter()
                        .filter(|cell_face| !non_manifold_faces.contains(cell_face))
                        .copied()
                        .collect()
                })
                .collect();
            let non_manifold_cells_bowtie_faces: Vec<Vec<usize>> =
                non_manifold_cells_non_manifold_faces
                    .iter()
                    .zip(non_manifold_cells_other_faces.iter())
                    .map(|(non_manifold_faces, other_faces)| {
                        other_faces
                            .iter()
                            .filter(|&&other_face| {
                                faces_connectivity[other_face].iter().any(|node| {
                                    non_manifold_faces.iter().all(|&non_manifold_face| {
                                        faces_connectivity[non_manifold_face].contains(node)
                                    })
                                })
                            })
                            .copied()
                            .collect()
                    })
                    .collect();
            let cells_num_bowtie_faces: [usize; 2] = non_manifold_cells_bowtie_faces
                .iter()
                .map(|non_manifold_cell_bowtie_faces| non_manifold_cell_bowtie_faces.len())
                .collect::<Vec<usize>>()
                .try_into()
                .expect("There should be two non-manifold cells.");
            let cell_index = match cells_num_bowtie_faces {
                [0, 0] => unimplemented!("Change below [0] once implemented."),
                [1, 0] | [1, 1] => 0,
                [0, 1] => 1,
                [2, 2] => unimplemented!("Change below [0] once implemented."),
                _ => panic!(),
            };
            let node = edge
                .iter()
                .find(|node| {
                    faces_connectivity[non_manifold_cells_bowtie_faces[cell_index][0]]
                        .contains(node)
                })
                .unwrap();
            let faces: [usize; 3] = node_face_connectivity[*node]
                .iter()
                .filter(|face| cell_faces[non_manifold_cells[cell_index]].contains(face))
                .copied()
                .collect::<Vec<usize>>()
                .try_into()
                .expect("Should be 3 faces.");
            nodal_coordinates.push(nodal_coordinates[*node].clone());
            let node_new = nodal_coordinates.len() - 1;
            let mut position = 0;
            faces.iter().for_each(|&face| {
                position = faces_connectivity[face]
                    .iter()
                    .position(|face_node| face_node == node)
                    .unwrap();
                faces_connectivity[face][position] = node_new
            });
        });
        let node_face_connectivity =
            invert_connectivity(&faces_connectivity, nodal_coordinates.len());
        let mut faces = [[0; 3]; 2];
        let mut faces_temp;
        for node_index in 0..nodal_coordinates.len() {
            if node_face_connectivity[node_index].len() == 6 {
                faces[0][0] = node_face_connectivity[node_index][0];
                if let Ok(trial_faces) = <[usize; 2]>::try_from(
                    node_face_connectivity[node_index]
                        .iter()
                        .skip(1)
                        .filter(|&&face| {
                            faces_connectivity[faces[0][0]]
                                .iter()
                                .filter(|node| faces_connectivity[face].contains(node))
                                .count()
                                == 2
                        })
                        .copied()
                        .collect::<Vec<usize>>(),
                ) {
                    faces[0][1] = trial_faces[0];
                    faces[0][2] = trial_faces[1];
                    faces_temp = node_face_connectivity[node_index].clone();
                    faces_temp.retain(|face| !faces[0].contains(face));
                    if let Ok(trial_faces_2) = <[usize; 3]>::try_from(faces_temp.clone()) {
                        faces[1] = trial_faces_2;
                        if faces[0].iter().all(|&face_a| {
                            faces[1].iter().all(|&face_b| {
                                faces_connectivity[face_a]
                                    .iter()
                                    .filter(|node_a| faces_connectivity[face_b].contains(node_a))
                                    .count()
                                    == 1
                            })
                        }) {
                            nodal_coordinates.push(nodal_coordinates[node_index].clone());
                            let node = node_index;
                            let node_new = nodal_coordinates.len() - 1;
                            let mut position = 0;
                            faces[0].iter().for_each(|&face| {
                                position = faces_connectivity[face]
                                    .iter()
                                    .position(|face_node| face_node == &node)
                                    .unwrap();
                                faces_connectivity[face][position] = node_new
                            });
                        }
                    }
                }
            }
        }
        let mut element_blocks = vec![0; 2 * face_blocks.len()];
        let mut element_node_connectivity = vec![[0; 3]; 2 * faces_connectivity.len()];
        let mut face = 0;
        let mut triangle = 0;
        faces_connectivity.iter().for_each(|face_connectivity| {
            element_blocks[triangle] = face_blocks[face];
            element_blocks[triangle + 1] = face_blocks[face];
            element_node_connectivity[triangle] = [
                face_connectivity[0],
                face_connectivity[1],
                face_connectivity[3],
            ];
            element_node_connectivity[triangle + 1] = [
                face_connectivity[1],
                face_connectivity[2],
                face_connectivity[3],
            ];
            face += 1;
            triangle += 2;
        });
        #[cfg(feature = "profile")]
        println!(
            "             \x1b[1;93mTriangular finite elements\x1b[0m {:?} ",
            time.elapsed()
        );
        Self::from((element_blocks, element_node_connectivity, nodal_coordinates))
    }
}

fn invert_connectivity(faces_connectivity: &[[usize; 4]], num_nodes: usize) -> Vec<Vec<usize>> {
    let mut node_face_connectivity = vec![vec![]; num_nodes];
    faces_connectivity
        .iter()
        .enumerate()
        .for_each(|(face, connectivity)| {
            connectivity
                .iter()
                .for_each(|&node| node_face_connectivity[node].push(face))
        });
    node_face_connectivity
}

fn edges(faces_connectivity: &[[usize; 4]]) -> Edges {
    let mut edges: Edges = faces_connectivity
        .iter()
        .flat_map(|&[node_0, node_1, node_2, node_3]| {
            [
                [node_0, node_1],
                [node_1, node_2],
                [node_2, node_3],
                [node_3, node_0],
            ]
            .into_iter()
        })
        .collect();
    edges.iter_mut().for_each(|edge| edge.sort());
    edges.sort();
    edges.dedup();
    edges
}

fn non_manifold(faces_connectivity: &[[usize; 4]], node_face_connectivity: &[Vec<usize>]) -> Edges {
    edges(faces_connectivity)
        .iter()
        .flat_map(|&edge| {
            if node_face_connectivity[edge[0]]
                .iter()
                .filter(|face_a| node_face_connectivity[edge[1]].contains(face_a))
                .count()
                == 4
            {
                Some(edge)
            } else {
                None
            }
        })
        .collect()
}
