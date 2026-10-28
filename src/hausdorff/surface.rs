use super::super::ErrorWrapper;
use conspire::{
    geometry::{
        Coordinate, Coordinates,
        bvh::BoundingVolumeHierarchy,
        mesh::{Connectivity, Mesh},
    },
    math::{FxHashMap, Tensor},
    units::Length,
};
const BARYCENTRIC_TOLERANCE: f64 = 1e-10;

type Point = [f64; 3];

pub struct Surface {
    mesh: Mesh<3>,
    points: Vec<Point>,
    triangles: Vec<[usize; 3]>,
    face_normals: Vec<Point>,
    edge_normals: FxHashMap<[usize; 2], Point>,
    vertex_normals: Vec<Point>,
    bvh: BoundingVolumeHierarchy<3>,
}

impl Surface {
    pub fn new(triangles: Vec<[usize; 3]>, coordinates: Coordinates<3>) -> Self {
        let points = coordinates
            .iter()
            .map(|coordinate| [0, 1, 2].map(|i| coordinate[i].value()))
            .collect::<Vec<_>>();
        let mesh = Mesh::from((
            vec![Connectivity::Triangular(triangles.clone().into())],
            coordinates,
        ));
        let bvh = BoundingVolumeHierarchy::from(&mesh);
        let face_normals = triangles
            .iter()
            .map(|&[a, b, c]| {
                normalized(cross(sub(points[b], points[a]), sub(points[c], points[a])))
            })
            .collect::<Vec<_>>();
        let mut edge_normals = FxHashMap::<[usize; 2], Point>::default();
        let mut vertex_normals = vec![[0.0; 3]; points.len()];
        triangles
            .iter()
            .zip(face_normals.iter())
            .for_each(|(triangle, &normal)| {
                (0..3).for_each(|corner| {
                    let node = triangle[corner];
                    let next = triangle[(corner + 1) % 3];
                    let previous = triangle[(corner + 2) % 3];
                    let entry = edge_normals.entry(edge(node, next)).or_default();
                    *entry = add(*entry, normal);
                    let angle = angle(
                        sub(points[next], points[node]),
                        sub(points[previous], points[node]),
                    );
                    vertex_normals[node] = add(vertex_normals[node], scale(normal, angle));
                })
            });
        Self {
            mesh,
            points,
            triangles,
            face_normals,
            edge_normals,
            vertex_normals,
            bvh,
        }
    }
    pub fn points(&self) -> &[Point] {
        &self.points
    }
    pub fn check(&self, name: &str) -> Result<(), ErrorWrapper> {
        let open = self.mesh.boundary_edges();
        if let Some(&[a, b]) = open.first() {
            return Err(format!(
                "The {name} is not closed: {} edges belong to one triangle only, the first from {:?} to {:?}",
                open.len(),
                self.points[a],
                self.points[b]
            )
            .into());
        }
        let non_manifold = self.mesh.non_manifold_edges();
        if let Some(&[a, b]) = non_manifold.first() {
            return Err(format!(
                "The {name} is not manifold: {} edges belong to three or more triangles, the first from {:?} to {:?}",
                non_manifold.len(),
                self.points[a],
                self.points[b]
            )
            .into());
        }
        let mut directed = FxHashMap::<[usize; 2], usize>::default();
        self.triangles.iter().for_each(|triangle| {
            (0..3).for_each(|corner| {
                *directed
                    .entry([triangle[corner], triangle[(corner + 1) % 3]])
                    .or_default() += 1
            })
        });
        let mut repeated = directed
            .into_iter()
            .filter_map(|(edge, count)| (count > 1).then_some(edge))
            .collect::<Vec<_>>();
        repeated.sort_unstable();
        if let Some(&[a, b]) = repeated.first() {
            return Err(format!(
                "The {name} is not consistently wound: {} edges are traversed in the same direction by both of their triangles, the first from {:?} to {:?}",
                repeated.len(),
                self.points[a],
                self.points[b]
            )
            .into());
        }
        let volume = self.volume();
        if volume <= 0.0 {
            return Err(format!(
                "The {name} is not outward: it encloses a volume of {volume}, which must be positive"
            )
            .into());
        }
        Ok(())
    }
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|&[a, b, c]| dot(self.points[a], cross(self.points[b], self.points[c])))
            .sum::<f64>()
            / 6.0
    }
    pub fn signed_distances(&self, samples: &[Point]) -> Vec<f64> {
        let elements = self
            .triangles
            .iter()
            .map(|triangle| triangle.as_slice())
            .collect::<Vec<_>>();
        samples
            .iter()
            .map(|&sample| {
                let (closest, triangle) = self
                    .bvh
                    .closest_point(
                        &Coordinate::from(sample.map(Length::meters)),
                        self.mesh.coordinates(),
                        &elements,
                    )
                    .expect("the surface has no triangles");
                let closest = [0, 1, 2].map(|i| closest[i].value());
                let offset = sub(sample, closest);
                let distance = norm(offset);
                if distance == 0.0 {
                    0.0
                } else {
                    distance.copysign(dot(offset, self.pseudonormal(triangle, closest)))
                }
            })
            .collect()
    }
    fn pseudonormal(&self, triangle: usize, point: Point) -> Point {
        let nodes = self.triangles[triangle];
        let weights = barycentric(nodes.map(|node| self.points[node]), point);
        if let Some(corner) = (0..3).find(|&corner| weights[corner] >= 1.0 - BARYCENTRIC_TOLERANCE)
        {
            self.vertex_normals[nodes[corner]]
        } else if let Some(corner) = (0..3).find(|&corner| weights[corner] <= BARYCENTRIC_TOLERANCE)
        {
            self.edge_normals[&edge(nodes[(corner + 1) % 3], nodes[(corner + 2) % 3])]
        } else {
            self.face_normals[triangle]
        }
    }
}

fn barycentric([a, b, c]: [Point; 3], point: Point) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(point, a);
    let d00 = dot(ab, ab);
    let d01 = dot(ab, ac);
    let d11 = dot(ac, ac);
    let d20 = dot(ap, ab);
    let d21 = dot(ap, ac);
    let denominator = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denominator;
    let w = (d00 * d21 - d01 * d20) / denominator;
    [1.0 - v - w, v, w]
}

fn edge(a: usize, b: usize) -> [usize; 2] {
    if a < b { [a, b] } else { [b, a] }
}

fn angle(u: Point, v: Point) -> f64 {
    let lengths = norm(u) * norm(v);
    if lengths == 0.0 {
        0.0
    } else {
        (dot(u, v) / lengths).clamp(-1.0, 1.0).acos()
    }
}

fn add(u: Point, v: Point) -> Point {
    [u[0] + v[0], u[1] + v[1], u[2] + v[2]]
}

fn sub(u: Point, v: Point) -> Point {
    [u[0] - v[0], u[1] - v[1], u[2] - v[2]]
}

fn scale(u: Point, factor: f64) -> Point {
    u.map(|entry| entry * factor)
}

fn dot(u: Point, v: Point) -> f64 {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
}

fn cross(u: Point, v: Point) -> Point {
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn norm(u: Point) -> f64 {
    dot(u, u).sqrt()
}

fn normalized(u: Point) -> Point {
    let length = norm(u);
    if length == 0.0 {
        u
    } else {
        scale(u, 1.0 / length)
    }
}
