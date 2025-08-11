use super::super::{HexConnectivity, NUM_FACES, NUM_OCTANTS, Octree};

pub fn apply(cells_nodes: &[usize], tree: &Octree) -> HexConnectivity {
    let mut element_node_connectivity = vec![];
    let mut connected_faces = [None; NUM_FACES];
    let mut d_01_subcells = None;
    let mut d_04_subcells = None;
    let mut d_14_subcells = None;
    let mut d014_subcells = None;
    let mut fa_0_subcells = [0; NUM_OCTANTS];
    let mut fa_1_subcells = [0; NUM_OCTANTS];
    let mut fa_4_subcells = [0; NUM_OCTANTS];
    let mut face_0_faces = &[None; NUM_FACES];
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .for_each(|(cell_subcells, cell_faces)| {
            connected_faces = [None; NUM_FACES];
            d_01_subcells = None;
            d_04_subcells = None;
            d_14_subcells = None;
            d014_subcells = None;
            cell_faces
                .iter()
                .enumerate()
                .for_each(|(face_index, face_cell)| {
                    if let Some(face_cell_index) = face_cell
                        && let Some(face_subcells) = tree[*face_cell_index].get_cells()
                        && tree.just_leaves(face_subcells)
                    {
                        match face_index {
                            0 => {
                                element_node_connectivity.push([
                                    cells_nodes[face_subcells[2]],
                                    cells_nodes[face_subcells[3]],
                                    cells_nodes[cell_subcells[1]],
                                    cells_nodes[cell_subcells[0]],
                                    cells_nodes[face_subcells[6]],
                                    cells_nodes[face_subcells[7]],
                                    cells_nodes[cell_subcells[5]],
                                    cells_nodes[cell_subcells[4]],
                                ]);
                                connected_faces[0] = Some(face_cell_index)
                            }
                            1 => {
                                element_node_connectivity.push([
                                    cells_nodes[cell_subcells[1]],
                                    cells_nodes[face_subcells[0]],
                                    cells_nodes[face_subcells[2]],
                                    cells_nodes[cell_subcells[3]],
                                    cells_nodes[cell_subcells[5]],
                                    cells_nodes[face_subcells[4]],
                                    cells_nodes[face_subcells[6]],
                                    cells_nodes[cell_subcells[7]],
                                ]);
                                connected_faces[1] = Some(face_cell_index)
                            }
                            4 => {
                                element_node_connectivity.push([
                                    cells_nodes[face_subcells[4]],
                                    cells_nodes[face_subcells[5]],
                                    cells_nodes[face_subcells[7]],
                                    cells_nodes[face_subcells[6]],
                                    cells_nodes[cell_subcells[0]],
                                    cells_nodes[cell_subcells[1]],
                                    cells_nodes[cell_subcells[3]],
                                    cells_nodes[cell_subcells[2]],
                                ]);
                                connected_faces[4] = Some(face_cell_index)
                            }
                            2 | 3 | 5 => {}
                            _ => panic!(),
                        }
                    }
                });
            if let Some(face_4) = connected_faces[4] {
                fa_4_subcells = tree[*face_4].get_cells().unwrap();
            }
            if let Some(face_1) = connected_faces[1] {
                fa_1_subcells = tree[*face_1].get_cells().unwrap();
                if connected_faces[4].is_some()
                    && let Some(diag_subcells) =
                        tree[tree[*face_1].get_faces()[4].unwrap()].get_cells()
                    && tree.just_leaves(diag_subcells)
                {
                    d_14_subcells = Some(diag_subcells);
                }
            }
            if let Some(face_0) = connected_faces[0] {
                fa_0_subcells = tree[*face_0].get_cells().unwrap();
                face_0_faces = tree[*face_0].get_faces();
                if connected_faces[1].is_some()
                    && let Some(diag_subcells) = tree[face_0_faces[1].unwrap()].get_cells()
                    && tree.just_leaves(diag_subcells)
                {
                    d_01_subcells = Some(diag_subcells);
                }
                if connected_faces[4].is_some()
                    && let Some(diag_subcells) = tree[face_0_faces[4].unwrap()].get_cells()
                    && tree.just_leaves(diag_subcells)
                {
                    d_04_subcells = Some(diag_subcells);
                    if d_01_subcells.is_some()
                        && d_01_subcells.is_some()
                        && let Some(diag_subcells_also) =
                            tree[tree[face_0_faces[1].unwrap()].get_faces()[4].unwrap()].get_cells()
                        && tree.just_leaves(diag_subcells_also)
                    {
                        d014_subcells = Some(diag_subcells_also)
                    }
                }
            }
            if let Some(diag_subcells) = d_01_subcells {
                element_node_connectivity.push([
                    cells_nodes[fa_0_subcells[3]],
                    cells_nodes[diag_subcells[2]],
                    cells_nodes[fa_1_subcells[0]],
                    cells_nodes[cell_subcells[1]],
                    cells_nodes[fa_0_subcells[7]],
                    cells_nodes[diag_subcells[6]],
                    cells_nodes[fa_1_subcells[4]],
                    cells_nodes[cell_subcells[5]],
                ]);
            }
            if let Some(diag_subcells) = d_04_subcells {
                element_node_connectivity.push([
                    cells_nodes[diag_subcells[6]],
                    cells_nodes[diag_subcells[7]],
                    cells_nodes[fa_4_subcells[5]],
                    cells_nodes[fa_4_subcells[4]],
                    cells_nodes[fa_0_subcells[2]],
                    cells_nodes[fa_0_subcells[3]],
                    cells_nodes[cell_subcells[1]],
                    cells_nodes[cell_subcells[0]],
                ]);
            }
            if let Some(d_14_subcells) = d_14_subcells {
                element_node_connectivity.push([
                    cells_nodes[fa_4_subcells[5]],
                    cells_nodes[d_14_subcells[4]],
                    cells_nodes[d_14_subcells[6]],
                    cells_nodes[fa_4_subcells[7]],
                    cells_nodes[cell_subcells[1]],
                    cells_nodes[fa_1_subcells[0]],
                    cells_nodes[fa_1_subcells[2]],
                    cells_nodes[cell_subcells[3]],
                ]);
                if let Some(diag_subcells) = d014_subcells {
                    element_node_connectivity.push([
                        cells_nodes[d_04_subcells.unwrap()[7]],
                        cells_nodes[diag_subcells[6]],
                        cells_nodes[d_14_subcells[4]],
                        cells_nodes[fa_4_subcells[5]],
                        cells_nodes[fa_0_subcells[3]],
                        cells_nodes[d_01_subcells.unwrap()[2]],
                        cells_nodes[fa_1_subcells[0]],
                        cells_nodes[cell_subcells[1]],
                    ]);
                }
            }
        });
    element_node_connectivity
}
