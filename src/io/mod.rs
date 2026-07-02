use super::ErrorWrapper;
use conspire::{
    geometry::{
        grid::{Input as GridInput, Output as GridOutput, Voxels},
        mesh::{Input as MeshInput, Mesh, Output as MeshOutput, Tessellation},
    },
    io::Write,
};
use std::{path::Path, time::Instant};

pub fn extension(file: &str) -> Option<&str> {
    Path::new(file).extension().and_then(|ext| ext.to_str())
}

pub fn invalid_input(file: &str, extension: Option<&str>) -> ErrorWrapper {
    ErrorWrapper::from(format!(
        "Invalid extension .{} from input file {}",
        extension.unwrap_or("UNDEFINED"),
        file
    ))
}

pub fn invalid_output(file: &str, extension: Option<&str>) -> ErrorWrapper {
    ErrorWrapper::from(format!(
        "Invalid extension .{} from output file {}",
        extension.unwrap_or("UNDEFINED"),
        file
    ))
}

pub fn title(quiet: bool) {
    if !quiet {
        println!(
            "\x1b[1m    {} {}\x1b[0m",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    }
}

fn begin(verb: &str, file: &str, quiet: bool) -> Instant {
    crate::echo!(quiet, "     \x1b[1;96m{verb}\x1b[0m {file}");
    Instant::now()
}

fn done(time: Instant, quiet: bool) {
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
}

/// Reads a finite element mesh (exo | inp | mesh | stl | vtu) into a conspire mesh.
pub fn read_mesh(file: &str, quiet: bool, show_title: bool) -> Result<Mesh<3>, ErrorWrapper> {
    if show_title {
        title(quiet);
    }
    let time = begin("Reading", file, quiet);
    let extension = extension(file);
    let mesh = match extension {
        Some("inp") => Mesh::try_from(MeshInput::Abaqus(file))?,
        Some("exo") => Mesh::try_from(MeshInput::Exodus(file))?,
        Some("mesh") => Mesh::try_from(MeshInput::Medit(file))?,
        Some("vtu") => Mesh::try_from(MeshInput::Vtu(file))?,
        Some("stl") => Mesh::from(Tessellation::try_from(Path::new(file))?),
        _ => return Err(invalid_input(file, extension)),
    };
    done(time, quiet);
    Ok(mesh)
}

/// Writes a conspire mesh to a finite element file (exo | inp | mesh | vtu | stl).
pub fn write_mesh(file: &str, mesh: Mesh<3>, quiet: bool) -> Result<(), ErrorWrapper> {
    crate::echo!(quiet, "     \x1b[1;96mWriting\x1b[0m {file}");
    let time = Instant::now();
    let extension = extension(file);
    match extension {
        Some("inp") => mesh.write(MeshOutput::Abaqus(file))?,
        Some("exo") => mesh.write(MeshOutput::Exodus(file))?,
        Some("mesh") => mesh.write(MeshOutput::Medit(file))?,
        Some("vtu") => mesh.write(MeshOutput::Vtu(file))?,
        Some("stl") => Tessellation::from(mesh).write(file)?,
        _ => return Err(invalid_output(file, extension)),
    }
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    Ok(())
}

/// Resolves the voxels-per-direction needed to read an spn segmentation.
pub fn nel(
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
) -> Result<Vec<usize>, ErrorWrapper> {
    match (nelx, nely, nelz) {
        (Some(x), Some(y), Some(z)) => Ok(vec![x, y, z]),
        _ => Err(ErrorWrapper::from(
            "Need to specify nelx, nely, and nelz for an spn segmentation",
        )),
    }
}

/// Reads a segmentation (npy | spn) into voxels.
#[allow(clippy::too_many_arguments)]
pub fn read_segmentation(
    file: &str,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    quiet: bool,
    show_title: bool,
) -> Result<Voxels<u8>, ErrorWrapper> {
    if show_title {
        title(quiet);
    }
    let time = begin("Reading", file, quiet);
    let extension = extension(file);
    let voxels = match extension {
        Some("npy") => Voxels::<u8>::try_from(GridInput::Npy(file))?,
        Some("spn") => Voxels::<u8>::try_from(GridInput::Spn(file, nel(nelx, nely, nelz)?))?,
        _ => return Err(invalid_input(file, extension)),
    };
    let mut materials = [false; u8::MAX as usize + 1];
    voxels
        .data()
        .iter()
        .for_each(|&voxel| materials[voxel as usize] = true);
    let num_voxels = voxels.len();
    let num_materials = materials.iter().filter(|&&entry| entry).count();
    crate::echo!(
        quiet,
        "        \x1b[1;92mDone\x1b[0m {:?} \x1b[2m[{num_materials} materials, {num_voxels} voxels]\x1b[0m",
        time.elapsed()
    );
    Ok(voxels)
}

/// Writes a segmentation (npy | spn | vti).
pub fn write_segmentation(
    file: &str,
    voxels: &Voxels<u8>,
    quiet: bool,
) -> Result<(), ErrorWrapper> {
    crate::echo!(quiet, "     \x1b[1;96mWriting\x1b[0m {file}");
    let time = Instant::now();
    let extension = extension(file);
    match extension {
        Some("npy") => voxels.write(GridOutput::Npy(file))?,
        Some("spn") => voxels.write(GridOutput::Spn(file))?,
        Some("vti") => voxels.write(GridOutput::Vti(file))?,
        _ => return Err(invalid_output(file, extension)),
    }
    crate::echo!(quiet, "        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    Ok(())
}
