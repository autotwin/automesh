use super::{Coordinates, Faces, HEX, HexConnectivity, Indices, NodeMap, Octree};

pub mod edge_template_1;
pub mod edge_template_2;
pub mod edge_template_3;
pub mod edge_template_4;
pub mod face_template_0;
pub mod face_template_1;
mod vertex_template_1; // (O, A, AB, B) => (o, a, ab, b)
mod vertex_template_10; // (O, a, AB, b) => (O, a, ab, b)
mod vertex_template_11; // (O, a, AB, b) => (O, a, AB, b)
mod vertex_template_12; // (O, a, AB, b) => (o, a, ab, b)
mod vertex_template_2; // (O, a, ab, b) => (O, a, ab, b)
mod vertex_template_3; // (O, A, AB, B) => (o, A, AB, b)
mod vertex_template_4; // (O, A, AB, B) => (o, A, AB, B)
mod vertex_template_5; // (O, A, AB, B) => (o, A, ab, b)
mod vertex_template_6; // (O, A, AB, b) => (o, A, ab, b)
mod vertex_template_7; // (O, a, ab, b) => (o, a, ab, b)
mod vertex_template_8; // (O, A, AB, b) => (o, a, ab, b)
mod vertex_template_9; // (O, a, ab, b) => (o, a, AB, b)

pub fn apply_concurrently(
    index: usize,
    cells_nodes: &[usize],
    nodes_map: &NodeMap,
    tree: &Octree,
    nodal_coordinates: &Coordinates,
) -> HexConnectivity {
    match index {
        1 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_1::DATA,
            vertex_template_1::template,
        ),
        2 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_2::DATA,
            vertex_template_2::template,
        ),
        3 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_3::DATA,
            vertex_template_3::template,
        ),
        4 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_4::DATA,
            vertex_template_4::template,
        ),
        5 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_5::DATA,
            vertex_template_5::template,
        ),
        6 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_6::DATA,
            vertex_template_6::template,
        ),
        7 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_7::DATA,
            vertex_template_7::template,
        ),
        8 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_8::DATA,
            vertex_template_8::template,
        ),
        9 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_9::DATA,
            vertex_template_9::template,
        ),
        10 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_10::DATA,
            vertex_template_10::template,
        ),
        11 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_11::DATA,
            vertex_template_11::template,
        ),
        12 => apply_vertex_template(
            cells_nodes,
            tree,
            vertex_template_12::DATA,
            vertex_template_12::template,
        ),
        13 => edge_template_2::apply(cells_nodes, nodes_map, tree, nodal_coordinates),
        14 => edge_template_4::apply(cells_nodes, nodes_map, tree, nodal_coordinates),
        _ => panic!(),
    }
}

fn apply_vertex_template<const T: usize>(
    cells_nodes: &[usize],
    tree: &Octree,
    data: [[usize; 11]; T],
    template: impl Fn(
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        &Faces,
        &Indices,
        &[usize],
        &Octree,
    ) -> Option<[usize; HEX]>,
) -> HexConnectivity {
    tree.iter()
        .filter_map(|cell| tree.cell_contains_leaves(cell))
        .flat_map(|(cell_subcells, cell_faces)| {
            data.iter()
                .filter_map(|data| {
                    template(
                        data[0],
                        data[1],
                        data[2],
                        data[3],
                        data[4],
                        data[5],
                        data[6],
                        data[7],
                        data[8],
                        data[9],
                        data[10],
                        cell_faces,
                        cell_subcells,
                        cells_nodes,
                        tree,
                    )
                })
                .collect::<HexConnectivity>()
        })
        .collect()
}
