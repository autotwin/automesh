use crate::{
    Coordinate, Coordinates, NSD, Nel, Remove, Scale, Translate,
    fem::{Connectivity, Size, TRI, TriangularFiniteElements, VecConnectivity},
    tessellation::Tessellation,
    tree::{Cell, NUM_FACES, Octree, PADDING},
};
use conspire::math::{Scalar, Tensor, TensorArray};
use std::collections::HashMap;

#[cfg(feature = "profile")]
use std::time::Instant;

impl From<Octree> for Tessellation {
    fn from(tree: Octree) -> Self {
        TriangularFiniteElements::from(tree).into()
    }
}

type Bins = HashMap<[usize; NSD], Vec<usize>>;
pub type Samples = Vec<Vec<Vec<bool>>>;

type OctreeAndStuff = (
    Octree,
    Vec<Vec<Vec<bool>>>,
    Connectivity<TRI>,
    VecConnectivity,
    HashMap<[usize; NSD], Vec<usize>>,
);

pub fn octree_from_surface(
    triangular_finite_elements: TriangularFiniteElements,
    size: Size,
) -> OctreeAndStuff {
    let (blocks, connectivity, mut surface_coordinates, inverse_connectivity) =
        triangular_finite_elements.into();
    let block = blocks[0];
    if !blocks.iter().all(|entry| entry == &block) {
        panic!()
    }
    if let Some(size) = size {
        let mut tree = octree_from_bounding_cube(&mut surface_coordinates, size);
        #[cfg(feature = "profile")]
        let time = Instant::now();
        let (bins, samples) = bin_samples_on_surface(tree.nel(), surface_coordinates);
        let mut index = 0;
        while index < tree.len() {
            if tree[index].is_voxel() || !tree[index].any_samples_inside(&samples) {
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
        (tree, samples, connectivity, inverse_connectivity, bins)
    } else {
        todo!()
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

fn bin_samples_on_surface(nel: Nel, surface_coordinates: Coordinates) -> (Bins, Samples) {
    let rounded: Vec<[_; NSD]> = surface_coordinates
        .into_iter()
        .map(|coordinates| {
            [
                coordinates[0].floor() as usize,
                coordinates[1].floor() as usize,
                coordinates[2].floor() as usize,
            ]
        })
        .collect();
    let (nel_x, nel_y, nel_z) = nel.into();
    let mut samples = vec![vec![vec![false; nel_x]; nel_y]; nel_z];
    let mut bins = HashMap::<_, Vec<_>>::new();
    rounded
        .into_iter()
        .enumerate()
        .for_each(|(node, [i, j, k])| {
            samples[i][j][k] = true;
            bins.entry([i, j, k]).or_default().push(node);
        });
    (bins, samples)
}
