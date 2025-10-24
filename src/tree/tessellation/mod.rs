use crate::{
    Coordinate, Coordinates, NSD, Nel, Remove, Scale, Translate,
    fem::{FiniteElementMethods, Size, TriangularFiniteElements},
    tessellation::Tessellation,
    tree::{Cell, NUM_FACES, Octree, PADDING},
};
use conspire::math::{Scalar, Tensor, TensorArray};

pub fn octree_from_tessellation(tessellation: Tessellation, size: Size) -> Octree {
    let triangular_finite_elements = TriangularFiniteElements::from(tessellation);
    let (blocks, _, mut surface_coordinates) = triangular_finite_elements.data();
    let block = blocks[0];
    if !blocks.iter().all(|entry| entry == &block) {
        panic!()
    }
    if let Some(size) = size {
        let mut tree = octree_from_bounding_cube(&mut surface_coordinates, size);
        let mut index = 0;
        while index < tree.len() {
            if tree[index].is_voxel() || !tree[index].any_coordinates_inside(&surface_coordinates) {
                tree[index].block = Some(block)
            } else {
                tree.subdivide(index)
            }
            index += 1;
        }
        tree.balance_and_pair(true);
        tree
    } else {
        todo!()
    }
}

pub fn octree_from_bounding_cube(samples: &mut Coordinates, minimum_cell_size: Scalar) -> Octree {
    let (minimum, mut maximum) = samples.iter().fold(
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
    maximum -= &minimum;
    let scale = 1.0 / minimum_cell_size;
    let total_length = maximum.clone().into_iter().reduce(f64::max).unwrap();
    let nel0 = total_length / minimum_cell_size;
    let nel = if nel0 > 0.0 && (nel0.log2().fract() == 0.0) {
        nel0 as usize
    } else {
        2.0_f64.powf(nel0.log2().ceil()) as usize
    };
    samples.iter_mut().for_each(|sample| {
        *sample -= &minimum;
        *sample *= &scale;
    });
    let mut tree = Octree {
        nel: Nel::from([nel; NSD]),
        octree: vec![],
        remove: Remove::Some(vec![PADDING]),
        scale: Scale::from([1.0 / scale; NSD]),
        translate: Translate::from(minimum),
    };
    tree.push(Cell {
        block: None,
        cells: None,
        faces: [None; NUM_FACES],
        lngth: nel as u16,
        min_x: 0,
        min_y: 0,
        min_z: 0,
    });
    tree
}
