//! This module tests the QuadTree implementation.
//! Example:
//!     cargo test tree_level_1_viz -- --nocapture && python tree_level_1.py


use crate::tree::inventory;

// use reverse_string::*;
use super::{Cell2D, Point2D, QuadTree};

#[test]
fn tree_level_1() {
    // This is a domain with origin (1.0, -1.0) and extrema
    // at (3, 1), to match the illustration at
    // https://github.com/sandialabs/sibl/blob/master/geo/doc/quadtree.md#refinement-example
    let cell = Cell2D {
        origin: Point2D { x: 1.0, y: -1.0 },
        width: 2.0,
        height: 2.0,
    };

    // let points: Vec<Point2D> = vec![seed, Point2D { x: 1.0, y: 1.0 }, Point2D { x: 3.0, y: 4.0 }];
    let seed = Point2D { x: 2.6, y: 0.6 };
    let seed_outside = Point2D { x: 0.0, y: 0.0 };

    assert!(cell.contains(&seed));
    assert!(!cell.contains(&seed_outside));

    let mut tree = QuadTree::new(cell, 0, 1);  // start level 0 up to 1

    // The initial state of the tree is undivided, with no children
    assert!(!tree.divided, "QuadTree should not be divided yet.");
    assert!(tree.sw.is_none(), "QuadTree 00 should be None before subdivision.");
    assert!(tree.se.is_none(), "QuadTree 01 should be None before subdivision.");
    assert!(tree.nw.is_none(), "QuadTree 10 should be None before subdivision.");
    assert!(tree.ne.is_none(), "QuadTree 11 should be None before subdivision.");

    tree.insert(seed);

    // After insertion of a valid point, the tree should be subdivided
    assert!(tree.divided, "QuadTree should be divided since its cell domain contains the seed.");

    assert!(tree.sw.is_some(), "QuadTree 00 should exist.");
    assert!(tree.se.is_some(), "QuadTree 01 should exist.");
    assert!(tree.nw.is_some(), "QuadTree 10 should exist.");
    assert!(tree.ne.is_some(), "QuadTree 11 should exist.");

    assert!(tree.sw.unwrap().level == 1, "QuadTree 00 should be at level 1.");
    assert!(tree.se.unwrap().level == 1, "QuadTree 01 should be at level 1.");
    assert!(tree.nw.unwrap().level == 1, "QuadTree 10 should be at level 1.");
    assert!(tree.ne.unwrap().level == 1, "QuadTree 11 should be at level 1.");

}

#[test]
fn tree_level_1_viz() {
    // Prints to the terminal the Python code needed to visualize
    // the QuadTree.  Use `cargo test tree_level_1_viz -- --nocapture` to collect the
    // Python code for plotting the QuadTree
    let cell = Cell2D {
        origin: Point2D { x: 1.0, y: -1.0 },
        width: 2.0,
        height: 2.0,
    };

    let seed = Point2D { x: 2.6, y: 0.6 };
    assert!(cell.contains(&seed));

    let mut tree = QuadTree::new(cell, 0, 1);  // start level 0 up to 1
    tree.insert(seed);
    let show = true;
    let save = true;
    let filename = "tree_level_1.py";
    let _ = tree.pyplot(show, save, filename);
    // tree.inventory();
    inventory(&tree);
}


