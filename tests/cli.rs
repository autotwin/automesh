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

/// A tessellation input skips the reader that once printed the banner.
#[test]
fn mesh_hex_stl_prints_banner_once() {
    let output = out("exo");
    let result = Command::new(BIN)
        .args([
            "mesh",
            "hex",
            "-i",
            sphere().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--uniform",
            "0.2",
        ])
        .output()
        .expect("failed to spawn automesh");
    assert!(result.status.success(), "command failed");
    let stdout = String::from_utf8_lossy(&result.stdout);
    let banner = concat!("automesh ", env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout.matches(banner).count(), 1, "stdout was: {stdout}");
}

/// Clap rejects a bad method while parsing, before any file is read or written.
#[test]
fn smooth_rejects_an_unknown_method() {
    let output = out("exo");
    let result = Command::new(BIN)
        .args([
            "mesh",
            "hex",
            "-i",
            input("letter_f_3d.npy").to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "smooth",
            "-m",
            "bogus",
        ])
        .output()
        .expect("failed to spawn automesh");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(result.status.code(), Some(2), "stderr was: {stderr}");
    assert!(
        stderr.contains("invalid value 'bogus'"),
        "stderr was: {stderr}"
    );
    assert!(
        result.stdout.is_empty(),
        "the command did work before failing"
    );
    assert!(!output.exists(), "the command wrote an output file");
}

#[test]
fn smooth_accepts_method_spellings() {
    let inp = out("inp");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        inp.to_str().unwrap(),
    ]);
    for method in [
        "Laplace",
        "laplace",
        "Laplacian",
        "laplacian",
        "Taubin",
        "taubin",
    ] {
        let output = out("inp");
        run(&[
            "smooth",
            "-i",
            inp.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-n",
            "2",
            "-m",
            method,
        ]);
        assert_nonempty(&output);
    }
}

#[test]
fn partition_rcb_and_rib_to_exo() {
    let source = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        source.to_str().unwrap(),
    ]);
    ["rcb", "rib"].into_iter().for_each(|method| {
        let output = out("exo");
        run(&[
            "partition",
            "-i",
            source.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-m",
            method,
            "-n",
            "3",
            "-j",
            "2",
        ]);
        assert_nonempty(&output);
    });
}

#[test]
fn partition_box_to_vtu() {
    let source = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        source.to_str().unwrap(),
    ]);
    let output = out("vtu");
    run(&[
        "partition",
        "-i",
        source.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-m",
        "box",
        "-d",
        "2",
        "1",
        "1",
    ]);
    assert_nonempty(&output);
}

#[test]
fn partition_rejects_missing_parts() {
    let source = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        source.to_str().unwrap(),
    ]);
    let result = Command::new(BIN)
        .args([
            "partition",
            "-i",
            source.to_str().unwrap(),
            "-o",
            out("exo").to_str().unwrap(),
        ])
        .output()
        .expect("failed to spawn automesh");
    assert!(!result.status.success());
}

fn hex_source() -> PathBuf {
    let source = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        source.to_str().unwrap(),
    ]);
    source
}

#[test]
fn agglomerate_to_exo_and_vtu() {
    let source = hex_source();
    [("rcb", "exo"), ("rib", "exo"), ("rcb", "vtu")]
        .into_iter()
        .for_each(|(method, extension)| {
            let output = out(extension);
            run(&[
                "agglomerate",
                "-i",
                source.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "-m",
                method,
                "-n",
                "3",
            ]);
            assert_nonempty(&output);
        });
}

#[test]
fn agglomerate_box_to_exo() {
    let source = hex_source();
    let output = out("exo");
    run(&[
        "agglomerate",
        "-i",
        source.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-m",
        "box",
        "-d",
        "2",
        "1",
        "1",
    ]);
    assert_nonempty(&output);
}

#[test]
fn agglomerate_rejects_unsupported_output() {
    let source = hex_source();
    let result = Command::new(BIN)
        .args([
            "agglomerate",
            "-i",
            source.to_str().unwrap(),
            "-o",
            out("inp").to_str().unwrap(),
            "-n",
            "3",
        ])
        .output()
        .expect("failed to spawn automesh");
    assert!(!result.status.success());
}
