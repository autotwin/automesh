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

/// The book's unit sphere, the only closed tessellation fixture in the repository.
fn sphere() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("book")
        .join("examples")
        .join("remesh")
        .join("sphere_radius_1.stl")
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
    // Exodus output is the netCDF-4 (HDF5) container: expect the HDF5 magic.
    let bytes = std::fs::read(&output).expect("output file was not created");
    assert_eq!(
        &bytes[..8],
        b"\x89HDF\r\n\x1a\n",
        "expected a netCDF-4 (HDF5) Exodus file"
    );
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
fn mesh_poly_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "poly",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-s",
        "5",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_hexdom_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hexdom",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-s",
        "6",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_tet_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "tet",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-s",
        "5",
        "-t",
        "0.1",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_tet_uniform_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "tet",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.3",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_tet_rejects_a_segmentation_input() {
    let output = out("vtu");
    let status = Command::new(BIN)
        .args([
            "mesh",
            "tet",
            "-i",
            input("letter_f_3d.npy").to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .arg("--quiet")
        .status()
        .expect("failed to spawn automesh");
    assert!(!status.success(), "tet meshing accepted a segmentation");
}

#[test]
fn mesh_hex_uniform_to_exo() {
    let output = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.2",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_hexdom_uniform_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hexdom",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.2",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_uniform_rejects_a_segmentation_input() {
    let output = out("exo");
    let status = Command::new(BIN)
        .args([
            "mesh",
            "hex",
            "-i",
            input("letter_f_3d.npy").to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--uniform",
            "0.2",
        ])
        .arg("--quiet")
        .status()
        .expect("failed to spawn automesh");
    assert!(!status.success(), "uniform meshing accepted a segmentation");
}

#[test]
fn smooth_poly() {
    let vtu = out("vtu");
    run(&[
        "mesh",
        "poly",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        vtu.to_str().unwrap(),
        "-s",
        "5",
    ]);
    let output = out("vtu");
    let metrics = out("csv");
    run(&[
        "smooth",
        "-i",
        vtu.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-n",
        "5",
        "--metrics",
        metrics.to_str().unwrap(),
    ]);
    assert_nonempty(&output);
    // Polyhedra have no Verdict metrics, so every column is NaN rather than a panic.
    let table = std::fs::read_to_string(&metrics).expect("metrics file was not created");
    let mut rows = table.lines().skip(1).peekable();
    assert!(rows.peek().is_some(), "metrics file has no rows");
    rows.for_each(|row| {
        assert!(
            row.split(',').all(|value| value.trim() == "NaN"),
            "expected all-NaN row, got {row:?}"
        )
    });
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
        "uniform",
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