#[test]
fn tree_level_2() {
    // This is a domain with origin (1.0, -1.0) and extrema
    // at (3, 1), to match the illustration at
    // https://github.com/sandialabs/sibl/blob/master/geo/doc/quadtree.md#refinement-example
    let (width, height) = (2.0, 2.0);
    println!("\n**** *** width: {}, height: {}\n *********", width, height);
    let (x0, y0) = (1.0, -1.0) ;
    let (x1, y1) = (x0 + width, y0 + height);

    let cell = Cell2D {
        origin: Point2D { x: x0, y: y0 },
        width,
        height
    };

    let seed = Point2D { x: 2.6, y: 0.6 };
    assert!(cell.contains(&seed));

    let mut tree = QuadTree::new(cell, 0, 2);  // start level 0 up to 2

    tree.insert(seed);

    // Check level 1 QuadTree
    assert!(tree.sw.is_some(), "QuadTree 00 should exist.");
    assert!(tree.se.is_some(), "QuadTree 01 should exist.");
    assert!(tree.nw.is_some(), "QuadTree 10 should exist.");
    assert!(tree.ne.is_some(), "QuadTree 11 should exist.");

    assert_eq!(tree.sw.as_ref().unwrap().level, 1, "QuadTree 00 should be at level 1.");
    assert_eq!(tree.se.as_ref().unwrap().level, 1, "QuadTree 01 should be at level 1.");
    assert_eq!(tree.nw.as_ref().unwrap().level, 1, "QuadTree 10 should be at level 1.");
    assert_eq!(tree.ne.as_ref().unwrap().level, 1, "QuadTree 11 should be at level 1.");

    assert!(!tree.sw.as_ref().unwrap().divided, "QuadTree 00 should not be divided");
    assert!(!tree.se.as_ref().unwrap().divided, "QuadTree 01 should not be divided");
    assert!(!tree.nw.as_ref().unwrap().divided, "QuadTree 10 should not be divided");
    assert!(tree.ne.as_ref().unwrap().divided, "QuadTree 11 should be divided");

    // Check level 2 QuadTree
    assert!(tree.sw.as_ref().unwrap().sw.is_none(), "QuadTree 0000 should not exist.");
    assert!(tree.sw.as_ref().unwrap().se.is_none(), "QuadTree 0001 should not exist.");
    assert!(tree.sw.as_ref().unwrap().nw.is_none(), "QuadTree 0010 should not exist.");
    assert!(tree.sw.as_ref().unwrap().ne.is_none(), "QuadTree 0011 should not exist.");

    assert!(tree.se.as_ref().unwrap().sw.is_none(), "QuadTree 0100 should not exist.");
    assert!(tree.se.as_ref().unwrap().se.is_none(), "QuadTree 0101 should not exist.");
    assert!(tree.se.as_ref().unwrap().nw.is_none(), "QuadTree 0110 should not exist.");
    assert!(tree.se.as_ref().unwrap().ne.is_none(), "QuadTree 0111 should not exist.");

    assert!(tree.nw.as_ref().unwrap().sw.is_none(), "QuadTree 1000 should not exist.");
    assert!(tree.nw.as_ref().unwrap().se.is_none(), "QuadTree 1001 should not exist.");
    assert!(tree.nw.as_ref().unwrap().nw.is_none(), "QuadTree 1010 should not exist.");
    assert!(tree.nw.as_ref().unwrap().ne.is_none(), "QuadTree 1011 should not exist.");

    assert!(tree.ne.as_ref().unwrap().sw.is_some(), "QuadTree 1100 should exist.");
    assert!(tree.ne.as_ref().unwrap().se.is_some(), "QuadTree 1101 should exist.");
    assert!(tree.ne.as_ref().unwrap().nw.is_some(), "QuadTree 1110 should exist.");
    assert!(tree.ne.as_ref().unwrap().ne.is_some(), "QuadTree 1111 should exist.");

    assert_eq!(tree.ne.as_ref().unwrap().sw.as_ref().unwrap().level, 2, "QuadTree 1100 should be at level 2.");
    assert_eq!(tree.ne.as_ref().unwrap().se.as_ref().unwrap().level, 2, "QuadTree 1101 should be at level 2.");
    assert_eq!(tree.ne.as_ref().unwrap().nw.as_ref().unwrap().level, 2, "QuadTree 1110 should be at level 2.");
    assert_eq!(tree.ne.as_ref().unwrap().ne.as_ref().unwrap().level, 2, "QuadTree 1111 should be at level 2.");

    assert!(!tree.ne.as_ref().unwrap().sw.as_ref().unwrap().divided, "QuadTree 1100 should not be divided.");
    assert!(!tree.ne.as_ref().unwrap().se.as_ref().unwrap().divided, "QuadTree 1101 should not be divided.");
    assert!(!tree.ne.as_ref().unwrap().nw.as_ref().unwrap().divided, "QuadTree 1110 should not be divided.");
    assert!(!tree.ne.as_ref().unwrap().ne.as_ref().unwrap().divided, "QuadTree 1111 should not be divided.");

    inventory(&tree);

}
