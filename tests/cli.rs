//! End-to-end smoke tests driving the compiled binary against fixtures in tests/input.

use std::{
    path::{Path, PathBuf},
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
fn mesh_tri_to_off() {
    let output = out("off");
    run(&[
        "mesh",
        "tri",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert_nonempty(&output);
    let bytes = std::fs::read(&output).expect("output file was not created");
    assert!(bytes.starts_with(b"OFF"), "expected an OFF magic header");
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

/// Size of the vtu a cut path writes for the sphere, with an optional `--tolerance`.
fn cut_size(element: &str, tolerance: Option<&str>) -> u64 {
    let (input, output) = (sphere(), out("vtu"));
    let mut args = vec![
        "mesh",
        element,
        "-i",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-s",
        "5",
    ];
    if let Some(tolerance) = tolerance {
        args.extend(["-t", tolerance]);
    }
    run(&args);
    std::fs::metadata(&output)
        .expect("output file was not created")
        .len()
}

#[test]
fn mesh_hexdom_tolerance_refines() {
    assert!(cut_size("hexdom", Some("0.001")) > cut_size("hexdom", None));
}

#[test]
fn mesh_poly_tolerance_refines() {
    assert!(cut_size("poly", Some("0.001")) > cut_size("poly", None));
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
fn mesh_hex_uniform_inflated_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.35",
        "--inflate",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_hex_uniform_inflated_and_snapped_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.35",
        "--inflate",
        "--snap",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_hex_uniform_marching_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.35",
        "--marching",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_marching_conflicts_are_rejected() {
    for extra in [
        &["--uniform", "0.35", "--marching", "--inflate"][..],
        &["--uniform", "0.35", "--marching", "--snap"][..],
        &["--marching"][..],
    ] {
        let output = out("vtu");
        let status = Command::new(BIN)
            .args([
                "mesh",
                "hex",
                "-i",
                sphere().to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ])
            .args(extra)
            .arg("--quiet")
            .status()
            .expect("failed to spawn automesh");
        assert!(!status.success(), "accepted {extra:?}");
    }
}

#[test]
fn mesh_hex_uniform_pyramids_snapped_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--uniform",
        "0.2",
        "--pyramids",
        "0.3",
        "--snap",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_hex_adaptive_pyramids_to_vtu() {
    let output = out("vtu");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--scale",
        "3",
        "--pyramids",
        "0.3",
    ]);
    assert_nonempty(&output);
}

#[test]
fn mesh_pyramids_conflicts_with_marching_and_inflate() {
    for flag in ["--marching", "--inflate"] {
        let output = out("vtu");
        let status = Command::new(BIN)
            .args([
                "mesh",
                "hex",
                "-i",
                sphere().to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                "--uniform",
                "0.2",
                "--pyramids",
                "0.3",
                flag,
            ])
            .arg("--quiet")
            .status()
            .expect("failed to spawn automesh");
        assert!(!status.success(), "pyramids accepted {flag}");
    }
}

#[test]
fn mesh_inflate_requires_uniform() {
    let output = out("vtu");
    let status = Command::new(BIN)
        .args([
            "mesh",
            "hex",
            "-i",
            sphere().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--inflate",
        ])
        .arg("--quiet")
        .status()
        .expect("failed to spawn automesh");
    assert!(!status.success(), "inflation accepted an octree background");
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
fn convert_mesh_off_to_inp() {
    let off = out("off");
    run(&[
        "mesh",
        "tri",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        off.to_str().unwrap(),
    ]);
    let inp = out("inp");
    run(&[
        "convert",
        "mesh",
        "-i",
        off.to_str().unwrap(),
        "-o",
        inp.to_str().unwrap(),
    ]);
    assert_nonempty(&inp);
}

#[test]
fn convert_mesh_off_rejects_hexahedra() {
    let exo = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        exo.to_str().unwrap(),
    ]);
    let off = out("off");
    let status = Command::new(BIN)
        .args([
            "convert",
            "mesh",
            "-i",
            exo.to_str().unwrap(),
            "-o",
            off.to_str().unwrap(),
        ])
        .arg("--quiet")
        .status()
        .expect("failed to spawn automesh");
    assert!(!status.success(), ".off accepted a hexahedral mesh");
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

fn assert_parts(output: &Path, parts: usize) {
    let width = parts.to_string().len();
    (0..parts).for_each(|rank| {
        assert_nonempty(&PathBuf::from(format!(
            "{}.{parts}.{rank:0width$}",
            output.display()
        )))
    });
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
        assert_parts(&output, 3);
    });
}

#[test]
fn partition_box_to_exo() {
    let source = out("exo");
    run(&[
        "mesh",
        "hex",
        "-i",
        input("letter_f_3d.npy").to_str().unwrap(),
        "-o",
        source.to_str().unwrap(),
    ]);
    let output = out("exo");
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
    assert_parts(&output, 2);
}

#[test]
fn partition_rejects_non_exo_output() {
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
            out("vtu").to_str().unwrap(),
            "-n",
            "3",
        ])
        .output()
        .expect("failed to spawn automesh");
    assert!(!result.status.success());
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
fn partition_as_polyhedra_to_exo_and_vtu() {
    let source = hex_source();
    [("rcb", "exo"), ("rib", "exo"), ("rcb", "vtu")]
        .into_iter()
        .for_each(|(method, extension)| {
            let output = out(extension);
            run(&[
                "partition",
                "--as-polyhedra",
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
fn partition_as_polyhedra_box_to_exo() {
    let source = hex_source();
    let output = out("exo");
    run(&[
        "partition",
        "--as-polyhedra",
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
fn partition_as_polyhedra_rejects_unsupported_output() {
    let source = hex_source();
    let result = Command::new(BIN)
        .args([
            "partition",
            "--as-polyhedra",
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

/// The help printed for the given args.
fn help(args: &[&str]) -> String {
    let result = Command::new(BIN)
        .args(args)
        .arg("--help")
        .output()
        .expect("failed to spawn automesh");
    assert!(result.status.success(), "help failed: automesh {args:?}");
    String::from_utf8(result.stdout).expect("help is not utf-8")
}

/// The description of the input option, which is on the same or the next line of the help.
fn input_help(args: &[&str]) -> String {
    let text = help(args);
    let start = text.find("-i, --input <FILE>").expect("no input option");
    let mut lines = text[start..].lines();
    let description = lines.next().unwrap().split("<FILE>").nth(1).unwrap().trim();
    if description.is_empty() {
        lines.next().unwrap().trim().to_string()
    } else {
        description.to_string()
    }
}

#[test]
fn help_says_mesh_is_made_from_a_tessellation_too() {
    assert!(
        help(&[]).contains("Creates a finite element mesh from a segmentation or tessellation")
    );
}

#[test]
fn help_lists_mesh_among_the_mesh_inputs() {
    for args in [
        &["convert", "mesh"][..],
        &["metrics"],
        &["remesh"],
        &["segment"],
        &["smooth"],
    ] {
        assert!(
            input_help(args).contains("(exo | inp | mesh | off | stl | vtu)"),
            "automesh {args:?}"
        );
    }
    assert!(
        help(&["convert", "mesh"]).contains(
            "(exo | inp | mesh | off | stl | vtu) -> (exo | inp | mesh | off | stl | vtu)"
        )
    );
}

#[test]
fn mesh_help_names_the_inputs_each_mode_accepts() {
    let hex = input_help(&["mesh", "hex"]);
    assert!(hex.contains("Segmentation (npy | spn) or tessellation (stl)"));
    for mode in ["hexdom", "poly", "tet"] {
        let text = input_help(&["mesh", mode]);
        assert!(text.contains("Tessellation (stl) input file"), "{mode}");
        assert!(!text.contains("Segmentation"), "{mode}");
    }
    let tri = input_help(&["mesh", "tri"]);
    assert!(tri.contains("Segmentation (npy | spn) input file"));
    assert!(!tri.contains("tessellation"));
}

/// A hex mesh of the unit sphere, snapped onto it so that improve starts on the target surface.
fn snapped_sphere() -> PathBuf {
    let output = out("vtu");
    run(&[
        "mesh",
        "hex",
        "-i",
        sphere().to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-s",
        "5",
        "--snap",
    ]);
    output
}

/// The worst minimum scaled Jacobian after improve, with optional extra args.
fn improved_worst(input: &Path, extra: &[&str]) -> f64 {
    let (target, output, metrics) = (sphere(), out("vtu"), out("csv"));
    let mut args = vec![
        "improve",
        "-i",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        target.to_str().unwrap(),
        "--metrics",
        metrics.to_str().unwrap(),
    ];
    args.extend(extra);
    run(&args);
    std::fs::read_to_string(&metrics)
        .expect("metrics file was not created")
        .lines()
        .skip(1)
        .map(|line| line.split(',').nth(1).unwrap().parse::<f64>().unwrap())
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn improve_help_shows_the_sweeps_and_threshold_defaults() {
    let text = help(&["improve"]);
    assert!(text.contains("--sweeps <NUM>"));
    assert!(text.contains("[default: 20]"));
    assert!(text.contains("--threshold <SJ>"));
    assert!(text.contains("[default: 1]"));
}

#[test]
fn improve_rejects_zero_sweeps() {
    let (input, target, output) = (sphere(), sphere(), out("vtu"));
    let result = Command::new(BIN)
        .args([
            "improve",
            "-i",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            target.to_str().unwrap(),
            "--sweeps",
            "0",
        ])
        .output()
        .expect("failed to spawn automesh");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--sweeps <NUM>"));
    assert!(!output.exists());
}

#[test]
fn improve_with_one_sweep_stops_below_the_default() {
    let input = snapped_sphere();
    assert!(improved_worst(&input, &["--sweeps", "1"]) < improved_worst(&input, &[]));
}

/// The terminal output of improve on the given mesh and target, with optional extra args.
fn improve_output(input: &Path, target: &Path, extra: &[&str]) -> String {
    let output = out("vtu");
    let result = Command::new(BIN)
        .args([
            "improve",
            "-i",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            target.to_str().unwrap(),
        ])
        .args(extra)
        .output()
        .expect("failed to spawn automesh");
    assert!(result.status.success(), "improve failed");
    String::from_utf8_lossy(&result.stdout).into_owned()
}

#[test]
fn improve_reports_a_used_up_sweep_budget() {
    let text = improve_output(&snapped_sphere(), &sphere(), &["--sweeps", "1"]);
    assert!(text.contains("The scaled Jacobian pass used all 1 sweeps without reaching 0.66."));
}

#[test]
fn improve_reports_a_stall() {
    let (mesh, surface) = (out("vtu"), out("stl"));
    let segmentation = input("letter_f_3d.npy");
    for (subcommand, output) in [("hex", &mesh), ("tri", &surface)] {
        run(&[
            "mesh",
            subcommand,
            "-i",
            segmentation.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ]);
    }
    let text = improve_output(&mesh, &surface, &[]);
    assert!(text.contains("The scaled Jacobian pass stalled after 1 sweeps short of 0.23."));
}

#[test]
fn improve_stops_at_the_threshold() {
    let input = snapped_sphere();
    let text = improve_output(&input, &sphere(), &["--threshold", "0.6"]);
    assert!(text.contains("reached the threshold 0.60"));
    let worst = improved_worst(&input, &["--threshold", "0.6"]);
    assert!(worst >= 0.6 && worst < improved_worst(&input, &[]));
}

#[test]
fn improve_with_a_threshold_below_the_input_writes_it_unchanged() {
    let (input, target, output) = (snapped_sphere(), sphere(), out("vtu"));
    run(&[
        "improve",
        "-i",
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "-t",
        target.to_str().unwrap(),
        "--threshold",
        "0.5",
    ]);
    assert_eq!(
        std::fs::read(&input).unwrap(),
        std::fs::read(&output).unwrap()
    );
}

#[test]
fn improve_rejects_a_threshold_above_one() {
    let (input, target, output) = (sphere(), sphere(), out("vtu"));
    let result = Command::new(BIN)
        .args([
            "improve",
            "-i",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "-t",
            target.to_str().unwrap(),
            "--threshold",
            "1.5",
        ])
        .output()
        .expect("failed to spawn automesh");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("1.5 is not between 0 and 1"));
    assert!(!output.exists());
}

/// Runs `mesh <element>` on the sphere tessellation with extra flags, asserting failure.
fn assert_stl_rejects(element: &str, flags: &[&str]) {
    let output = out("vtu");
    let status = Command::new(BIN)
        .args([
            "mesh",
            element,
            "-i",
            sphere().to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .args(flags)
        .arg("--quiet")
        .status()
        .expect("failed to spawn automesh");
    assert!(!status.success(), "mesh {element} accepted {flags:?}");
}

#[test]
fn mesh_hex_stl_rejects_levels() {
    assert_stl_rejects("hex", &["-l", "3"]);
}

#[test]
fn mesh_tet_stl_rejects_levels() {
    assert_stl_rejects("tet", &["-l", "3"]);
}

#[test]
fn mesh_stl_rejects_segmentation_flags() {
    for element in ["hex", "tet", "hexdom", "poly"] {
        assert_stl_rejects(element, &["-r", "1", "2"]);
        assert_stl_rejects(element, &["-d", "5"]);
        assert_stl_rejects(element, &["-x", "3"]);
    }
}
