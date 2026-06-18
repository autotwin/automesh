//! End-to-end smoke tests driving the compiled binary against fixtures in tests/input.

use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const BIN: &str = env!("CARGO_BIN_EXE_automesh");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn input(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("input")
        .join(name)
}

fn out(extension: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "automesh_cli_{}_{id}.{extension}",
        std::process::id()
    ))
}

/// Runs the binary with the given args, asserting success.
fn run(args: &[&str]) {
    let status = Command::new(BIN)
        .args(args)
        .arg("--quiet")
        .status()
        .expect("failed to spawn automesh");
    assert!(status.success(), "command failed: automesh {args:?}");
}

fn assert_nonempty(path: &PathBuf) {
    let metadata = std::fs::metadata(path).expect("output file was not created");
    assert!(metadata.len() > 0, "output file is empty: {path:?}");
}

#[test]
fn mesh_hex_to_exo() {
    let output = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_tri_to_stl() {
    let output = out("stl");
    run(&[
        "mesh",
        "tri",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert_nonempty(&output);
}

#[test]
fn convert_mesh_exo_to_inp() {
    let exo = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        exo.to_str().unwrap(),
    ]);
    let inp = out("inp");
    run(&[
        "convert",
        "mesh",
        "-i",
        exo.to_str().unwrap(),
        "-o",
        inp.to_str().unwrap(),
    ]);
    assert_nonempty(&inp);
}

#[test]
fn convert_segmentation_npy_to_spn() {
    let output = out("spn");
    run(&[
        "convert",
        "segmentation",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert_nonempty(&output);
}

#[test]
fn metrics_csv_and_npy() {
    let exo = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        exo.to_str().unwrap(),
    ]);
    for extension in ["csv", "npy"] {
        let metrics = out(extension);
        run(&[
            "metrics",
            "-i",
            exo.to_str().unwrap(),
            "-o",
            metrics.to_str().unwrap(),
        ]);
        assert_nonempty(&metrics);
    }
}

#[test]
fn smooth_taubin() {
    let inp = out("inp");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        inp.to_str().unwrap(),
    ]);
    let output = out("inp");
    run(&[
        "smooth",
        "-i",
        inp.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-n",
        "5",
    ]);
    assert_nonempty(&output);
}

#[test]
fn remesh_triangles() {
    let stl = out("stl");
    run(&[
        "mesh",
        "tri",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        stl.to_str().unwrap(),
    ]);
    let output = out("stl");
    run(&[
        "remesh",
        "-i",
        stl.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-n",
        "2",
    ]);
    assert_nonempty(&output);
}

#[test]
fn segment_mesh_to_segmentation() {
    let exo = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        exo.to_str().unwrap(),
    ]);
    let output = out("npy");
    run(&[
        "segment",
        "-i",
        exo.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-s",
        "1.0",
    ]);
    assert_nonempty(&output);
}

#[test]
fn diff_segmentations() {
    let output = out("npy");
    run(&[
        "diff",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert_nonempty(&output);
}

#[test]
fn extract_subrange() {
    let output = out("npy");
    run(&[
        "extract",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--xmin",
        "0",
        "--xmax",
        "1",
        "--ymin",
        "0",
        "--ymax",
        "1",
        "--zmin",
        "0",
        "--zmax",
        "1",
    ]);
    assert_nonempty(&output);
}

#[test]
fn defeature_segmentation() {
    let output = out("npy");
    run(&[
        "defeature",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--min",
        "1",
    ]);
    assert_nonempty(&output);
}
