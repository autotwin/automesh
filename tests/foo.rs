//! This module tests ideas for a QuadTree implementation.
//! 
//! # Example
//! 
//! ```
//! cargo test foo -- --nocapture && python target/foo.py
//! ```
//! 
//! ## Reference
//! 
//! [primal/dual quadrilateral transitions](https://github.com/sandialabs/sibl/blob/master/geo/doc/dual_quad_transitions.md) 


use flavio::math::{Tensor, TensorRank1};
use std::array::from_fn;

type Faces<const F: usize> = [Option<usize>; F];
type Indices<const N: usize> = [usize; N];
type Point = TensorRank1<2, 1>;
type Points = Vec<Point>;

#[derive(Debug)]
struct Cell {
    cells: Option<Indices<4>>,
    level: usize,
    faces: Faces<4>,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    template: Option<Template>,
}

type Cells<const N: usize> = [Cell; N];

impl Cell {
    fn contains(&self, points: &Points) -> bool {
        for point in points {
            // TODO: discuss if we want to use traditional definitions
            // of contains, i.e.,
            // if &point[0] >= self.get_min_x()
            //     && &point[0] < self.get_max_x()
            //     && &point[1] >= self.get_min_y()
            //     && &point[1] < self.get_max_y()
            if &point[0] >= self.get_min_x()
                && &point[0] <= self.get_max_x()
                && &point[1] >= self.get_min_y()
                && &point[1] <= self.get_max_y()
            {
                return true;
            }
        }
        false
    }
    fn get_faces(&self) -> &Faces<4> {
        &self.faces
    }
    fn get_level(&self) -> &usize {
        &self.level
    }
    fn get_min_x(&self) -> &f64 {
        &self.min_x
    }
    fn get_min_y(&self) -> &f64 {
        &self.min_y
    }
    fn get_max_x(&self) -> &f64 {
        &self.max_x
    }
    fn get_max_y(&self) -> &f64 {
        &self.max_y
    }
    fn subdivide(&mut self, indices: Indices<4>) -> Cells<4> {
        self.cells = Some(indices);
        let level = self.get_level() + 1;
        let min_x = self.get_min_x();
        let min_y = self.get_min_y();
        let max_x = self.get_max_x();
        let max_y = self.get_max_y();
        let val_x = 0.5 * (min_x + max_x);
        let val_y = 0.5 * (min_y + max_y);
        [
            Cell {
                cells: None,
                faces: [None, Some(indices[1]), Some(indices[2]), None],
                level,
                min_x: *min_x,
                min_y: *min_y,
                max_x: val_x,
                max_y: val_y,
                template: None,
            },
            Cell {
                cells: None,
                faces: [None, None, Some(indices[3]), Some(indices[0])],
                level,
                min_x: val_x,
                min_y: *min_y,
                max_x: *max_x,
                max_y: val_y,
                template: None,
            },
            Cell {
                cells: None,
                faces: [Some(indices[0]), Some(indices[3]), None, None],
                level,
                min_x: *min_x,
                min_y: val_y,
                max_x: val_x,
                max_y: *max_y,
                template: None,
            },
            Cell {
                cells: None,
                faces: [Some(indices[1]), None, None, Some(indices[2])],
                level,
                min_x: val_x,
                min_y: val_y,
                max_x: *max_x,
                max_y: *max_y,
                template: None,
            },
        ]
    }
}

type QuadTree = Vec<Cell>;

type Connectivity = Vec<[usize; 4]>;

#[derive(Debug)]
enum Template {
    T0000(Connectivity),
    T0001(Connectivity),
    T1000(Connectivity),
    None,
}

trait Tree {
    fn balance(&mut self, levels: &usize);
    fn from_bounds(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self;
    fn from_points(&mut self, levels: &usize, points: Points);
    fn prune(&mut self);
    fn sandbox(&mut self, levels: &usize) -> (Connectivity, Points);
    fn subdivide(&mut self, index: usize);
    fn template(&self, cell: &Cell, node: &mut usize) -> (Template, Points);
}

impl Tree for QuadTree {
    fn balance(&mut self, levels: &usize) {
        let mut balanced;
        let mut index;
        let mut subdivide;
        for _iteration in 1.. {
            balanced = true;
            index = 0;
            subdivide = false;
            while index < self.len() {
                if self[index].get_level() < &(levels - 1) {
                    if self[index].cells.is_none() {
                        'faces: for (face, face_cell) in self[index].get_faces().iter().enumerate() {
                            if let Some(neighbor) = face_cell {
                                if let Some(kids) = self[*neighbor].cells {
                                    if match face {
                                        0 => {
                                            self[kids[2]].cells.is_some()
                                                || self[kids[3]].cells.is_some()
                                        }
                                        1 => {
                                            self[kids[0]].cells.is_some()
                                                || self[kids[2]].cells.is_some()
                                        }
                                        2 => {
                                            self[kids[0]].cells.is_some()
                                                || self[kids[1]].cells.is_some()
                                        }
                                        3 => {
                                            self[kids[1]].cells.is_some()
                                                || self[kids[3]].cells.is_some()
                                        }
                                        _ => panic!(),
                                    } {
                                        subdivide = true;
                                        break 'faces;
                                    }
                                }
                            }
                        }
                        if subdivide {
                            balanced = false;
                            self.subdivide(index);
                            subdivide = false;
                        }
                    }
                }
                index += 1;
            }
            if balanced {
                break;
            }
        }
    }
    fn from_bounds(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        vec![Cell {
            cells: None,
            faces: [None; 4],
            level: 0,
            min_x,
            min_y,
            max_x,
            max_y,
            template: None,
        }]
    }
    fn from_points(&mut self, levels: &usize, points: Points) {
        let mut index = 0;
        while index < self.len() {
            if self[index].get_level() < levels && self[index].contains(&points) {
                self.subdivide(index);
            }
            index += 1;
        }
    }
    fn prune(&mut self) {
        self.retain(|cell| cell.cells.is_none())
    }
    fn sandbox(&mut self, levels: &usize) -> (Connectivity, Points) {
        let mut connectivity: Vec<[usize; 4]> = vec![];
        let mut node = 1;
        let mut nodal_coordinates = vec![];
        let maps = (0..*levels)
            .rev()
            .map(|level| {
                self.iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.get_level() == &level)
                    .map(|(index, _)| index)
                    .collect()
            })
            .collect::<Vec<Vec<usize>>>();
        maps.into_iter().for_each(|map| {
            map.into_iter().for_each(|index| {
                let (template, mut points) = self.template(&self[index], &mut node);
                match template {
                    Template::T0000(mut conn) => {
                        self[index].template = Some(Template::T0000(conn.clone()));
                        connectivity.append(&mut conn);
                        nodal_coordinates.append(&mut points);
                    }
                    Template::T0001(mut conn) => {
                        self[index].template = Some(Template::T0001(conn.clone()));
                        connectivity.append(&mut conn);
                        nodal_coordinates.append(&mut points);
                    }
                    Template::T1000(mut conn) => {
                        self[index].template = Some(Template::T1000(conn.clone()));
                        connectivity.append(&mut conn);
                        nodal_coordinates.append(&mut points);
                    }
                    _ => {}
                }
            })
        });
        (connectivity, nodal_coordinates)
    }
    fn subdivide(&mut self, index: usize) {
        let new_index = self.len() - 1;
        let new_indices = [new_index + 1, new_index + 2, new_index + 3, new_index + 4];
        let mut new_cells = self[index].subdivide(new_indices);
        self[index]
            .get_faces()
            .clone()
            .iter()
            .enumerate()
            .for_each(|(face, face_cell)| {
                if let Some(neighbor) = face_cell {
                    if let Some(kids) = self[*neighbor].cells {
                        match face {
                            0 => {
                                // if self[kids[2]].faces[2].is_some() || self[kids[3]].faces[2].is_some() {
                                //     panic!("TEMPORARY CHECKS")
                                // }
                                new_cells[0].faces[0] = Some(kids[2]);
                                new_cells[1].faces[0] = Some(kids[3]);
                                self[kids[2]].faces[2] = Some(new_indices[0]);
                                self[kids[3]].faces[2] = Some(new_indices[1]);
                            }
                            1 => {
                                // if self[kids[0]].faces[3].is_some() || self[kids[2]].faces[3].is_some() {
                                //     panic!("TEMPORARY CHECKS")
                                // }
                                new_cells[1].faces[1] = Some(kids[0]);
                                new_cells[3].faces[1] = Some(kids[2]);
                                self[kids[0]].faces[3] = Some(new_indices[1]);
                                self[kids[2]].faces[3] = Some(new_indices[3]);
                            }
                            2 => {
                                // if self[kids[0]].faces[0].is_some() || self[kids[1]].faces[0].is_some() {
                                //     panic!("TEMPORARY CHECKS")
                                // }
                                new_cells[2].faces[2] = Some(kids[0]);
                                new_cells[3].faces[2] = Some(kids[1]);
                                self[kids[0]].faces[0] = Some(new_indices[2]);
                                self[kids[1]].faces[0] = Some(new_indices[3]);
                            }
                            3 => {
                                // if self[kids[1]].faces[1].is_some() || self[kids[3]].faces[1].is_some() {
                                //     panic!("TEMPORARY CHECKS")
                                // }
                                new_cells[0].faces[3] = Some(kids[1]);
                                new_cells[2].faces[3] = Some(kids[3]);
                                self[kids[1]].faces[1] = Some(new_indices[0]);
                                self[kids[3]].faces[1] = Some(new_indices[2]);
                            }
                            _ => panic!(),
                        }
                    }
                }
            });
        self.extend(new_cells);
    }
    fn template(&self, cell: &Cell, node: &mut usize) -> (Template, Points) {
        if let Some(cells) = cell.cells {
            let ave_x = 0.5 * cell.get_max_x() + 0.5 * cell.get_min_x();
            let ave_y = 0.5 * cell.get_max_y() + 0.5 * cell.get_min_y();
            let len_x = 0.5 * cell.get_max_x() - 0.5 * cell.get_min_x();
            let len_y = 0.5 * cell.get_max_y() - 0.5 * cell.get_min_y();
            let cx1 = ave_x - len_x / 6.0;
            let cx2 = ave_x + len_x / 6.0;
            let cy1 = ave_y - len_y / 6.0;
            let cy2 = ave_y + len_y / 6.0;
            let x_1 = 0.75 * cell.get_min_x() + 0.25 * cell.get_max_x();
            let y_1 = 0.75 * cell.get_min_y() + 0.25 * cell.get_max_y();
            let x_2 = 0.25 * cell.get_min_x() + 0.75 * cell.get_max_x();
            let y_2 = 0.25 * cell.get_min_y() + 0.75 * cell.get_max_y();
            let foo: Vec<bool> = cells
                .iter()
                .map(|&subcell| self[subcell].cells.is_some())
                .collect();
            let node_0 = *node;
            if !foo[0] && !foo[1] && !foo[2] && !foo[3] {
                *node += 4;
                return (
                    Template::T0000(vec![from_fn(|k| node_0 + k)]),
                    vec![
                        Point::new([x_1, y_1]),
                        Point::new([x_2, y_1]),
                        Point::new([x_2, y_2]),
                        Point::new([x_1, y_2]),
                    ],
                );
            } else if !foo[0] && !foo[1] && !foo[2] && foo[3] {
                let nodes = match &self[cells[3]].template {
                    Some(Template::T0000(connectivity)) => connectivity[0],
                    _ => panic!(),
                };
                *node += 6;
                return (
                    Template::T0001(vec![
                        [node_0, node_0 + 1, node_0 + 4, node_0 + 3],
                        [node_0 + 1, nodes[1], nodes[0], node_0 + 4],
                        [node_0 + 2, node_0 + 5, nodes[0], nodes[3]],
                        [node_0, node_0 + 3, node_0 + 5, node_0 + 2],
                        [node_0 + 3, node_0 + 4, nodes[0], node_0 + 5],
                    ]),
                    vec![
                        Point::new([x_1, y_1]),
                        Point::new([x_2, y_1]),
                        Point::new([x_1, y_2]),
                        Point::new([cx1, cy1]),
                        Point::new([cx2, cy1]),
                        Point::new([cx1, cy2]),
                    ],
                );
            //
            // think you have to have some consistent numbering scheme between the rotationally-equivalent cases
            // then you will be able to stich together templates more easily
            // like for constructing T0112, need to know about NE subcell template
            // need to know the node numbers of the left-most and lower-most nodes for the templates of the sub-sub cells
            // so set up some expected numbering, driven by the basic boxes
            // if it's not a basic box (it's T0001 or something), what you would connect to in place of the basic box nodes should match up IN ALL CASES
            //
            //
            } else if foo[0] && !foo[1] && !foo[2] && !foo[3] {
                let nodes = match &self[cells[0]].template {
                    Some(Template::T0000(connectivity)) => connectivity[0],
                    _ => panic!(),
                };
                *node += 6;
                return (
                    Template::T1000(vec![
                        [nodes[1], node_0, node_0 + 3, nodes[2]],
                        [node_0, node_0 + 1, node_0 + 4, node_0 + 3],
                        [node_0 + 1, node_0 + 2, node_0 + 5, node_0 + 4],
                        [nodes[3], nodes[2], node_0 + 5, node_0 + 2],
                        [nodes[2], node_0 + 3, node_0 + 4, node_0 + 5],
                    ]),
                    vec![
                        Point::new([x_2, y_1]),
                        Point::new([x_2, y_2]),
                        Point::new([x_1, y_2]),
                        Point::new([cx2, cy1]),
                        Point::new([cx2, cy2]),
                        Point::new([cx1, cy2]),
                    ],
                );
            }
        } else {
            return (Template::None, vec![]);
        }
        (Template::None, vec![])
    }
}

