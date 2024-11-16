// Reference:
// https://github.com/sandialabs/sibl/blob/master/geo/src/ptg/quadtree.py

use std::fs::File;
use std::env;
use std::io::{self, Write};

#[cfg(test)]
pub mod test;

#[derive(Debug, Clone)]
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
    // Method to determine if the cell contains a Point2D
    fn contains(&self, point: &Point2D) -> bool {
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
    sw: Option<Box<QuadTree>>,  // southwest, index (i: 0, j:0)
    se: Option<Box<QuadTree>>,  // southeast, index (i: 1, j:0)
    nw: Option<Box<QuadTree>>,  // northwest, index (i: 0, j:1)
    ne: Option<Box<QuadTree>>,  // northeast, index (i: 1, j:1)
}

impl QuadTree {
    pub fn new(cell: Cell2D, level: usize, level_max: usize) -> Self {
        QuadTree {
            cell,
            level,
            level_max,
            points: Vec::new(),
            divided: false,
            se: None,
            sw: None,
            nw: None,
            ne: None,
        }
    }

    // Insert a point into the QuadTree
    pub fn insert(&mut self, point: Point2D) -> bool {
        // Check that the point is within the cell
        if !self.cell.contains(&point) {
            return false;  // Point is not contained in the cell
        }

        // The point is within the cell bounds, so push to self points
        self.points.push(point.clone());

        // If the maximum level has been reached, do not subdivide
        if self.level == self.level_max {
            return true;
        }
    
        // If the cell is not divided, subdivide it
        if !self.divided {
            self.subdivide();
        }
    
        // Try to insert the point into one of the children
        if let Some(sw) = &mut self.sw {
            if sw.insert(point.clone()) {
                return true;
            }
        }
        if let Some(se) = &mut self.se {
            if se.insert(point.clone()) {
                return true;
            }
        }
        if let Some(nw) = &mut self.nw {
            if nw.insert(point.clone()) {
                return true;
            }
        }
        if let Some(ne) = &mut self.ne {
            if ne.insert(point.clone()) {
                return true;
            }
        }

        false  // # shouldn't reach this point
    }

    // Subdivide the cell into four children cells
    fn subdivide(&mut self) {
        // Check if cell is already divided, of the maximum number
        // of levels has been reached
        if self.divided || self.level == self.level_max { 
            return;  // already subdivided or no more levels to subdivide
        }

        self.divided = true;  // mark this parent QuadTree as divided

        // create origins and dimensions for Cell2D parts of children
        let x = self.cell.origin.x;
        let y = self.cell.origin.y;
        let width = self.cell.width / 2.0;
        let height = self.cell.height / 2.0;

        self.sw = Some(Box::new(
            QuadTree::new(
                Cell2D {
                    origin: Point2D { x, y },
                    width,
                    height
                },
                self.level + 1,  // children are next higher level
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
                self.level + 1,  // children are next higher level
                self.level_max,
            )
        ));

        self.nw = Some(Box::new(
            QuadTree::new(
                Cell2D {
                    origin: Point2D { x, y: y + height },
                    width,
                    height,
                },
                self.level + 1,  // children are next higher level
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
                self.level + 1,  // children are next higher level
                self.level_max,
            )
        ));


    }

    pub fn pyplot(&self, show: bool, save: bool, filename: &str) -> io::Result<()> {
        let mut script = r#"
# This module, tree/mod.rs::pyplot, plots the
# QuadTree as a collection of square patches.

import matplotlib.pyplot as plt
import matplotlib.patches as patches

def main():
    print("Hello, World!")

if __name__ == "__main__":
    main()

"#.to_string(); // Convert the raw string literal to a String

        // if show {
        //     script.push_str("SHOW: bool = TRUE\n");
        // } else {
        //     script.push_str("SHOW: bool = FALSE\n");
        // }

        // if save {
        //     script.push_str("SAVE: bool = TRUE\n");
        // } else {
        //     script.push_str("SAVE: bool = FALSE\n");
        // }
        
        // Get the current working directory
        let cwd = env::current_dir()?;
        
        // Create the full path for the file
        let full_path = cwd.join(filename);
        
        // Create or open the file
        let mut file = File::create(&full_path)?;

        // Write the Python script contents to the file
        file.write_all(script.as_bytes())?;

        // Print the full path to the terminal
        println!("Created Python script: {:?}", full_path);

        println!("-----------------------------------");
        println!("Python script created successfully!");
        println!("-----------------------------------");

        Ok(())

    }

}
