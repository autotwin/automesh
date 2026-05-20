use crate::{
    Coordinate, Coordinates, fem::tri::TriangularFiniteElements, tessellation::Tessellation,
};
use conspire::math::{Scalar, Tensor};
use ndarray::parallel::prelude::*;
use std::f64::consts::TAU;

// TODO: ignore false intersections where normals are 90deg or less.

// TODO: "The SDF at a point is defined as the weighted average of all rays lengths which fall within one standard deviation from the median of all lengths."

// TODO: project SDF on faces to vertices using average of incident faces.

// TODO: bilateral smoothing to smooth out SDF across the mesh.

// TODO: spatial acceleration.

// TODO: parallelize.

// TODO: reconsider flood fill strategy for inside/outside if spatial acceleration pays off big here.

pub fn shape_diameter_function(
    tessellation: &Tessellation,
    half_angle: Scalar,
    number_of_rays: usize,
) -> Vec<Scalar> {
    let face_sdf = shape_diameter_function_faces(tessellation, half_angle, number_of_rays);

    let mut sums = vec![0.0; tessellation.data.vertices.len()];
    let mut counts = vec![0; tessellation.data.vertices.len()];

    for (face_index, face) in tessellation.data.faces.iter().enumerate() {
        if let Some(value) = face_sdf[face_index] {
            for &vertex in &face.vertices {
                sums[vertex] += value;
                counts[vertex] += 1;
            }
        }
    }

    sums.into_iter()
        .zip(counts)
        .map(|(sum, count)| {
            if count > 0 {
                sum / count as Scalar
            } else {
                panic!()
            }
        })
        .collect()
}

fn shape_diameter_function_faces(
    tessellation: &Tessellation,
    half_angle: Scalar,
    number_of_rays: usize,
) -> Vec<Option<Scalar>> {
    let coordinates: Coordinates = tessellation
        .data
        .vertices
        .iter()
        .map(|vertex| {
            Coordinate::const_from([vertex[0] as f64, vertex[1] as f64, vertex[2] as f64])
        })
        .collect();

    tessellation
        .data
        .faces
        .par_iter()
        .enumerate()
        .map(|(triangle_i, face_i)| {
            let apex = face_i
                .vertices
                .iter()
                .map(|&vertex| &coordinates[vertex])
                .sum::<Coordinate>()
                / 3.0;

            let normal_i = face_i.normal;
            let mut axis = Coordinate::const_from([
                -normal_i[0] as f64,
                -normal_i[1] as f64,
                -normal_i[2] as f64,
            ]);
            axis.normalize();

            let epsilon = 1e-8;
            let origin = &apex + &(&axis * epsilon);

            let triangles = triangles_in_cone(tessellation, &coordinates, triangle_i, half_angle);

            let ray_hits: Vec<(Coordinate, Scalar)> =
                conical_directions(&axis, half_angle, number_of_rays)
                    .into_iter()
                    .filter_map(|ray| {
                        triangles
                            .iter()
                            .filter_map(|&triangle_j| {
                                let face_j = &tessellation.data.faces[triangle_j];

                                let opposite_normal = normal_i
                                    .0
                                    .iter()
                                    .zip(face_j.normal.0.iter())
                                    .map(|(n_i_a, n_j_a)| n_i_a * n_j_a)
                                    .sum::<f32>()
                                    < 0.0;

                                if opposite_normal {
                                    TriangularFiniteElements::intersection(
                                        &ray,
                                        &origin,
                                        &coordinates,
                                        face_j.vertices,
                                    )
                                    .map(|point| (point - &origin).norm())
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .map(|length| (ray, length))
                    })
                    .collect();

            aggregate_sdf(&ray_hits, &axis)
        })
        .collect()
}

fn aggregate_sdf(ray_lengths: &[(Coordinate, Scalar)], axis: &Coordinate) -> Option<Scalar> {
    if ray_lengths.is_empty() {
        return None;
    }

    let mut lengths: Vec<Scalar> = ray_lengths.iter().map(|(_, l)| *l).collect();
    lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = lengths.len();
    let median = if n % 2 == 1 {
        lengths[n / 2]
    } else {
        (lengths[n / 2 - 1] + lengths[n / 2]) / 2.0
    };

    let mean = lengths.iter().copied().sum::<Scalar>() / n as Scalar;
    let variance = lengths
        .iter()
        .map(|l| {
            let d = *l - mean;
            d * d
        })
        .sum::<Scalar>()
        / n as Scalar;
    let std_dev = variance.sqrt();

    let eps = 1e-8;

    let filtered: Vec<_> = ray_lengths
        .iter()
        .filter(|(_, l)| (*l - median).abs() <= std_dev)
        .collect();

    if filtered.is_empty() {
        return None;
    }

    let (weighted_sum, weight_sum) =
        filtered
            .into_iter()
            .fold((0.0, 0.0), |(ws, wsum), (ray, length)| {
                let cos_angle = (ray * axis).clamp(-1.0, 1.0);
                let angle = cos_angle.acos();
                let weight = 1.0 / angle.max(eps);

                (ws + weight * *length, wsum + weight)
            });

    if weight_sum > 0.0 {
        Some(weighted_sum / weight_sum)
    } else {
        None
    }
}

fn conical_directions(
    axis: &Coordinate,
    half_angle: Scalar,
    number_of_directions: usize,
) -> Vec<Coordinate> {
    let [n, u, v] = axis.orthonormal_basis().into();
    let cos_max = half_angle.cos();
    let golden = (5.0_f64.sqrt() - 1.0) / 2.0;
    (0..number_of_directions)
        .map(|i| {
            let fi = i as f64;
            let fnn = number_of_directions as f64;
            let z = 1.0 - ((fi + 0.5) / fnn) * (1.0 - cos_max);
            let phi = TAU * ((fi * golden).fract());
            let r = (1.0 - z * z).sqrt();
            let x = r * phi.cos();
            let y = r * phi.sin();
            (&u * x + &v * y + &n * z).normalized()
        })
        .collect()
}

// Is possible but maybe not that important that triangle could intersect cone while none of its vertices do?
// It is temporary anyway, will eventually speed up things using some fancy tree instead.

fn triangles_in_cone(
    tessellation: &Tessellation,
    coordinates: &Coordinates,
    triangle_0: usize,
    half_angle: Scalar,
) -> Vec<usize> {
    let face_0 = tessellation.data.faces[triangle_0];
    let apex = face_0
        .vertices
        .iter()
        .map(|&vertex| &coordinates[vertex])
        .sum::<Coordinate>()
        / 3.0;
    let normal = face_0.normal;
    let mut axis =
        Coordinate::const_from([-normal[0] as f64, -normal[1] as f64, -normal[2] as f64]);
    axis.normalize();
    let cos_angle = half_angle.cos();
    tessellation
        .data
        .faces
        .iter()
        .enumerate()
        .filter_map(|(triangle, face)| {
            if triangle == triangle_0 {
                None
            } else {
                if face
                    .vertices
                    .iter()
                    .any(|&vertex| point_in_cone(&coordinates[vertex], &apex, &axis, cos_angle))
                {
                    Some(triangle)
                } else {
                    None
                }
            }
        })
        .collect()
}

fn point_in_cone(
    point: &Coordinate,
    apex: &Coordinate,
    axis_unit: &Coordinate,
    cos_angle: Scalar,
) -> bool {
    let u = point - apex;
    let du = &u * axis_unit;
    if du <= 0.0 {
        return false;
    }
    let uu = u.norm_squared();
    if uu == 0.0 {
        return true;
    }
    du * du >= uu * cos_angle * cos_angle
}