#[test]
fn foo() {
    let xmin = 0.0;
    let xmax = 8.0;
    let ymin = 0.0;
    let ymax = 8.0;
    let levels = 5;
    // let levels = 7;
    let points = vec![
        Point::new([1.2, 3.3]),
        Point::new([5.2, 2.3]),
        Point::new([6.6, 6.6]),
        //     Point::new([7.0, 0.0]),
        //     Point::new([6.999861549230783, 0.04402603321247425]),
        //     Point::new([6.999446202399878, 0.0880503248711874]),
        //     Point::new([6.998753975937312, 0.13207113349126964]),
        //     Point::new([6.997784897225737, 0.17608671772563092]),
        //     Point::new([6.9965390045993505, 0.2200953364338442]),
        //     Point::new([6.99501634734238, 0.26409524875102025]),
        //     Point::new([6.9932169856871305, 0.30808471415667166]),
        //     Point::new([6.991140990811604, 0.3520619925435633]),
        //     Point::new([6.988788444836682, 0.3960253442865464]),
        //     Point::new([6.9861594408228775, 0.4399730303113736]),
        //     Point::new([6.983254082766658, 0.4839033121634918]),
        //     Point::new([6.980072485596325, 0.5278144520768111]),
        //     Point::new([6.97661477516747, 0.5717047130424464]),
        //     Point::new([6.972881088258, 0.6155723588774288]),
        //     Point::new([6.968871572562723, 0.6594156542933838]),
        //     Point::new([6.964586386687501, 0.7032328649651756]),
        //     Point::new([6.960025700142991, 0.7470222575995122]),
        //     Point::new([6.95518969333792, 0.7907821000035098]),
        //     Point::new([6.950078557571966, 0.8345106611532135]),
        //     Point::new([6.944692495028179, 0.8782062112620722]),
        //     Point::new([6.939031718764988, 0.9218670218493652]),
        //     Point::new([6.9330964527077725, 0.9654913658085751]),
        //     Point::new([6.926886931640004, 1.009077517475708]),
        //     Point::new([6.920403401193958, 1.0526237526975568]),
        //     Point::new([6.913646117841003, 1.0961283488999036]),
        //     Point::new([6.906615348881444, 1.1395895851556601]),
        //     Point::new([6.899311372433959, 1.1830057422529434]),
        //     Point::new([6.891734477424594, 1.226375102763083]),
        //     Point::new([6.883884963575331, 1.2696959511085575]),
        //     Point::new([6.875763141392237, 1.31296657363086]),
        //     Point::new([6.867369332153176, 1.3561852586582834]),
        //     Point::new([6.858703867895106, 1.399350296573632]),
        //     Point::new([6.849767091400937, 1.4424599798818478]),
        //     Point::new([6.840559356185979, 1.485512603277555]),
        //     Point::new([6.831081026483953, 1.528506463712518]),
        //     Point::new([6.821332477232582, 1.571439860463008]),
        //     Point::new([6.811314094058766, 1.614311095197081]),
        //     Point::new([6.8010262732633215, 1.6571184720417582]),
        //     Point::new([6.790469421805305, 1.69986029765011]),
        //     Point::new([6.779643957285919, 1.7425348812682413]),
        //     Point::new([6.768550307931989, 1.785140534802173]),
        //     Point::new([6.757188912579023, 1.8276755728846175]),
        //     Point::new([6.745560220653861, 1.8701383129416491]),
        //     Point::new([6.733664692156884, 1.912527075259261]),
        //     Point::new([6.721502797643827, 1.9548401830498103]),
        //     Point::new([6.709075018207161, 1.9970759625183483]),
        //     Point::new([6.696381845457064, 2.03923274292883]),
        //     Point::new([6.683423781501975, 2.081308856670206]),
        //     Point::new([6.670201338928727, 2.123302639322387]),
        //     Point::new([6.656715040782277, 2.165212429722082]),
        //     Point::new([6.642965420545011, 2.2070365700285173]),
        //     Point::new([6.628953022115643, 2.2487734057890054]),
        //     Point::new([6.614678399787699, 2.290421286004401]),
        //     Point::new([6.6001421182275894, 2.331978563194403]),
        //     Point::new([6.5853447524522775, 2.373443593462726]),
        //     Point::new([6.5702868878065255, 2.4148147365621333]),
        //     Point::new([6.554969119939747, 2.4560903559593132]),
        //     Point::new([6.5393920547824385, 2.497268818899622]),
        //     Point::new([6.523556308522218, 2.5383484964716683]),
        //     Point::new([6.50746250757944, 2.579327763671749]),
        //     Point::new([6.491111288582426, 2.620204999468132]),
        //     Point::new([6.474503298342276, 2.6609785868651765]),
        //     Point::new([6.457639193827281, 2.7016469129673006]),
        //     Point::new([6.440519642136939, 2.742208369042781]),
        //     Point::new([6.423145320475566, 2.7826613505873903]),
        //     Point::new([6.405516916125502, 2.8230042573878684]),
        //     Point::new([6.387635126419933, 2.863235493585221]),
        //     Point::new([6.369500658715298, 2.9033534677378485]),
        //     Point::new([6.3511142303633115, 2.9433565928845002]),
        //     Point::new([6.332476568682587, 2.983243286607047]),
        //     Point::new([6.313588410929867, 3.023011971093081]),
        //     Point::new([6.294450504270857, 3.06266107319833]),
        //     Point::new([6.275063605750672, 3.102189024508883]),
        //     Point::new([6.255428482263886, 3.1415942614032346]),
        //     Point::new([6.235545910524198, 3.180875225114142]),
        //     Point::new([6.215416677033711, 3.220030361790277]),
        //     Point::new([6.1950415780518115, 3.2590581225576996]),
        //     Point::new([6.17442141956368, 3.297956963581125]),
        //     Point::new([6.153557017248403, 3.3367253461249917]),
        //     Point::new([6.132449196446707, 3.375361736614334]),
        //     Point::new([6.111098792128317, 3.4138646066954412]),
        //     Point::new([6.089506648858918, 3.4522324332963206]),
        //     Point::new([6.06767362076675, 3.4904636986869404]),
        //     Point::new([6.045600571508825, 3.528556890539272]),
        //     Point::new([6.023288374236754, 3.5665105019871124]),
        //     Point::new([6.000737911562218, 3.6043230316856882]),
        //     Point::new([5.977950075522045, 3.6419929838710523]),
        //     Point::new([5.95492576754293, 3.6795188684192452]),
        //     Point::new([5.9316658984057735, 3.716899200905242]),
        //     Point::new([5.908171388209652, 3.7541325026616774]),
        //     Point::new([5.884443166335427, 3.7912173008373298]),
        //     Point::new([5.860482171408975, 3.8281521284553914]),
        //     Point::new([5.836289351264061, 3.864935524471491]),
        //     Point::new([5.811865662904846, 3.901566033831495]),
        //     Point::new([5.787212072468025, 3.9380422075290595]),
        //     Point::new([5.762329555184618, 3.9743626026629544]),
        //     Point::new([5.7372190953413815, 4.010525782494139]),
        //     Point::new([5.711881686241882, 4.046530316502595]),
        //     Point::new([5.686318330167202, 4.082374780443913]),
        //     Point::new([5.660530038336286, 4.118057756405635]),
        //     Point::new([5.634517830865944, 4.153577832863343]),
        //     Point::new([5.608282736730502, 4.188933604736487]),
        //     Point::new([5.5818257937210936, 4.224123673443983]),
        //     Point::new([5.555148048404605, 4.259146646959519]),
        //     Point::new([5.528250556082282, 4.294001139866633]),
        //     Point::new([5.501134380747985, 4.328685773413507]),
        //     Point::new([5.473800595046091, 4.363199175567516]),
        //     Point::new([5.446250280229076, 4.397539981069496]),
        //     Point::new([5.418484526114734, 4.431706831487749]),
        //     Point::new([5.390504431043069, 4.465698375271782]),
        //     Point::new([5.362311101832846, 4.499513267805774]),
        //     Point::new([5.3339056537378156, 4.533150171461759]),
        //     Point::new([5.305289210402587, 4.5666077556525355]),
        //     Point::new([5.276462903818187, 4.5998846968843194]),
        //     Point::new([5.247427874277278, 4.632979678809076]),
        //     Point::new([5.218185270329053, 4.665891392276603]),
        //     Point::new([5.188736248733802, 4.698618535386319]),
        //     Point::new([5.159081974417149, 4.731159813538753]),
        //     Point::new([5.129223620423979, 4.763513939486766]),
        //     Point::new([5.099162367872027, 4.795679633386466]),
        //     Point::new([5.068899405905161, 4.827655622847833]),
        //     Point::new([5.038435931646345, 4.859440642985057]),
        //     Point::new([5.007773150150274, 4.8910334364665715]),
        //     Point::new([4.976912274355716, 4.922432753564787]),
        //     Point::new([4.945854525037527, 4.95363735220553]),
        //     Point::new([4.914601130758358, 4.984645998017178]),
        //     Point::new([4.883153327820061, 5.015457464379482]),
        //     Point::new([4.85151236021478, 5.046070532472095]),
        //     Point::new([4.819679479575745, 5.076483991322781]),
        //     Point::new([4.78765594512776, 5.106696637855319]),
        //     Point::new([4.755443023637389, 5.136707276937093]),
        //     Point::new([4.723041989362849, 5.166514721426372]),
        //     Point::new([4.690454124003601, 5.196117792219265]),
        //     Point::new([4.657680716649656, 5.225515318296363]),
        //     Point::new([4.6247230637305705, 5.25470613676907]),
        //     Point::new([4.591582468964174, 5.2836890929255915]),
        //     Point::new([4.558260243304991, 5.312463040276622]),
        //     Point::new([4.5247577048923855, 5.341026840600691]),
        //     Point::new([4.491076178998418, 5.369379363989191]),
        //     Point::new([4.457216997975424, 5.3975194888910725]),
        //     Point::new([4.423181501203305, 5.425446102157211]),
        //     Point::new([4.38897103503655, 5.453158099084438]),
        //     Point::new([4.354586952750973, 5.480654383459242]),
        //     Point::new([4.320030614490187, 5.50793386760113]),
        //     Point::new([4.285303387211796, 5.534995472405656]),
        //     Point::new([4.250406644633322, 5.561838127387106]),
        //     Point::new([4.215341767177867, 5.588460770720841]),
        //     Point::new([4.180110141919504, 5.614862349285306]),
        //     Point::new([4.144713162528408, 5.641041818703684]),
        //     Point::new([4.109152229215731, 5.666998143385208]),
        //     Point::new([4.073428748678206, 5.69273029656613]),
        //     Point::new([4.037544134042508, 5.718237260350337]),
        //     Point::new([4.0014998048093515, 5.743518025749612]),
        //     Point::new([3.965297186797338, 5.768571592723551]),
        //     Point::new([3.9289377120865567, 5.793396970219118]),
        //     Point::new([3.892422818961933, 5.817993176209855]),
        //     Point::new([3.855753951856338, 5.842359237734721]),
        //     Point::new([3.818932561293444, 5.866494190936585]),
        //     Point::new([3.781960103830349, 5.890397081100351]),
        //     Point::new([3.7448380419999623, 5.914066962690724]),
        //     Point::new([3.7075678442531443, 5.9375028993896155]),
        //     Point::new([3.6701509849006233, 5.960703964133178]),
        //     Point::new([3.632588944054672, 5.983669239148481]),
        //     Point::new([3.5948832075705615, 6.006397815989812]),
        //     Point::new([3.557035266987785, 6.028888795574615]),
        //     Point::new([3.5190466194710526, 6.051141288219054]),
        //     Point::new([3.48091876775107, 6.073154413673208]),
        //     Point::new([3.442653220065093, 6.094927301155892]),
        //     Point::new([3.4042514900972685, 6.116459089389099]),
        //     Point::new([3.3657150969187537, 6.137748926632075]),
        //     Point::new([3.3270455649276296, 6.158795970715006]),
        //     Point::new([3.2882444237885964, 6.179599389072337]),
        //     Point::new([3.2493132083724636, 6.200158358775705]),
        //     Point::new([3.210253458695437, 6.220472066566491]),
        //     Point::new([3.1710667198581994, 6.240539708887987]),
        //     Point::new([3.1317545419847885, 6.26036049191719]),
        //     Point::new([3.0923184801612824, 6.279933631596198]),
        //     Point::new([3.052760094374278, 6.299258353663227]),
        //     Point::new([3.0130809494491873, 6.318333893683238]),
        //     Point::new([2.9732826149883356, 6.337159497078177]),
        //     Point::new([2.9333666653088706, 6.355734419156823]),
        //     Point::new([2.893334679380489, 6.374057925144249]),
        //     Point::new([2.853188240762974, 6.392129290210883]),
        //     Point::new([2.8129289375435587, 6.409947799501184]),
        //     Point::new([2.7725583622740984, 6.427512748161918]),
        //     Point::new([2.7320781119080793, 6.444823441370041]),
        //     Point::new([2.6914897877374444, 6.461879194360185]),
        //     Point::new([2.6507949953292522, 6.478679332451745]),
        //     Point::new([2.60999534446216, 6.495223191075565]),
        //     Point::new([2.5690924490627576, 6.511510115800229]),
        //     Point::new([2.5280879271417085, 6.527539462357952]),
        //     Point::new([2.4869834007297564, 6.543310596670056]),
        //     Point::new([2.4457804958135574, 6.558822894872066]),
        //     Point::new([2.404480842271364, 6.574075743338374]),
        //     Point::new([2.3630860738085473, 6.589068538706522]),
        //     Point::new([2.3215978278929734, 6.603800687901068]),
        //     Point::new([2.2800177456902344, 6.618271608157044]),
        //     Point::new([2.2383474719987175, 6.632480727043008]),
        //     Point::new([2.1965886551845513, 6.646427482483693]),
        //     Point::new([2.1547429471163957, 6.660111322782236]),
        //     Point::new([2.1128120031000988, 6.673531706642005]),
        //     Point::new([2.0707974818132175, 6.686688103188008]),
        //     Point::new([2.0287010452394085, 6.699579991987895]),
        //     Point::new([1.9865243586026784, 6.712206863072548]),
        //     Point::new([1.944269090301515, 6.724568216956246]),
        //     Point::new([1.9019369118428893, 6.736663564656434]),
        //     Point::new([1.8595294977761354, 6.748492427713054]),
        //     Point::new([1.817048525626711, 6.760054338207482]),
        //     Point::new([1.7744956758298334, 6.771348838781031]),
        //     Point::new([1.7318726316640176, 6.7823754826530465]),
        //     Point::new([1.689181079184476, 6.793133833638579]),
        //     Point::new([1.646422707156431, 6.803623466165635]),
        //     Point::new([1.6035992069883112, 6.813843965292019]),
        //     Point::new([1.560712272664842, 6.823794926721739]),
        //     Point::new([1.5177636006800346, 6.833475956821007]),
        //     Point::new([1.4747548899700849, 6.842886672633803]),
        //     Point::new([1.431687841846157, 6.852026701897031]),
        //     Point::new([1.3885641599270913, 6.860895683055236]),
        //     Point::new([1.3453855500720115, 6.869493265274916]),
        //     Point::new([1.3021537203128457, 6.877819108458394]),
        //     Point::new([1.2588703807867592, 6.885872883257271]),
        //     Point::new([1.2155372436685128, 6.893654271085456]),
        //     Point::new([1.1721560231027246, 6.90116296413177]),
        //     Point::new([1.128728435136067, 6.90839866537212]),
        //     Point::new([1.0852561976493862, 6.915361088581246]),
        //     Point::new([1.0417410302897447, 6.92204995834405]),
        //     Point::new([0.9981846544023978, 6.928465010066484]),
        //     Point::new([0.9545887929626999, 6.9346059899860215]),
        //     Point::new([0.9109551705079546, 6.9404726551816935]),
        //     Point::new([0.8672855130691866, 6.946064773583696]),
        //     Point::new([0.8235815481028712, 6.951382123982574]),
        //     Point::new([0.7798450044225994, 6.95642449603797]),
        //     Point::new([0.7360776121306897, 6.961191690286943]),
        //     Point::new([0.6922811025497491, 6.965683518151862]),
        //     Point::new([0.6484572081541919, 6.9698998019478635]),
        //     Point::new([0.6046076625016997, 6.97384037488988]),
        //     Point::new([0.5607342001646507, 6.977505081099239]),
        //     Point::new([0.516838556661505, 6.980893775609829]),
        //     Point::new([0.47292246838815033, 6.98400632437383]),
        //     Point::new([0.42898767254921466, 6.986842604267024]),
        //     Point::new([0.3850359070893521, 6.989402503093657]),
        //     Point::new([0.34106891062448597, 6.991685919590884]),
        //     Point::new([0.29708842237303923, 6.99369276343277]),
        //     Point::new([0.25309618208713447, 6.995422955233867]),
        //     Point::new([0.2090939299837737, 6.996876426552347]),
        //     Point::new([0.16508340667600016, 6.998053119892721]),
        //     Point::new([0.12106635310404236, 6.998952988708103]),
        //     Point::new([0.07704451046645368, 6.999575997402056]),
        //     Point::new([0.033019620151225566, 6.999922121329998]),
        //     Point::new([-0.0110065763330909, 6.999991346800182]),
        //     Point::new([-0.055032337426276, 6.999783671074228]),
        //     Point::new([-0.09905592158533297, 6.999299102367242]),
        //     Point::new([-0.14307558735338033, 6.998537659847483]),
        //     Point::new([-0.1870895934285332, 6.997499373635609]),
        //     Point::new([-0.2310961987327939, 6.996184284803485]),
        //     Point::new([-0.27509366248091804, 6.994592445372556]),
        //     Point::new([-0.3190802442492773, 6.992723918311793]),
        //     Point::new([-0.3630542040447056, 6.9905787775351955]),
        //     Point::new([-0.40701380237332885, 6.988157107898878]),
        //     Point::new([-0.4509573003093764, 6.985459005197702]),
        //     Point::new([-0.4948829595639617, 6.9824845761614975]),
        //     Point::new([-0.5387890425538546, 6.9792339384508315]),
        //     Point::new([-0.582673812470208, 6.975707220652358]),
        //     Point::new([-0.6265355333472646, 6.971904562273731]),
        //     Point::new([-0.6703724701310259, 6.967826113738088]),
        //     Point::new([-0.7141828887478882, 6.963472036378097]),
        //     Point::new([-0.7579650561732321, 6.958842502429575]),
        //     Point::new([-0.8017172404999854, 6.953937695024675]),
        //     Point::new([-0.8454377110071267, 6.9487578081846415]),
        //     Point::new([-0.8891247382281497, 6.943303046812138]),
        //     Point::new([-0.9327765940194761, 6.937573626683138]),
        //     Point::new([-0.9763915516288171, 6.93156977443839]),
        //     Point::new([-1.0199678857634793, 6.9252917275744545]),
        //     Point::new([-1.0635038726586084, 6.918739734434309]),
        //     Point::new([-1.1069977901453845, 6.911914054197522]),
        //     Point::new([-1.1504479177191416, 6.904814956870003]),
        //     Point::new([-1.1938525366074262, 6.897442723273316]),
        //     Point::new([-1.2372099298379897, 6.889797645033581]),
        //     Point::new([-1.280518382306706, 6.88188002456993]),
        //     Point::new([-1.3237761808454132, 6.8736901750825465]),
        //     Point::new([-1.366981614289689, 6.865228420540278]),
        //     Point::new([-1.410132973546535, 6.856495095667816]),
        //     Point::new([-1.4532285516619852, 6.84749054593246]),
        //     Point::new([-1.4962666438886276, 6.838215127530448]),
        //     Point::new([-1.5392455477530431, 6.828669207372871]),
        //     Point::new([-1.5821635631231425, 6.818853163071154]),
        //     Point::new([-1.6250189922754301, 6.808767382922121]),
        //     Point::new([-1.6678101399621539, 6.798412265892634]),
        //     Point::new([-1.710535313478367, 6.7877882216038135]),
        //     Point::new([-1.7531928227288878, 6.776895670314832]),
        //     Point::new([-1.7957809802951537, 6.76573504290629]),
        //     Point::new([-1.8382981015019728, 6.754306780863174]),
        //     Point::new([-1.880742504484161, 6.742611336257388]),
        //     Point::new([-1.923112510253079, 6.730649171729879]),
        //     Point::new([-1.9654064427630438, 6.718420760472323]),
        //     Point::new([-2.00762262897763, 6.705926586208422]),
        //     Point::new([-2.0497593989358505, 6.693167143174757]),
        //     Point::new([-2.091815085818218, 6.680142936101242]),
        //     Point::new([-2.133788026012671, 6.6668544801911604]),
        //     Point::new([-2.1756765591803973, 6.653302301100781]),
        //     Point::new([-2.217479028321498, 6.639486934918567]),
        //     Point::new([-2.2591937798405435, 6.625408928143968]),
        //     Point::new([-2.30081916361198, 6.611068837665803]),
        //     Point::new([-2.3423535330454097, 6.59646723074023]),
        //     Point::new([-2.3837952451507145, 6.581604684968313]),
        //     Point::new([-2.4251426606030635, 6.566481788273161]),
        //     Point::new([-2.466394143807749, 6.551099138876685]),
        //     Point::new([-2.5075480629648905, 6.5354573452759235]),
        //     Point::new([-2.5486027901339847, 6.51955702621898]),
        //     Point::new([-2.5895567012983007, 6.50339881068054]),
        //     Point::new([-2.6304081764291247, 6.486983337836995]),
        //     Point::new([-2.6711555995498384, 6.470311257041159]),
        //     Point::new([-2.71179735879985, 6.453383227796577]),
        //     Point::new([-2.7523318464983513, 6.436199919731438]),
        //     Point::new([-2.7927574592079125, 6.4187620125720946]),
        //     Point::new([-2.8330725977979125, 6.4010701961161605]),
        //     Point::new([-2.873275667507796, 6.383125170205236]),
        //     Point::new([-2.9133650780101528, 6.36492764469722]),
        //     Point::new([-2.9533392434736343, 6.346478339438226]),
        //     Point::new([-2.9931965826256826, 6.327777984234113]),
        //     Point::new([-3.032935518815078, 6.308827318821614]),
        //     Point::new([-3.072554480074312, 6.289627092839072]),
        //     Point::new([-3.1120518991817665, 6.270178065796789]),
        //     Point::new([-3.151426213723711, 6.2504810070469805]),
        //     Point::new([-3.190675866156106, 6.230536695753343]),
        //     Point::new([-3.229799303866218, 6.21034592086023]),
        //     Point::new([-3.2687949792340363, 6.189909481061444]),
        //     Point::new([-3.307661349693483, 6.16922818476865]),
        //     Point::new([-3.3463968777934516, 6.148302850079381]),
        //     Point::new([-3.3850000312586097, 6.127134304744691]),
        //     Point::new([-3.423469283050017, 6.105723386136404]),
        //     Point::new([-3.4618031114255303, 6.084070941213993]),
        //     Point::new([-3.4999999999999982, 6.062177826491071]),
        //     Point::new([-3.53805843780525, 6.040044908001518]),
        //     Point::new([-3.5759769193498583, 6.017673061265218]),
        //     Point::new([-3.6137539446786993, 5.995063171253422]),
        //     Point::new([-3.6513880194322823, 5.972216132353751]),
        //     Point::new([-3.6888776549058644, 5.9491328483348065]),
        //     Point::new([-3.726221368108342, 5.925814232310425]),
        //     Point::new([-3.7634176818209104, 5.902261206703557]),
        //     Point::new([-3.8004651246554952, 5.878474703209778]),
        //     Point::new([-3.8373622311129676, 5.854455662760433]),
        //     Point::new([-3.874107541641109, 5.830205035485415]),
        //     Point::new([-3.9106996026923433, 5.80572378067558]),
        //     Point::new([-3.9471369667812435, 5.781012866744803]),
        //     Point::new([-3.9834181925417855, 5.756073271191667]),
        //     Point::new([-4.019541844784363, 5.730905980560797]),
        //     Point::new([-4.055506494552567, 5.705511990403837]),
        //     Point::new([-4.0913107191797025, 5.679892305240061]),
        //     Point::new([-4.126953102345073, 5.6540479385166496]),
        //     Point::new([-4.162432234130003, 5.627979912568585]),
        //     Point::new([-4.197746711073614, 5.601689258578224]),
        //     Point::new([-4.232895136228327, 5.575177016534504]),
        //     Point::new([-4.267876119215148, 5.548444235191794]),
        //     Point::new([-4.302688276278646, 5.521491972028421]),
        //     Point::new([-4.3373302303417, 5.494321293204832]),
        //     Point::new([-4.371800611059971, 5.466933273521423]),
        //     Point::new([-4.406098054876109, 5.4393289963760205]),
        //     Point::new([-4.4402212050736924, 5.411509553721025]),
        //     Point::new([-4.474168711830893, 5.383476046020219]),
        //     Point::new([-4.507939232273876, 5.355229582205231]),
        //     Point::new([-4.541531430529919, 5.326771279631675]),
        //     Point::new([-4.5749439777802525, 5.298102264034944]),
        //     Point::new([-4.6081755523126295, 5.269223669485686]),
        //     Point::new([-4.641224839573606, 5.240136638344934]),
        //     Point::new([-4.674090532220538, 5.210842321218933]),
        //     Point::new([-4.706771330173306, 5.1813418769136055]),
        //     Point::new([-4.739265940665734, 5.1516364723887245]),
        //     Point::new([-4.771573078296729, 5.121727282711749]),
        //     Point::new([-4.803691465081134, 5.091615491011342]),
        //     Point::new([-4.835619830500275, 5.061302288430566]),
        //     Point::new([-4.867356911552224, 5.03078887407977]),
        //     Point::new([-4.898901452801756, 5.000076454989148]),
        //     Point::new([-4.930252206430013, 4.969166246061002]),
        //     Point::new([-4.9614079322838665, 4.938059470021673]),
        //     Point::new([-4.992367397924969, 4.906757357373184]),
        //     Point::new([-5.0231293786785125, 4.875261146344554]),
        //     Point::new([-5.053692657681671, 4.843572082842823]),
        //     Point::new([-5.084056025931729, 4.811691420403773]),
        //     Point::new([-5.11421828233392, 4.779620420142324]),
        //     Point::new([-5.144178233748931, 4.747360350702666]),
        //     Point::new([-5.1739346950400975, 4.714912488208062]),
        //     Point::new([-5.20348648912029, 4.682278116210378]),
        //     Point::new([-5.232832446998474, 4.649458525639299]),
        //     Point::new([-5.261971407825952, 4.616455014751272]),
        //     Point::new([-5.290902218942282, 4.583268889078147]),
        //     Point::new([-5.319623735920881, 4.549901461375528]),
        //     Point::new([-5.348134822614284, 4.516354051570856]),
        //     Point::new([-5.376434351199095, 4.482627986711184]),
        //     Point::new([-5.404521202220601, 4.448724600910691]),
        //     Point::new([-5.432394264637053, 4.4146452352979]),
        //     Point::new([-5.460052435863608, 4.380391237962641]),
        //     Point::new([-5.4874946218159595, 4.345963963902706]),
        //     Point::new([-5.514719736953607, 4.311364774970258]),
        //     Point::new([-5.541726704322796, 4.276595039817962]),
        //     Point::new([-5.568514455599129, 4.241656133844838]),
        //     Point::new([-5.59508193112981, 4.206549439141861]),
        //     Point::new([-5.621428079975578, 4.171276344437286]),
        //     Point::new([-5.647551859952268, 4.135838245041708]),
        //     Point::new([-5.673452237672043, 4.100236542792878]),
        //     Point::new([-5.699128188584268, 4.064472646000241]),
        //     Point::new([-5.724578697016042, 4.0285479693892325]),
        //     Point::new([-5.749802756212375, 3.992463934045313]),
        //     Point::new([-5.774799368376011, 3.9562219673577506]),
        //     Point::new([-5.799567544706894, 3.919823502963168]),
        //     Point::new([-5.824106305441295, 3.8832699806888207]),
        //     Point::new([-5.848414679890554, 3.846562846495644]),
        //     Point::new([-5.87249170647949, 3.8097035524210545]),
        //     Point::new([-5.896336432784431, 3.7726935565215167]),
        //     Point::new([-5.919947915570892, 3.735534322814858]),
        //     Point::new([-5.943325220830884, 3.698227321222362]),
        //     Point::new([-5.966467423819867, 3.6607740275106195]),
        //     Point::new([-5.989373609093325, 3.623175923233151]),
        //     Point::new([-6.012042870542981, 3.5854344956718025]),
        //     Point::new([-6.034474311432638, 3.5475512377779106]),
        //     Point::new([-6.056667044433656, 3.5095276481132442]),
        //     Point::new([-6.078620191660048, 3.4713652307907266]),
        //     Point::new([-6.100332884703207, 3.433065495414944]),
        //     Point::new([-6.121804264666259, 3.394629957022415]),
        //     Point::new([-6.143033482198043, 3.3560601360216697]),
        //     Point::new([-6.164019697526702, 3.3173575581331036]),
        //     Point::new([-6.184762080492906, 3.2785237543286256]),
        //     Point::new([-6.20525981058269, 3.239560260771095]),
        //     Point::new([-6.2255120769599115, 3.200468618753556]),
        //     Point::new([-6.245518078498325, 3.161250374638268]),
        //     Point::new([-6.265277023813274, 3.1219070797955344]),
        //     Point::new([-6.284788131292992, 3.0824402905423383]),
        //     Point::new([-6.304050629129527, 3.0428515680807724]),
        //     Point::new([-6.323063755349266, 3.0031424784362857]),
        //     Point::new([-6.341826757843079, 2.963314592395742]),
        //     Point::new([-6.360338894396073, 2.9233694854452694]),
        //     Point::new([-6.378599432716949, 2.88330873770795]),
        //     Point::new([-6.396607650466972, 2.843133933881308]),
        //     Point::new([-6.414362835288541, 2.8028466631746287]),
        //     Point::new([-6.431864284833371, 2.7624485192460875]),
        //     Point::new([-6.449111306790278, 2.7219411001397145]),
        //     Point::new([-6.466103218912558, 2.681326008222176]),
        //     Point::new([-6.482839349044981, 2.640604850119392]),
        //     Point::new([-6.499319035150379, 2.5997792366529797]),
        //     Point::new([-6.515541625335833, 2.558850782776536]),
        //     Point::new([-6.531506477878461, 2.5178211075117534]),
        //     Point::new([-6.5472129612508, 2.4766918338843715]),
        //     Point::new([-6.562660454145792, 2.435464588859988]),
        //     Point::new([-6.5778483455013586, 2.394141003279682]),
        //     Point::new([-6.592776034524573, 2.3527227117955136]),
        //     Point::new([-6.60744293071543, 2.311211352805859]),
        //     Point::new([-6.6218484538901965, 2.2696085683905984]),
        //     Point::new([-6.635992034204367, 2.2279160042461617]),
        //     Point::new([-6.649873112175209, 2.1861353096204286]),
        //     Point::new([-6.663491138703884, 2.1442681372474865]),
        //     Point::new([-6.6768455750971825, 2.102316143282256]),
        //     Point::new([-6.689935893088817, 2.0602809872349757]),
        //     Point::new([-6.702761574860336, 2.018164331905556]),
        //     Point::new([-6.715322113061591, 1.9759678433178054]),
        //     Point::new([-6.727617010830819, 1.9336931906535213]),
        //     Point::new([-6.73964578181429, 1.8913420461864756]),
        //     Point::new([-6.751407950185549, 1.8489160852162452]),
        //     Point::new([-6.762903050664235, 1.806416986001953]),
        //     Point::new([-6.7741306285344915, 1.7638464296958778]),
        //     Point::new([-6.785090239662948, 1.7212061002769539]),
        //     Point::new([-6.7957814505162935, 1.678497684484154]),
        //     Point::new([-6.806203838178423, 1.6357228717497694]),
        //     Point::new([-6.816356990367168, 1.5928833541325793]),
        //     Point::new([-6.826240505450604, 1.5499808262509167]),
        //     Point::new([-6.835853992462945, 1.5070169852156354]),
        //     Point::new([-6.845197071119995, 1.463993530562975]),
        //     Point::new([-6.854269371834206, 1.420912164187333]),
        //     Point::new([-6.863070535729287, 1.377774590273939]),
        //     Point::new([-6.871600214654406, 1.334582515231453]),
        //     Point::new([-6.8798580711979636, 1.2913376476244478]),
        //     Point::new([-6.887843778700931, 1.2480416981058333]),
        //     Point::new([-6.895557021269781, 1.204696379349185]),
        //     Point::new([-6.90299749378898, 1.1613034059809955]),
        //     Point::new([-6.91016490193306, 1.1178644945128484]),
        //     Point::new([-6.917058962178256, 1.0743813632735164]),
        //     Point::new([-6.923679401813729, 1.0308557323409908]),
        //     Point::new([-6.9300259589523465, 0.9872893234744382]),
        //     Point::new([-6.936098382541045, 0.943683860046093]),
        //     Point::new([-6.941896432370765, 0.9000410669730843]),
        //     Point::new([-6.947419879085945, 0.8563626706492011]),
        //     Point::new([-6.952668504193602, 0.8126503988766125]),
        //     Point::new([-6.957642100071966, 0.7689059807975013]),
        //     Point::new([-6.962340469978704, 0.7251311468256766]),
        //     Point::new([-6.966763428058694, 0.68132762857812]),
        //     Point::new([-6.970910799351378, 0.6374971588064876]),
        //     Point::new([-6.974782419797686, 0.5936414713285665]),
        //     Point::new([-6.9783781362465245, 0.5497623009596898]),
        //     Point::new([-6.981697806460833, 0.5058613834441117]),
        //     Point::new([-6.984741299123215, 0.4619404553863464]),
        //     Point::new([-6.987508493841126, 0.41800125418247236]),
        //     Point::new([-6.989999281151642, 0.3740455179514054]),
        //     Point::new([-6.992213562525785, 0.33007498546614333]),
        //     Point::new([-6.994151250372424, 0.28609139608498185]),
        //     Point::new([-6.995812268041739, 0.24209648968272224]),
        //     Point::new([-6.997196549828251, 0.19809200658182793]),
        //     Point::new([-6.998304040973425, 0.1540796874835945]),
        //     Point::new([-6.999134697667828, 0.11006127339928878]),
        //     Point::new([-6.999688487052877, 0.06603850558127933]),
        //     Point::new([-6.9999653872221215, 0.022013125454156814]),
        //     Point::new([-6.9999653872221215, -0.02201312545415199]),
        //     Point::new([-6.999688487052877, -0.0660385055812745]),
        //     Point::new([-6.999134697667828, -0.11006127339928395]),
        //     Point::new([-6.998304040973425, -0.15407968748358966]),
        //     Point::new([-6.997196549828252, -0.19809200658182313]),
        //     Point::new([-6.99581226804174, -0.24209648968271744]),
        //     Point::new([-6.994151250372425, -0.28609139608498013]),
        //     Point::new([-6.9922135625257855, -0.3300749854661385]),
        //     Point::new([-6.989999281151642, -0.37404551795140056]),
        //     Point::new([-6.987508493841126, -0.4180012541824676]),
        //     Point::new([-6.984741299123215, -0.4619404553863416]),
        //     Point::new([-6.981697806460834, -0.5058613834441068]),
        //     Point::new([-6.9783781362465245, -0.5497623009596849]),
        //     Point::new([-6.974782419797686, -0.5936414713285617]),
        //     Point::new([-6.970910799351378, -0.6374971588064828]),
        //     Point::new([-6.966763428058694, -0.6813276285781154]),
        //     Point::new([-6.962340469978704, -0.7251311468256718]),
        //     Point::new([-6.957642100071967, -0.7689059807974965]),
        //     Point::new([-6.952668504193602, -0.8126503988766077]),
        //     Point::new([-6.947419879085945, -0.8563626706491995]),
        //     Point::new([-6.941896432370766, -0.9000410669730794]),
        //     Point::new([-6.936098382541046, -0.9436838600460881]),
        //     Point::new([-6.930025958952347, -0.9872893234744335]),
        //     Point::new([-6.92367940181373, -1.030855732340986]),
        //     Point::new([-6.917058962178257, -1.0743813632735115]),
        //     Point::new([-6.910164901933061, -1.1178644945128435]),
        //     Point::new([-6.902997493788981, -1.1613034059809908]),
        //     Point::new([-6.895557021269781, -1.2046963793491803]),
        //     Point::new([-6.8878437787009315, -1.2480416981058284]),
        //     Point::new([-6.879858071197964, -1.291337647624443]),
        //     Point::new([-6.871600214654408, -1.3345825152314483]),
        //     Point::new([-6.863070535729288, -1.377774590273934]),
        //     Point::new([-6.8542693718342065, -1.4209121641873281]),
        //     Point::new([-6.845197071119996, -1.46399353056297]),
        //     Point::new([-6.835853992462946, -1.5070169852156305]),
        //     Point::new([-6.826240505450606, -1.5499808262509123]),
        //     Point::new([-6.816356990367169, -1.5928833541325746]),
        //     Point::new([-6.806203838178423, -1.6357228717497647]),
        //     Point::new([-6.795781450516294, -1.6784976844841493]),
        //     Point::new([-6.785090239662949, -1.7212061002769492]),
        //     Point::new([-6.774130628534493, -1.7638464296958731]),
        //     Point::new([-6.762903050664237, -1.8064169860019486]),
        //     Point::new([-6.75140795018555, -1.8489160852162405]),
        //     Point::new([-6.739645781814292, -1.891342046186471]),
        //     Point::new([-6.72761701083082, -1.9336931906535169]),
        //     Point::new([-6.715322113061592, -1.9759678433178007]),
        //     Point::new([-6.702761574860336, -2.0181643319055516]),
        //     Point::new([-6.689935893088819, -2.0602809872349708]),
        //     Point::new([-6.676845575097183, -2.1023161432822515]),
        //     Point::new([-6.663491138703886, -2.144268137247482]),
        //     Point::new([-6.64987311217521, -2.186135309620424]),
        //     Point::new([-6.635992034204369, -2.2279160042461577]),
        //     Point::new([-6.621848453890197, -2.2696085683905936]),
        //     Point::new([-6.607442930715432, -2.3112113528058544]),
        //     Point::new([-6.592776034524576, -2.3527227117955096]),
        //     Point::new([-6.577848345501359, -2.3941410032796773]),
        //     Point::new([-6.562660454145794, -2.4354645888599835]),
        //     Point::new([-6.547212961250801, -2.4766918338843698]),
        //     Point::new([-6.531506477878462, -2.5178211075117485]),
        //     Point::new([-6.515541625335835, -2.5588507827765317]),
        //     Point::new([-6.49931903515038, -2.5997792366529753]),
        //     Point::new([-6.482839349044982, -2.6406048501193875]),
        //     Point::new([-6.466103218912559, -2.6813260082221717]),
        //     Point::new([-6.449111306790279, -2.72194110013971]),
        //     Point::new([-6.431864284833374, -2.7624485192460835]),
        //     Point::new([-6.414362835288543, -2.8028466631746243]),
        //     Point::new([-6.396607650466974, -2.8431339338813038]),
        //     Point::new([-6.3785994327169515, -2.8833087377079454]),
        //     Point::new([-6.360338894396075, -2.923369485445265]),
        //     Point::new([-6.341826757843082, -2.9633145923957374]),
        //     Point::new([-6.323063755349267, -3.003142478436284]),
        //     Point::new([-6.304050629129529, -3.042851568080768]),
        //     Point::new([-6.284788131292995, -3.082440290542334]),
        //     Point::new([-6.265277023813276, -3.1219070797955304]),
        //     Point::new([-6.2455180784983275, -3.1612503746382634]),
        //     Point::new([-6.225512076959913, -3.2004686187535514]),
        //     Point::new([-6.205259810582692, -3.2395602607710905]),
        //     Point::new([-6.184762080492908, -3.278523754328621]),
        //     Point::new([-6.1640196975267045, -3.317357558133099]),
        //     Point::new([-6.143033482198046, -3.3560601360216653]),
        //     Point::new([-6.121804264666261, -3.394629957022411]),
        //     Point::new([-6.100332884703208, -3.43306549541494]),
        //     Point::new([-6.07862019166005, -3.471365230790723]),
        //     Point::new([-6.0566670444336586, -3.5095276481132403]),
        //     Point::new([-6.034474311432641, -3.5475512377779066]),
        //     Point::new([-6.012042870542983, -3.585434495671799]),
        //     Point::new([-5.989373609093327, -3.6231759232331475]),
        //     Point::new([-5.96646742381987, -3.6607740275106155]),
        //     Point::new([-5.943325220830886, -3.6982273212223573]),
        //     Point::new([-5.919947915570893, -3.7355343228148543]),
        //     Point::new([-5.896336432784434, -3.772693556521512]),
        //     Point::new([-5.872491706479494, -3.8097035524210505]),
        //     Point::new([-5.848414679890557, -3.846562846495639]),
        //     Point::new([-5.824106305441298, -3.8832699806888167]),
        //     Point::new([-5.799567544706897, -3.919823502963164]),
        //     Point::new([-5.774799368376013, -3.9562219673577457]),
        //     Point::new([-5.749802756212378, -3.9924639340453085]),
        //     Point::new([-5.724578697016046, -4.028547969389228]),
        //     Point::new([-5.699128188584271, -4.064472646000237]),
        //     Point::new([-5.673452237672046, -4.1002365427928735]),
        //     Point::new([-5.647551859952271, -4.135838245041704]),
        //     Point::new([-5.621428079975581, -4.171276344437281]),
        //     Point::new([-5.595081931129813, -4.206549439141858]),
        //     Point::new([-5.568514455599131, -4.241656133844835]),
        //     Point::new([-5.5417267043228, -4.276595039817957]),
        //     Point::new([-5.51471973695361, -4.311364774970254]),
        //     Point::new([-5.487494621815962, -4.345963963902702]),
        //     Point::new([-5.46005243586361, -4.380391237962638]),
        //     Point::new([-5.432394264637054, -4.4146452352978995]),
        //     Point::new([-5.404521202220604, -4.448724600910688]),
        //     Point::new([-5.376434351199099, -4.482627986711181]),
        //     Point::new([-5.348134822614287, -4.516354051570852]),
        //     Point::new([-5.319623735920884, -4.549901461375525]),
        //     Point::new([-5.290902218942286, -4.583268889078143]),
        //     Point::new([-5.261971407825954, -4.61645501475127]),
        //     Point::new([-5.232832446998477, -4.649458525639296]),
        //     Point::new([-5.2034864891202925, -4.682278116210374]),
        //     Point::new([-5.1739346950401, -4.714912488208058]),
        //     Point::new([-5.144178233748933, -4.747360350702661]),
        //     Point::new([-5.1142182823339235, -4.7796204201423205]),
        //     Point::new([-5.084056025931732, -4.811691420403768]),
        //     Point::new([-5.0536926576816725, -4.843572082842822]),
        //     Point::new([-5.023129378678516, -4.875261146344551]),
        //     Point::new([-4.992367397924973, -4.906757357373181]),
        //     Point::new([-4.961407932283869, -4.9380594700216704]),
        //     Point::new([-4.9302522064300165, -4.969166246060998]),
        //     Point::new([-4.898901452801759, -5.000076454989145]),
        //     Point::new([-4.8673569115522275, -5.030788874079766]),
        //     Point::new([-4.835619830500279, -5.061302288430563]),
        //     Point::new([-4.803691465081138, -5.091615491011339]),
        //     Point::new([-4.771573078296733, -5.1217272827117455]),
        //     Point::new([-4.739265940665737, -5.151636472388721]),
        //     Point::new([-4.706771330173311, -5.181341876913602]),
        //     Point::new([-4.674090532220543, -5.21084232121893]),
        //     Point::new([-4.641224839573608, -5.240136638344934]),
        //     Point::new([-4.608175552312632, -5.269223669485682]),
        //     Point::new([-4.574943977780256, -5.298102264034941]),
        //     Point::new([-4.5415314305299255, -5.326771279631669]),
        //     Point::new([-4.5079392322738805, -5.3552295822052285]),
        //     Point::new([-4.474168711830899, -5.383476046020213]),
        //     Point::new([-4.440221205073696, -5.4115095537210225]),
        //     Point::new([-4.406098054876111, -5.43932899637602]),
        //     Point::new([-4.371800611059975, -5.46693327352142]),
        //     Point::new([-4.337330230341701, -5.49432129320483]),
        //     Point::new([-4.30268827627865, -5.521491972028417]),
        //     Point::new([-4.267876119215149, -5.548444235191793]),
        //     Point::new([-4.232895136228331, -5.5751770165345]),
        //     Point::new([-4.197746711073615, -5.601689258578223]),
        //     Point::new([-4.16243223413001, -5.6279799125685805]),
        //     Point::new([-4.126953102345078, -5.654047938516646]),
        //     Point::new([-4.091310719179709, -5.679892305240057]),
        //     Point::new([-4.055506494552571, -5.705511990403834]),
        //     Point::new([-4.01954184478437, -5.730905980560793]),
        //     Point::new([-3.983418192541789, -5.756073271191664]),
        //     Point::new([-3.9471369667812453, -5.781012866744802]),
        //     Point::new([-3.9106996026923473, -5.805723780675577]),
        //     Point::new([-3.87410754164111, -5.830205035485414]),
        //     Point::new([-3.8373622311129716, -5.854455662760431]),
        //     Point::new([-3.8004651246554966, -5.878474703209777]),
        //     Point::new([-3.7634176818209144, -5.902261206703554]),
        //     Point::new([-3.7262213681083454, -5.925814232310423]),
        //     Point::new([-3.6888776549058715, -5.949132848334802]),
        //     Point::new([-3.651388019432286, -5.972216132353749]),
        //     Point::new([-3.613753944678706, -5.995063171253419]),
        //     Point::new([-3.5759769193498623, -6.017673061265215]),
        //     Point::new([-3.5380584378052564, -6.040044908001515]),
        //     Point::new([-3.500000000000003, -6.062177826491069]),
        //     Point::new([-3.461803111425531, -6.084070941213993]),
        //     Point::new([-3.423469283050021, -6.105723386136402]),
        //     Point::new([-3.3850000312586115, -6.12713430474469]),
        //     Point::new([-3.346396877793456, -6.148302850079379]),
        //     Point::new([-3.3076613496934844, -6.169228184768649]),
        //     Point::new([-3.2687949792340403, -6.189909481061442]),
        //     Point::new([-3.2297993038662223, -6.210345920860227]),
        //     Point::new([-3.190675866156113, -6.230536695753339]),
        //     Point::new([-3.1514262137237155, -6.250481007046979]),
        //     Point::new([-3.1120518991817736, -6.270178065796785]),
        //     Point::new([-3.0725544800743165, -6.289627092839069]),
        //     Point::new([-3.0329355188150853, -6.30882731882161]),
        //     Point::new([-2.993196582625687, -6.327777984234111]),
        //     Point::new([-2.953339243473636, -6.346478339438225]),
        //     Point::new([-2.913365078010157, -6.364927644697218]),
        //     Point::new([-2.8732756675077975, -6.383125170205235]),
        //     Point::new([-2.8330725977979183, -6.401070196116159]),
        //     Point::new([-2.792757459207916, -6.418762012572093]),
        //     Point::new([-2.7523318464983575, -6.436199919731436]),
        //     Point::new([-2.711797358799853, -6.453383227796575]),
        //     Point::new([-2.671155599549844, -6.470311257041157]),
        //     Point::new([-2.630408176429128, -6.486983337836994]),
        //     Point::new([-2.5895567012983083, -6.503398810680537]),
        //     Point::new([-2.548602790133989, -6.519557026218978]),
        //     Point::new([-2.507548062964898, -6.535457345275921]),
        //     Point::new([-2.4663941438077535, -6.551099138876682]),
        //     Point::new([-2.4251426606030653, -6.56648178827316]),
        //     Point::new([-2.3837952451507194, -6.581604684968312]),
        //     Point::new([-2.342353533045411, -6.59646723074023]),
        //     Point::new([-2.3008191636119864, -6.611068837665801]),
        //     Point::new([-2.259193779840546, -6.625408928143966]),
        //     Point::new([-2.217479028321504, -6.639486934918565]),
        //     Point::new([-2.1756765591804, -6.65330230110078]),
        //     Point::new([-2.1337880260126774, -6.666854480191158]),
        //     Point::new([-2.091815085818221, -6.680142936101241]),
        //     Point::new([-2.0497593989358585, -6.6931671431747555]),
        //     Point::new([-2.0076226289776344, -6.705926586208421]),
        //     Point::new([-1.9654064427630453, -6.718420760472323]),
        //     Point::new([-1.9231125102530833, -6.730649171729877]),
        //     Point::new([-1.8807425044841626, -6.742611336257388]),
        //     Point::new([-1.8382981015019775, -6.754306780863172]),
        //     Point::new([-1.7957809802951568, -6.765735042906289]),
        //     Point::new([-1.753192822728894, -6.77689567031483]),
        //     Point::new([-1.7105353134783703, -6.7877882216038135]),
        //     Point::new([-1.6678101399621599, -6.798412265892632]),
        //     Point::new([-1.6250189922754332, -6.80876738292212]),
        //     Point::new([-1.5821635631231488, -6.818853163071152]),
        //     Point::new([-1.5392455477530465, -6.8286692073728705]),
        //     Point::new([-1.4962666438886354, -6.838215127530447]),
        //     Point::new([-1.4532285516619898, -6.847490545932458]),
        //     Point::new([-1.4101329735465367, -6.856495095667816]),
        //     Point::new([-1.3669816142896938, -6.8652284205402765]),
        //     Point::new([-1.3237761808454147, -6.8736901750825465]),
        //     Point::new([-1.2805183823067108, -6.881880024569929]),
        //     Point::new([-1.2372099298379928, -6.889797645033581]),
        //     Point::new([-1.1938525366074326, -6.897442723273314]),
        //     Point::new([-1.150447917719145, -6.904814956870002]),
        //     Point::new([-1.106997790145391, -6.9119140541975215]),
        //     Point::new([-1.0635038726586117, -6.918739734434309]),
        //     Point::new([-1.0199678857634855, -6.925291727574454]),
        //     Point::new([-0.9763915516288217, -6.9315697744383895]),
        //     Point::new([-0.9327765940194841, -6.937573626683136]),
        //     Point::new([-0.8891247382281544, -6.9433030468121375]),
        //     Point::new([-0.8454377110071284, -6.9487578081846415]),
        //     Point::new([-0.8017172404999902, -6.953937695024675]),
        //     Point::new([-0.7579650561732337, -6.958842502429574]),
        //     Point::new([-0.7141828887478929, -6.963472036378096]),
        //     Point::new([-0.6703724701310292, -6.967826113738088]),
        //     Point::new([-0.6265355333472711, -6.97190456227373]),
        //     Point::new([-0.5826738124702113, -6.975707220652357]),
        //     Point::new([-0.5387890425538608, -6.979233938450831]),
        //     Point::new([-0.494882959563965, -6.9824845761614975]),
        //     Point::new([-0.4509573003093827, -6.985459005197702]),
        //     Point::new([-0.4070138023733337, -6.988157107898878]),
        //     Point::new([-0.3630542040447135, -6.990578777535195]),
        //     Point::new([-0.3190802442492821, -6.992723918311792]),
        //     Point::new([-0.2750936624809198, -6.994592445372556]),
        //     Point::new([-0.2310961987327987, -6.996184284803485]),
        //     Point::new([-0.18708959342853493, -6.997499373635609]),
        //     Point::new([-0.14307558735338516, -6.998537659847482]),
        //     Point::new([-0.09905592158533623, -6.999299102367242]),
        //     Point::new([-0.05503233742628238, -6.999783671074228]),
        //     Point::new([-0.011006576333094166, -6.999991346800182]),
        //     Point::new([0.03301962015121919, -6.999922121329998]),
        //     Point::new([0.07704451046645042, -6.999575997402056]),
        //     Point::new([0.12106635310403599, -6.998952988708103]),
        //     Point::new([0.16508340667599533, -6.998053119892721]),
        //     Point::new([0.2090939299837658, -6.9968764265523475]),
        //     Point::new([0.25309618208712964, -6.995422955233867]),
        //     Point::new([0.29708842237303756, -6.993692763432771]),
        //     Point::new([0.34106891062448114, -6.991685919590884]),
        //     Point::new([0.3850359070893504, -6.989402503093657]),
        //     Point::new([0.42898767254920983, -6.986842604267024]),
        //     Point::new([0.47292246838814717, -6.98400632437383]),
        //     Point::new([0.5168385566614987, -6.980893775609829]),
        //     Point::new([0.5607342001646475, -6.97750508109924]),
        //     Point::new([0.6046076625016933, -6.973840374889881]),
        //     Point::new([0.6484572081541887, -6.969899801947864]),
        //     Point::new([0.6922811025497426, -6.965683518151863]),
        //     Point::new([0.7360776121306849, -6.961191690286943]),
        //     Point::new([0.7798450044225915, -6.95642449603797]),
        //     Point::new([0.8235815481028664, -6.951382123982574]),
        //     Point::new([0.8672855130691848, -6.9460647735836965]),
        //     Point::new([0.9109551705079497, -6.940472655181694]),
        //     Point::new([0.9545887929626983, -6.9346059899860215]),
        //     Point::new([0.9981846544023913, -6.928465010066485]),
        //     Point::new([1.0417410302897414, -6.92204995834405]),
        //     Point::new([1.0852561976493798, -6.915361088581247]),
        //     Point::new([1.1287284351360638, -6.90839866537212]),
        //     Point::new([1.1721560231027182, -6.901162964131771]),
        //     Point::new([1.2155372436685097, -6.893654271085457]),
        //     Point::new([1.2588703807867527, -6.885872883257273]),
        //     Point::new([1.3021537203128408, -6.877819108458395]),
        //     Point::new([1.3453855500720036, -6.869493265274918]),
        //     Point::new([1.3885641599270866, -6.8608956830552374]),
        //     Point::new([1.4316878418461552, -6.852026701897031]),
        //     Point::new([1.4747548899700802, -6.842886672633805]),
        //     Point::new([1.5177636006800328, -6.833475956821007]),
        //     Point::new([1.5607122726648357, -6.823794926721741]),
        //     Point::new([1.603599206988308, -6.813843965292019]),
        //     Point::new([1.6464227071564248, -6.803623466165636]),
        //     Point::new([1.6891810791844726, -6.79313383363858]),
        //     Point::new([1.7318726316640114, -6.782375482653048]),
        //     Point::new([1.7744956758298303, -6.771348838781032]),
        //     Point::new([1.8170485256267033, -6.7600543382074845]),
        //     Point::new([1.8595294977761307, -6.748492427713056]),
        //     Point::new([1.9019369118428815, -6.736663564656436]),
        //     Point::new([1.94426909030151, -6.724568216956248]),
        //     Point::new([1.986524358602677, -6.712206863072549]),
        //     Point::new([2.028701045239404, -6.699579991987897]),
        //     Point::new([2.0707974818132153, -6.686688103188008]),
        //     Point::new([2.1128120031000925, -6.673531706642007]),
        //     Point::new([2.1547429471163926, -6.660111322782237]),
        //     Point::new([2.1965886551845455, -6.646427482483696]),
        //     Point::new([2.2383474719987144, -6.632480727043009]),
        //     Point::new([2.280017745690228, -6.618271608157046]),
        //     Point::new([2.3215978278929708, -6.603800687901069]),
        //     Point::new([2.36308607380854, -6.589068538706525]),
        //     Point::new([2.4044808422713593, -6.574075743338376]),
        //     Point::new([2.44578049581355, -6.558822894872069]),
        //     Point::new([2.4869834007297515, -6.543310596670059]),
        //     Point::new([2.5280879271417067, -6.527539462357953]),
        //     Point::new([2.5690924490627536, -6.511510115800232]),
        //     Point::new([2.609995344462159, -6.495223191075566]),
        //     Point::new([2.650794995329246, -6.4786793324517475]),
        //     Point::new([2.6914897877374417, -6.461879194360187]),
        //     Point::new([2.7320781119080735, -6.444823441370044]),
        //     Point::new([2.7725583622740957, -6.427512748161918]),
        //     Point::new([2.8129289375435524, -6.409947799501186]),
        //     Point::new([2.8531882407629707, -6.392129290210884]),
        //     Point::new([2.893334679380482, -6.3740579251442515]),
        //     Point::new([2.933366665308866, -6.355734419156825]),
        //     Point::new([2.973282614988328, -6.337159497078179]),
        //     Point::new([3.013080949449183, -6.31833389368324]),
        //     Point::new([3.052760094374276, -6.299258353663228]),
        //     Point::new([3.0923184801612775, -6.279933631596201]),
        //     Point::new([3.1317545419847868, -6.260360491917191]),
        //     Point::new([3.1710667198581937, -6.240539708887989]),
        //     Point::new([3.2102534586954343, -6.220472066566492]),
        //     Point::new([3.249313208372458, -6.200158358775708]),
        //     Point::new([3.2882444237885937, -6.179599389072338]),
        //     Point::new([3.3270455649276243, -6.158795970715008]),
        //     Point::new([3.3657150969187506, -6.137748926632076]),
        //     Point::new([3.4042514900972614, -6.116459089389102]),
        //     Point::new([3.442653220065089, -6.094927301155894]),
        //     Point::new([3.4809187677510627, -6.073154413673213]),
        //     Point::new([3.519046619471049, -6.051141288219057]),
        //     Point::new([3.557035266987784, -6.028888795574616]),
        //     Point::new([3.5948832075705575, -6.006397815989815]),
        //     Point::new([3.6325889440546693, -5.983669239148482]),
        //     Point::new([3.6701509849006175, -5.960703964133182]),
        //     Point::new([3.707567844253142, -5.937502899389617]),
        //     Point::new([3.744838041999957, -5.914066962690728]),
        //     Point::new([3.7819601038303468, -5.890397081100353]),
        //     Point::new([3.8189325612934377, -5.866494190936589]),
        //     Point::new([3.855753951856335, -5.842359237734723]),
        //     Point::new([3.892422818961928, -5.817993176209859]),
        //     Point::new([3.9289377120865527, -5.79339697021912]),
        //     Point::new([3.9652971867973363, -5.768571592723552]),
        //     Point::new([4.001499804809347, -5.743518025749615]),
        //     Point::new([4.0375441340425064, -5.718237260350338]),
        //     Point::new([4.073428748678202, -5.6927302965661335]),
        //     Point::new([4.109152229215729, -5.666998143385209]),
        //     Point::new([4.1447131625284035, -5.641041818703687]),
        //     Point::new([4.180110141919501, -5.614862349285308]),
        //     Point::new([4.215341767177862, -5.588460770720845]),
        //     Point::new([4.250406644633319, -5.561838127387107]),
        //     Point::new([4.2853033872117905, -5.53499547240566]),
        //     Point::new([4.320030614490184, -5.507933867601132]),
        //     Point::new([4.354586952750967, -5.480654383459246]),
        //     Point::new([4.388971035036546, -5.45315809908444]),
        //     Point::new([4.423181501203304, -5.4254461021572125]),
        //     Point::new([4.45721699797542, -5.397519488891076]),
        //     Point::new([4.4910761789984175, -5.369379363989192]),
        //     Point::new([4.524757704892381, -5.341026840600694]),
        //     Point::new([4.558260243304988, -5.312463040276624]),
        //     Point::new([4.5915824689641695, -5.283689092925596]),
        //     Point::new([4.624723063730568, -5.254706136769072]),
        //     Point::new([4.657680716649651, -5.2255153182963685]),
        //     Point::new([4.690454124003599, -5.196117792219267]),
        //     Point::new([4.7230419893628435, -5.166514721426377]),
        //     Point::new([4.755443023637386, -5.136707276937096]),
        //     Point::new([4.787655945127755, -5.106696637855324]),
        //     Point::new([4.8196794795757425, -5.0764839913227835]),
        //     Point::new([4.851512360214779, -5.046070532472096]),
        //     Point::new([4.883153327820057, -5.015457464379486]),
        //     Point::new([4.914601130758356, -4.98464599801718]),
        //     Point::new([4.9458545250375225, -4.953637352205535]),
        //     Point::new([4.976912274355714, -4.922432753564788]),
        //     Point::new([5.007773150150269, -4.891033436466576]),
        //     Point::new([5.038435931646343, -4.85944064298506]),
        //     Point::new([5.0688994059051575, -4.827655622847837]),
        //     Point::new([5.099162367872024, -4.795679633386468]),
        //     Point::new([5.129223620423973, -4.763513939486772]),
        //     Point::new([5.1590819744171466, -4.731159813538756]),
        //     Point::new([5.188736248733797, -4.698618535386324]),
        //     Point::new([5.21818527032905, -4.665891392276608]),
        //     Point::new([5.247427874277276, -4.632979678809077]),
        //     Point::new([5.276462903818183, -4.599884696884324]),
        //     Point::new([5.305289210402585, -4.566607755652538]),
        //     Point::new([5.333905653737812, -4.533150171461762]),
        //     Point::new([5.362311101832844, -4.499513267805777]),
        //     Point::new([5.390504431043064, -4.465698375271788]),
        //     Point::new([5.4184845261147325, -4.431706831487751]),
        //     Point::new([5.446250280229073, -4.397539981069501]),
        //     Point::new([5.4738005950460895, -4.3631991755675195]),
        //     Point::new([5.501134380747979, -4.328685773413513]),
        //     Point::new([5.52825055608228, -4.294001139866635]),
        //     Point::new([5.5551480484046, -4.259146646959525]),
        //     Point::new([5.58182579372109, -4.224123673443986]),
        //     Point::new([5.6082827367305015, -4.188933604736489]),
        //     Point::new([5.634517830865941, -4.153577832863347]),
        //     Point::new([5.660530038336284, -4.118057756405637]),
        //     Point::new([5.6863183301671985, -4.082374780443917]),
        //     Point::new([5.711881686241881, -4.046530316502597]),
        //     Point::new([5.737219095341377, -4.010525782494144]),
        //     Point::new([5.762329555184615, -3.974362602662957]),
        //     Point::new([5.787212072468022, -3.938042207529065]),
        //     Point::new([5.811865662904843, -3.901566033831498]),
        //     Point::new([5.836289351264058, -3.8649355244714974]),
        //     Point::new([5.8604821714089725, -3.8281521284553954]),
        //     Point::new([5.884443166335423, -3.791217300837337]),
        //     Point::new([5.9081713882096505, -3.7541325026616814]),
        //     Point::new([5.931665898405773, -3.7168992009052433]),
        //     Point::new([5.954925767542928, -3.6795188684192492]),
        //     Point::new([5.977950075522044, -3.6419929838710545]),
        //     Point::new([6.000737911562215, -3.6043230316856936]),
        //     Point::new([6.023288374236753, -3.566510501987114]),
        //     Point::new([6.045600571508821, -3.5285568905392775]),
        //     Point::new([6.067673620766748, -3.490463698686943]),
        //     Point::new([6.089506648858914, -3.4522324332963263]),
        //     Point::new([6.111098792128315, -3.413864606695445]),
        //     Point::new([6.132449196446704, -3.37536173661434]),
        //     Point::new([6.1535570172484, -3.3367253461249957]),
        //     Point::new([6.174421419563676, -3.2979569635811314]),
        //     Point::new([6.19504157805181, -3.2590581225577036]),
        //     Point::new([6.21541667703371, -3.220030361790278]),
        //     Point::new([6.235545910524196, -3.1808752251141463]),
        //     Point::new([6.255428482263885, -3.1415942614032373]),
        //     Point::new([6.2750636057506695, -3.1021890245088874]),
        //     Point::new([6.2944505042708565, -3.062661073198332]),
        //     Point::new([6.313588410929865, -3.0230119710930867]),
        //     Point::new([6.332476568682585, -2.98324328660705]),
        //     Point::new([6.351114230363308, -2.9433565928845065]),
        //     Point::new([6.369500658715296, -2.9033534677378525]),
        //     Point::new([6.38763512641993, -2.8632354935852278]),
        //     Point::new([6.405516916125501, -2.8230042573878724]),
        //     Point::new([6.423145320475562, -2.7826613505873974]),
        //     Point::new([6.4405196421369375, -2.7422083690427854]),
        //     Point::new([6.45763919382728, -2.701646912967303]),
        //     Point::new([6.474503298342274, -2.660978586865182]),
        //     Point::new([6.491111288582426, -2.6202049994681342]),
        //     Point::new([6.507462507579438, -2.5793277636717544]),
        //     Point::new([6.523556308522216, -2.538348496471671]),
        //     Point::new([6.539392054782436, -2.497268818899628]),
        //     Point::new([6.554969119939745, -2.456090355959317]),
        //     Point::new([6.570286887806523, -2.41481473656214]),
        //     Point::new([6.585344752452276, -2.37344359346273]),
        //     Point::new([6.600142118227587, -2.3319785631944097]),
        //     Point::new([6.614678399787697, -2.2904212860044053]),
        //     Point::new([6.628953022115641, -2.248773405789013]),
        //     Point::new([6.64296542054501, -2.2070365700285217]),
        //     Point::new([6.656715040782276, -2.165212429722084]),
        //     Point::new([6.670201338928726, -2.1233026393223913]),
        //     Point::new([6.683423781501974, -2.0813088566702085]),
        //     Point::new([6.696381845457063, -2.039232742928836]),
        //     Point::new([6.70907501820716, -1.9970759625183514]),
        //     Point::new([6.721502797643824, -1.954840183049816]),
        //     Point::new([6.733664692156883, -1.912527075259264]),
        //     Point::new([6.745560220653859, -1.8701383129416558]),
        //     Point::new([6.757188912579022, -1.8276755728846212]),
        //     Point::new([6.768550307931987, -1.78514053480218]),
        //     Point::new([6.779643957285918, -1.7425348812682455]),
        //     Point::new([6.790469421805305, -1.6998602976501114]),
        //     Point::new([6.801026273263321, -1.657118472041763]),
        //     Point::new([6.811314094058766, -1.6143110951970832]),
        //     Point::new([6.821332477232581, -1.5714398604630133]),
        //     Point::new([6.831081026483952, -1.5285064637125203]),
        //     Point::new([6.840559356185977, -1.485512603277561]),
        //     Point::new([6.8497670914009365, -1.4424599798818507]),
        //     Point::new([6.8587038678951044, -1.3993502965736382]),
        //     Point::new([6.867369332153176, -1.356185258658287]),
        //     Point::new([6.875763141392236, -1.3129665736308667]),
        //     Point::new([6.88388496357533, -1.2696959511085613]),
        //     Point::new([6.891734477424592, -1.22637510276309]),
        //     Point::new([6.899311372433958, -1.1830057422529479]),
        //     Point::new([6.906615348881443, -1.1395895851556619]),
        //     Point::new([6.913646117841002, -1.0961283488999087]),
        //     Point::new([6.920403401193958, -1.052623752697559]),
        //     Point::new([6.926886931640003, -1.0090775174757134]),
        //     Point::new([6.933096452707772, -0.9654913658085775]),
        //     Point::new([6.939031718764987, -0.9218670218493712]),
        //     Point::new([6.944692495028178, -0.8782062112620753]),
        //     Point::new([6.950078557571966, -0.8345106611532198]),
        //     Point::new([6.95518969333792, -0.7907821000035133]),
        //     Point::new([6.96002570014299, -0.7470222575995192]),
        //     Point::new([6.964586386687501, -0.7032328649651796]),
        //     Point::new([6.968871572562722, -0.6594156542933911]),
        //     Point::new([6.972881088258, -0.6155723588774332]),
        //     Point::new([6.97661477516747, -0.5717047130424482]),
        //     Point::new([6.980072485596324, -0.5278144520768161]),
        //     Point::new([6.983254082766658, -0.4839033121634939]),
        //     Point::new([6.9861594408228775, -0.43997303031137913]),
        //     Point::new([6.988788444836681, -0.39602534428654906]),
        //     Point::new([6.991140990811604, -0.3520619925435693]),
        //     Point::new([6.9932169856871305, -0.30808471415667477]),
        //     Point::new([6.99501634734238, -0.2640952487510267]),
        //     Point::new([6.9965390045993505, -0.2200953364338478]),
        //     Point::new([6.997784897225737, -0.1760867177256379]),
        //     Point::new([6.998753975937312, -0.13207113349127372]),
        //     Point::new([6.999446202399878, -0.08805032487119485]),
        //     Point::new([6.999861549230783, -0.04402603321247883]),
        //     Point::new([7.0, 0.0]),
    ];
    let mut tree = QuadTree::from_bounds(xmin, xmax, ymin, ymax);
    tree.from_points(&levels, points);
    tree.balance(&levels);

    let (connectivity, nodal_coordinates) = tree.sandbox(&levels);

    tree.prune();

    let mut file = BufWriter::new(File::create("target/foo.py").unwrap());
    file.write_all(format!("import matplotlib.pyplot as plt\nimport matplotlib.patches as patches\nimport numpy as np\n\n\nax = plt.subplot(xlim=[{},{}],ylim=[{},{}],aspect='equal')\n\n", xmin, xmax, ymin, ymax).as_bytes()).unwrap();
    tree.iter().for_each(|cell| {
        file.write_all(
            format!(
                "ax.add_patch(patches.Rectangle(({},{}),{},{}, edgecolor='red'))\n",
                cell.min_x,
                cell.min_y,
                cell.max_x - cell.min_x,
                cell.max_y - cell.min_y
            )
            .as_bytes(),
        )
        .unwrap()
    });
    connectivity.iter().for_each(|conn|
        file.write_all(
            format!(
                "ax.add_patch(patches.Polygon(np.array([[{}, {}], [{}, {}], [{}, {}], [{}, {}]]), edgecolor='white',fill=False))\n",
                nodal_coordinates[conn[0] - 1][0],
                nodal_coordinates[conn[0] - 1][1],
                nodal_coordinates[conn[1] - 1][0],
                nodal_coordinates[conn[1] - 1][1],
                nodal_coordinates[conn[2] - 1][0],
                nodal_coordinates[conn[2] - 1][1],
                nodal_coordinates[conn[3] - 1][0],
                nodal_coordinates[conn[3] - 1][1],
            )
        .as_bytes()).unwrap()
    );
    file.write_all("\n\nplt.show()\n".as_bytes()).unwrap();
    file.flush().unwrap();

    let mut file = BufWriter::new(File::create("target/foo.inp").unwrap());
    file.write_all("*PART, NAME=Part-Default\n**\n*NODE, NSET=ALLNODES\n".as_bytes())
        .unwrap();
    nodal_coordinates
        .iter()
        .enumerate()
        .for_each(|(node, coord)| {
            file.write_all(format!("\t{}", node + 1).as_bytes())
                .unwrap();
            coord
                .iter()
                .for_each(|entry| file.write_all(format!(",\t{}", entry).as_bytes()).unwrap());
            file.write_all("\n".as_bytes()).unwrap()
        });
    file.write_all("**\n*ELEMENT, TYPE=S4R, ELSET=EB1\n".as_bytes())
        .unwrap();
    connectivity.iter().enumerate().for_each(|(node, conn)| {
        file.write_all(format!("\t{}", node + 1).as_bytes())
            .unwrap();
        conn.iter()
            .for_each(|entry| file.write_all(format!(",\t{}", entry).as_bytes()).unwrap());
        file.write_all("\n".as_bytes()).unwrap()
    });
    file.write_all(
        "**
********************************** P R O P E R T I E S ************************
*SHELL SECTION, ELSET=EB1, SECTION INTEGRATION=SIMPSON, MATERIAL=Default-Steel
1.000000e+00
**
*END PART
**
**
**
********************************** E N D   P A R T S **********************************
**
**
********************************** A S S E M B L Y ************************************
**
*ASSEMBLY, NAME=ASSEMBLY1
**
*INSTANCE, NAME=Part-Default_1, PART=Part-Default
*END INSTANCE
**
*END ASSEMBLY
**
**
**
*MATERIAL, NAME = Default-Steel
**
"
        .as_bytes(),
    )
    .unwrap();
    file.flush().unwrap();
}

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Error as ErrorIO, Write},
    path::{Path, PathBuf},
};
