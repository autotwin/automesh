use crate::{
    Coordinate, Coordinates, NSD, Nel, Remove, Scale, Translate,
    fem::{Size, TriangularFiniteElements},
    tessellation::Tessellation,
    tree::{Cell, NUM_FACES, Octree, PADDING},
};
use conspire::math::{Scalar, Tensor, TensorArray};

#[cfg(feature = "profile")]
use std::time::Instant;

impl From<Octree> for Tessellation {
    fn from(tree: Octree) -> Self {
        TriangularFiniteElements::from(tree).into()
    }
}

impl From<(TriangularFiniteElements, Size)> for Octree {
    fn from((triangular_finite_elements, size): (TriangularFiniteElements, Size)) -> Self {
        let (blocks, _, mut surface_coordinates) = triangular_finite_elements.into();
        let block = blocks[0];
        if !blocks.iter().all(|entry| entry == &block) {
            panic!()
        }
        if let Some(size) = size {
            let mut tree = octree_from_bounding_cube(&mut surface_coordinates, size);
            #[cfg(feature = "profile")]
            let time = Instant::now();
            let mut index = 0;
            while index < tree.len() {
                if tree[index].is_voxel()
                    || !tree[index].any_coordinates_inside(&surface_coordinates)
                {
                    tree[index].block = Some(block)
                } else {
                    tree.subdivide(index)
                }
                index += 1;
            }
            #[cfg(feature = "profile")]
            println!(
                "             \x1b[1;93mSubdivision from size\x1b[0m {:?}",
                time.elapsed()
            );
            tree.balance_and_pair(true);
            tree
        } else {
            todo!()
        }
    }
}

pub fn octree_from_bounding_cube(samples: &mut Coordinates, minimum_cell_size: Scalar) -> Octree {
    #[cfg(feature = "profile")]
    let time = Instant::now();
    let (minimum, maximum) = samples.iter().fold(
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
    let maximum_length = (maximum.clone() - &minimum)
        .into_iter()
        .reduce(f64::max)
        .unwrap();
    let scale = 1.0 / minimum_cell_size;
    let nel = 2.0_f64.powf((maximum_length / minimum_cell_size).log2().ceil()) as usize;
    let translation =
        (minimum + maximum) * 0.5 - Coordinate::new([0.5 * (nel as f64) / scale; NSD]);
    samples.iter_mut().for_each(|sample| {
        *sample -= &translation;
        *sample *= &scale;
    });
    let mut tree = Octree {
        nel: Nel::from([nel; NSD]),
        octree: vec![],
        remove: Remove::Some(vec![PADDING]),
        scale: Scale::from([1.0 / scale; NSD]),
        translate: Translate::from(translation),
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
    #[cfg(feature = "profile")]
    println!(
        "             \x1b[1;93mOctree initialization\x1b[0m {:?}",
        time.elapsed()
    );
    tree
}
