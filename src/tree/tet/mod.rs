use super::{
    super::fem::{
        Connectivity,
        tet::{NUM_TETS_PER_HEX, TET, TetrahedralFiniteElements, TetrahedralTransition},
    },
    Blocks, Cell, NODE_NUMBERING_OFFSET, Neighbor, Octree, Tree,
};
use ndarray::parallel::prelude::*;

pub fn connectivity(
    tree: &Octree,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
    removed_data: &Blocks,
) -> Connectivity<TET> {
    #[cfg(feature = "profile")]
    let temporary = std::time::Instant::now();
    let element_node_connectivity = tree
        .par_iter()
        .filter(|cell| cell.is_leaf() && removed_data.binary_search(&cell.get_block()).is_err())
        .flat_map(|leaf| match tree.neighbors_template(leaf) {
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
            ] => TetrahedralFiniteElements::hex_to_tet(
                &leaf
                    .get_nodal_indices_cell()
                    .into_iter()
                    .filter_map(|[i, j, k]| indexed_nodes[i][j][k])
                    .collect::<Vec<usize>>()
                    .try_into()
                    .unwrap(),
            )
            .to_vec(),
            [
                Neighbor::Face,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_f00000(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::Face,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_0f0000(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::Face,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_00f000(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::Face,
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_000f00(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::Face,
                Neighbor::None,
            ] => connectivity_0000f0(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::Face,
            ] => connectivity_00000f(leaf, indexed_nodes),
            [
                Neighbor::Edges(_),
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_ee0000(leaf, indexed_nodes),
            [
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_e00e00(leaf, indexed_nodes),
            [
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::None,
            ] => connectivity_e000e0(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_0ee000(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
            ] => connectivity_0e000e(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
            ] => connectivity_00ee00(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
            ] => connectivity_00e00e(leaf, indexed_nodes),
            [
                Neighbor::None,
                Neighbor::None,
                Neighbor::None,
                Neighbor::Edges(_),
                Neighbor::Edges(_),
                Neighbor::None,
            ] => connectivity_000ee0(leaf, indexed_nodes),
            _ => {
                vec![] // panic here to indicate a missing template
            }
        })
        .collect();
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;91m✰ Connectivity\x1b[0m {:?} ",
        temporary.elapsed()
    );
    element_node_connectivity
}

fn connectivity_f00000(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [
        min_x,
        haf_x,
        max_x,
        min_y,
        haf_y,
        max_y,
        min_z,
        haf_z,
        max_z,
    ] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[haf_x][min_y][max_z].unwrap(),
        indexed_nodes[haf_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][haf_z].unwrap(),
        indexed_nodes[max_x][min_y][haf_z].unwrap(),
        indexed_nodes[haf_x][min_y][haf_z].unwrap(),
        indexed_nodes[haf_x][haf_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_face(nodes)
}

fn connectivity_0f0000(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [
        min_x,
        haf_x,
        max_x,
        min_y,
        haf_y,
        max_y,
        min_z,
        haf_z,
        max_z,
    ] = cell.get_all();
    let nodes = [
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][haf_y][max_z].unwrap(),
        indexed_nodes[max_x][haf_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][haf_z].unwrap(),
        indexed_nodes[max_x][max_y][haf_z].unwrap(),
        indexed_nodes[max_x][haf_y][haf_z].unwrap(),
        indexed_nodes[haf_x][haf_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_face(nodes)
}

fn connectivity_00f000(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [
        min_x,
        haf_x,
        max_x,
        min_y,
        haf_y,
        max_y,
        min_z,
        haf_z,
        max_z,
    ] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][haf_z].unwrap(),
        indexed_nodes[min_x][max_y][haf_z].unwrap(),
        indexed_nodes[haf_x][max_y][min_z].unwrap(),
        indexed_nodes[haf_x][max_y][max_z].unwrap(),
        indexed_nodes[haf_x][max_y][haf_z].unwrap(),
        indexed_nodes[haf_x][haf_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_face(nodes)
}

fn connectivity_000f00(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [
        min_x,
        haf_x,
        max_x,
        min_y,
        haf_y,
        max_y,
        min_z,
        haf_z,
        max_z,
    ] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][haf_z].unwrap(),
        indexed_nodes[min_x][min_y][haf_z].unwrap(),
        indexed_nodes[min_x][haf_y][min_z].unwrap(),
        indexed_nodes[min_x][haf_y][max_z].unwrap(),
        indexed_nodes[min_x][haf_y][haf_z].unwrap(),
        indexed_nodes[haf_x][haf_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_face(nodes)
}

fn connectivity_0000f0(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [
        min_x,
        haf_x,
        max_x,
        min_y,
        haf_y,
        max_y,
        min_z,
        haf_z,
        max_z,
    ] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][haf_y][min_z].unwrap(),
        indexed_nodes[min_x][haf_y][min_z].unwrap(),
        indexed_nodes[haf_x][min_y][min_z].unwrap(),
        indexed_nodes[haf_x][max_y][min_z].unwrap(),
        indexed_nodes[haf_x][haf_y][min_z].unwrap(),
        indexed_nodes[haf_x][haf_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_face(nodes)
}

fn connectivity_00000f(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [
        min_x,
        haf_x,
        max_x,
        min_y,
        haf_y,
        max_y,
        min_z,
        haf_z,
        max_z,
    ] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[haf_x][max_y][max_z].unwrap(),
        indexed_nodes[haf_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][haf_y][max_z].unwrap(),
        indexed_nodes[max_x][haf_y][max_z].unwrap(),
        indexed_nodes[haf_x][haf_y][max_z].unwrap(),
        indexed_nodes[haf_x][haf_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_face(nodes)
}

fn connectivity_ee0000(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, _, max_x, min_y, _, max_y, min_z, haf_z, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_b(nodes)
}

fn connectivity_e00e00(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, _, max_x, min_y, _, max_y, min_z, haf_z, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_a(nodes)
}

fn connectivity_e000e0(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, haf_x, max_x, min_y, _, max_y, min_z, _, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[haf_x][min_y][min_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_a(nodes)
}

fn connectivity_0ee000(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, _, max_x, min_y, _, max_y, min_z, haf_z, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_a(nodes)
}

fn connectivity_0e000e(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, _, max_x, min_y, haf_y, max_y, min_z, _, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][haf_y][max_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_a(nodes)
}

fn connectivity_00ee00(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, _, max_x, min_y, _, max_y, min_z, haf_z, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][max_y][haf_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_b(nodes)
}

fn connectivity_00e00e(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, haf_x, max_x, min_y, _, max_y, min_z, _, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[haf_x][max_y][max_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_a(nodes)
}

fn connectivity_000ee0(
    cell: &Cell,
    indexed_nodes: &[Vec<Vec<Option<usize>>>],
) -> Vec<[usize; TET]> {
    let [min_x, _, max_x, min_y, haf_y, max_y, min_z, _, max_z] = cell.get_all();
    let nodes = [
        indexed_nodes[min_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][max_z].unwrap(),
        indexed_nodes[max_x][min_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][max_z].unwrap(),
        indexed_nodes[max_x][max_y][min_z].unwrap(),
        indexed_nodes[min_x][haf_y][min_z].unwrap(),
    ];
    TetrahedralTransition::one_edge_a(nodes)
}

type CoordinatesOutput = (Blocks, Vec<[usize; 4]>, Vec<Vec<Vec<Option<usize>>>>);

pub fn coordinates(tree: &Octree, removed_data: &Blocks) -> CoordinatesOutput {
    let mut element_blocks = vec![];
    let mut indexed_coordinates = vec![];
    let mut node_index: usize = NODE_NUMBERING_OFFSET;
    #[cfg(feature = "profile")]
    let temporary = std::time::Instant::now();
    let mut indexed_nodes =
        vec![vec![vec![None; tree.nel.z() + 1]; tree.nel.y() + 1]; tree.nel.x() + 1];
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;91m✰ Indexed nodes\x1b[0m {:?} ",
        temporary.elapsed()
    );
    #[cfg(feature = "profile")]
    let temporary = std::time::Instant::now();
    tree.iter()
        .filter(|cell| cell.is_leaf() && removed_data.binary_search(&cell.get_block()).is_err())
        .for_each(|leaf| {
            leaf.get_nodal_indices_cell()
                .into_iter()
                .for_each(|[i, j, k]| {
                    if indexed_nodes[i][j][k].is_none() {
                        indexed_coordinates.push([node_index, i, j, k]);
                        indexed_nodes[i][j][k] = Some(node_index);
                        node_index += 1;
                    }
                });
            match tree.neighbors_template(leaf) {
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..NUM_TETS_PER_HEX).for_each(|_| element_blocks.push(leaf.get_block()));
                }
                [
                    Neighbor::Face,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..20).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_f00000(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::Face,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..20).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_0f0000(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Face,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..20).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_00f000(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Face,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..20).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_000f00(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Face,
                    Neighbor::None,
                ] => {
                    (0..20).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_0000f0(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Face,
                ] => {
                    (0..20).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_00000f(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::Edges(_),
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..8).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_ee0000(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..7).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_e00e00(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::None,
                ] => {
                    (0..7).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_e000e0(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..7).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_0ee000(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                ] => {
                    (0..7).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_0e000e(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                ] => {
                    (0..8).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_00ee00(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                ] => {
                    (0..7).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_00e00e(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                [
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::None,
                    Neighbor::Edges(_),
                    Neighbor::Edges(_),
                    Neighbor::None,
                ] => {
                    (0..7).for_each(|_| element_blocks.push(leaf.get_block()));
                    coordinates_000ee0(leaf).into_iter().for_each(|[i, j, k]| {
                        if indexed_nodes[i][j][k].is_none() {
                            indexed_coordinates.push([node_index, i, j, k]);
                            indexed_nodes[i][j][k] = Some(node_index);
                            node_index += 1;
                        }
                    });
                }
                _ => {} // panic here to indicate a missing template
            }
        });
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;91m✰ Initial coordinates\x1b[0m {:?} ",
        temporary.elapsed()
    );
    (element_blocks, indexed_coordinates, indexed_nodes)
}

const fn coordinates_f00000(cell: &Cell) -> [[usize; 3]; 6] {
    let [min_x, haf_x, max_x, min_y, haf_y, _, min_z, haf_z, max_z] = cell.get_all();
    [
        [haf_x, min_y, max_z],
        [haf_x, min_y, min_z],
        [min_x, min_y, haf_z],
        [max_x, min_y, haf_z],
        [haf_x, min_y, haf_z],
        [haf_x, haf_y, haf_z],
    ]
}

const fn coordinates_0f0000(cell: &Cell) -> [[usize; 3]; 6] {
    let [_, haf_x, max_x, min_y, haf_y, max_y, min_z, haf_z, max_z] = cell.get_all();
    [
        [max_x, haf_y, max_z],
        [max_x, haf_y, min_z],
        [max_x, min_y, haf_z],
        [max_x, max_y, haf_z],
        [max_x, haf_y, haf_z],
        [haf_x, haf_y, haf_z],
    ]
}

const fn coordinates_00f000(cell: &Cell) -> [[usize; 3]; 6] {
    let [min_x, haf_x, max_x, _, haf_y, max_y, min_z, haf_z, max_z] = cell.get_all();
    [
        [max_x, max_y, haf_z],
        [min_x, max_y, haf_z],
        [haf_x, max_y, min_z],
        [haf_x, max_y, max_z],
        [haf_x, max_y, haf_z],
        [haf_x, haf_y, haf_z],
    ]
}

const fn coordinates_000f00(cell: &Cell) -> [[usize; 3]; 6] {
    let [min_x, haf_x, _, min_y, haf_y, max_y, min_z, haf_z, max_z] = cell.get_all();
    [
        [min_x, max_y, haf_z],
        [min_x, min_y, haf_z],
        [min_x, haf_y, min_z],
        [min_x, haf_y, max_z],
        [min_x, haf_y, haf_z],
        [haf_x, haf_y, haf_z],
    ]
}

const fn coordinates_0000f0(cell: &Cell) -> [[usize; 3]; 6] {
    let [min_x, haf_x, max_x, min_y, haf_y, max_y, min_z, haf_z, _] = cell.get_all();
    [
        [max_x, haf_y, min_z],
        [min_x, haf_y, min_z],
        [haf_x, min_y, min_z],
        [haf_x, max_y, min_z],
        [haf_x, haf_y, min_z],
        [haf_x, haf_y, haf_z],
    ]
}

const fn coordinates_00000f(cell: &Cell) -> [[usize; 3]; 6] {
    let [min_x, haf_x, max_x, min_y, haf_y, max_y, _, haf_z, max_z] = cell.get_all();
    [
        [haf_x, max_y, max_z],
        [haf_x, min_y, max_z],
        [min_x, haf_y, max_z],
        [max_x, haf_y, max_z],
        [haf_x, haf_y, max_z],
        [haf_x, haf_y, haf_z],
    ]
}

const fn coordinates_ee0000(cell: &Cell) -> [[usize; 3]; 1] {
    let [_, _, max_x, min_y, _, _, _, haf_z, _] = cell.get_all();
    [[max_x, min_y, haf_z]]
}

const fn coordinates_e00e00(cell: &Cell) -> [[usize; 3]; 1] {
    let [min_x, _, _, min_y, _, _, _, haf_z, _] = cell.get_all();
    [[min_x, min_y, haf_z]]
}

const fn coordinates_e000e0(cell: &Cell) -> [[usize; 3]; 1] {
    let [_, haf_x, _, min_y, _, _, min_z, _, _] = cell.get_all();
    [[haf_x, min_y, min_z]]
}

const fn coordinates_0ee000(cell: &Cell) -> [[usize; 3]; 1] {
    let [_, _, max_x, _, _, max_y, _, haf_z, _] = cell.get_all();
    [[max_x, max_y, haf_z]]
}

const fn coordinates_0e000e(cell: &Cell) -> [[usize; 3]; 1] {
    let [_, _, max_x, _, haf_y, _, _, _, max_z] = cell.get_all();
    [[max_x, haf_y, max_z]]
}

const fn coordinates_00ee00(cell: &Cell) -> [[usize; 3]; 1] {
    let [min_x, _, _, _, _, max_y, _, haf_z, _] = cell.get_all();
    [[min_x, max_y, haf_z]]
}

const fn coordinates_00e00e(cell: &Cell) -> [[usize; 3]; 1] {
    let [_, haf_x, _, _, _, max_y, _, _, max_z] = cell.get_all();
    [[haf_x, max_y, max_z]]
}

const fn coordinates_000ee0(cell: &Cell) -> [[usize; 3]; 1] {
    let [min_x, _, _, _, haf_y, _, min_z, _, _] = cell.get_all();
    [[min_x, haf_y, min_z]]
}
