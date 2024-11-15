// Reference:
// https://github.com/sandialabs/sibl/blob/master/geo/src/ptg/quadtree.py


#[cfg(test)]
pub mod test;


#[derive(Debug)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}


#[derive(Debug)]
pub struct Cell2D {
    origin: Point2D,
    width: f64,
    height: f64,
}


impl Cell2D {
    fn contains(&self, point: Point2D) -> bool {
        point.x >= self.origin.x &&
        point.x < self.origin.x + self.width &&
        point.y >= self.origin.y &&
        point.y < self.origin.y + self.height
    }
}


#[derive(Debug)]
pub struct QuadTree {
    cell: Cell2D,
    level: usize,
    level_max: usize,
    points: Vec<Point2D>,
    divided: bool,
    nw: Option<Box<QuadTree>>,  // northwest
    ne: Option<Box<QuadTree>>,  // northeast
    sw: Option<Box<QuadTree>>,  // southwest
    se: Option<Box<QuadTree>>,  // southeast
}

impl QuadTree {
    fn new(cell: Cell2D, level: usize, level_max: usize) -> Self {
        QuadTree {
            cell,
            level,
            level_max,
            points: Vec::new(),
            divided: false,
            nw: None,
            ne: None,
            sw: None,
            se: None,
        }
    }

    fn subdivide(&mut self) {
        // Check if cell is already divided, of the maximum number
        // of levels has been reached
        if self.divided || self.level == self.level_max { 
            return;  // already subdivided or no more levels to subdivide

        }

        let x = self.cell.origin.x;
        let y = self.cell.origin.y;
        let width = self.cell.width / 2.0;
        let height = self.cell.height / 2.0;

        self.nw = Some(Box::new(
            QuadTree::new(
                Cell2D {
                    origin: Point2D { x, y: y + height },
                    width,
                    height,
                },
                self.level + 1,
                self.level_max,
            )
        ));

        self.ne = Some(Box::new(
            QuadTree::new(
                Cell2D {
                    origin: Point2D { x: x + width, y: y + height },
                    width,
                    height
                },
                self.level + 1,
                self.level_max,
            )
        ));

        self.sw = Some(Box::new(
            QuadTree::new(
                Cell2D {
                    origin: Point2D { x, y },
                    width,
                    height
                },
                self.level + 1,
                self.level_max,
            )
        ));

        self.se = Some(Box::new(
            QuadTree::new(
                Cell2D {
                    origin: Point2D { x: x + width, y },
                    width,
                    height
                },
                self.level + 1,
                self.level_max,
            )
        ));

    }
}


pub fn reverse(input: &str) -> String {
    // input.chars() converts a string slice into an iterator of its
    // characters, which is important because it handles Unicode
    // charcters correctly.
    // .rev() reverses the order of the characters in the iterator
    // .collect() collects all the characters back into a String
    input.chars().rev().collect()
}
