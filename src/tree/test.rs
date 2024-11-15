// use reverse_string::*;
use super::{reverse, Cell2D, Point2D, QuadTree};

#[test]
fn level_one_tree() {
    // This is a domain with origin (1.0, -1.0) and extrema
    // at (3, 1), to match the illustration at
    // https://github.com/sandialabs/sibl/blob/master/geo/doc/quadtree.md#refinement-example
    let cell = Cell2D {
        origin: Point2D { x: 1.0, y: -1.0 },
        width: 2.0,
        height: 2.0,
    };

    // let points: Vec<Point2D> = vec![seed, Point2D { x: 1.0, y: 1.0 }, Point2D { x: 3.0, y: 4.0 }];
    let seed_inside = Point2D { x: 2.6, y: 0.6 };
    let seed_outside = Point2D { x: 0.0, y: 0.0 };

    assert!(cell.contains(&seed_inside));
    assert!(!cell.contains(&seed_outside));

    // let points = vec![seed_inside];

    let mut tree = QuadTree {
        cell,
        level: 0,
        level_max: 1,
        points: Vec::new(),
        divided: false,
        sw: None,
        se: None, 
        nw: None,
        ne: None,
    };

    // The initial state of the tree is undivided, with no children
    assert!(!tree.divided, "QuadTree should not be divided yet.");
    assert!(tree.sw.is_none(), "Southwest QuadTree should be None before subdivision.");
    assert!(tree.se.is_none(), "Southeast QuadTree should be None before subdivision.");
    assert!(tree.nw.is_none(), "Northwest QuadTree should be None before subdivision.");
    assert!(tree.ne.is_none(), "Northeast QuadTree should be None before subdivision.");

    if tree.cell.contains(&seed_inside) {
        tree.subdivide();
        assert!(tree.divided, "QuadTree should be divided since its cell domain contains the seed.");

        assert!(tree.sw.is_some(), "Southwest QuadTree should exist.");
        assert!(tree.se.is_some(), "Southeast QuadTree should exist.");
        assert!(tree.nw.is_some(), "Northwest QuadTree should exist.");
        assert!(tree.ne.is_some(), "Northwest QuadTree should exist.");

        assert!(tree.sw.unwrap().level == 1, "Southwest QuadTree should be at level 1.");
        assert!(tree.se.unwrap().level == 1, "Southeast QuadTree should be at level 1.");
        assert!(tree.nw.unwrap().level == 1, "Northwest QuadTree should be at level 1.");
        assert!(tree.ne.unwrap().level == 1, "Northweast QuadTree should be at level 1.");

    }

    // assert!(tree.divided == true);

}


#[test]
fn an_empty_string() {
    let input = "";
    let output = reverse(input);
    let expected = "";
    assert_eq!(output, expected);
}

#[test]
fn one_word() {
    let input = "robot";
    let output = reverse(input);
    let expected = "tobor";
    assert_eq!(output, expected);
}

#[test]
fn a_capitalized_word() {
    let input = "Ramen";
    let output = reverse(input);
    let expected = "nemaR";
    assert_eq!(output, expected);
}

#[test]
fn a_sentence_with_punctuation() {
    let input = "I'm hungry!";
    let output = reverse(input);
    let expected = "!yrgnuh m'I";
    assert_eq!(output, expected);
}

#[test]
fn a_palindrome() {
    let input = "racecar";
    let output = reverse(input);
    let expected = "racecar";
    assert_eq!(output, expected);
}

#[test]
fn an_even_sized_word() {
    let input = "drawer";
    let output = reverse(input);
    let expected = "reward";
    assert_eq!(output, expected);
}

#[test]
fn wide_characters() {
    let input = "子猫";
    let output = reverse(input);
    let expected = "猫子";
    assert_eq!(output, expected);
}

// #[test]
// #[cfg(feature = "grapheme")]
// fn grapheme_cluster_with_pre_combined_form() {
//     let input = "Würstchenstand";
//     let output = reverse(input);
//     let expected = "dnatsnehctsrüW";
//     assert_eq!(output, expected);
// }
// 
// #[test]
// #[cfg(feature = "grapheme")]
// fn grapheme_clusters() {
//     let input = "ผู้เขียนโปรแกรม";
//     let output = reverse(input);
//     let expected = "มรกแรปโนยขีเผู้";
//     assert_eq!(output, expected);
// }
