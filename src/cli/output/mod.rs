use super::ErrorWrapper;
use automesh::{FiniteElementMethods, Tessellation, Voxels};
use std::{path::Path, time::Instant};

pub fn invalid_output(file: &str, extension: Option<&str>) -> ErrorWrapper {
    ErrorWrapper {
        message: format!(
            "Invalid extension .{} from output file {}",
            extension.unwrap_or("UNDEFINED"),
            file
        ),
    }
}

pub fn validate_output(command: &str, file: &str) -> Result<(), ErrorWrapper> {
    let extension = Path::new(file).extension().and_then(|ext| ext.to_str());
    match command {
        "mesh" => match extension {
            Some("inp") | Some("exo") | Some("mesh") | Some("stl") | Some("vtk") => Ok(()),
            _ => Err(invalid_output(file, extension)),
        },
        _ => panic!(),
    }
}

pub fn write_finite_elements<const M: usize, const N: usize, T>(
    file: String,
    finite_elements: T,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N>,
    Tessellation: From<T>,
{
    let time = Instant::now();
    if !quiet {
        println!("     \x1b[1;96mWriting\x1b[0m {file}");
    }
    let extension = Path::new(&file).extension().and_then(|ext| ext.to_str());
    match extension {
        Some("inp") => finite_elements.write_inp(&file)?,
        Some("exo") => finite_elements.write_exo(&file)?,
        Some("mesh") => finite_elements.write_mesh(&file)?,
        Some("stl") => Tessellation::from(finite_elements).write_stl(&file)?,
        Some("vtk") => finite_elements.write_vtk(&file)?,
        _ => return Err(invalid_output(&file, extension)),
    }
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}

pub fn write_metrics<const M: usize, const N: usize, T>(
    fem: &T,
    output: String,
    quiet: bool,
) -> Result<(), ErrorWrapper>
where
    T: FiniteElementMethods<M, N>,
{
    let time = Instant::now();
    if !quiet {
        println!("     \x1b[1;96mMetrics\x1b[0m {output}");
    }
    fem.write_metrics(&output)?;
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}

pub fn write_segmentation(file: String, voxels: Voxels, quiet: bool) -> Result<(), ErrorWrapper> {
    let time = Instant::now();
    if !quiet {
        println!("     \x1b[1;96mWriting\x1b[0m {file}");
    }
    let extension = Path::new(&file).extension().and_then(|ext| ext.to_str());
    match extension {
        Some("npy") => voxels.write_npy(&file)?,
        Some("spn") => voxels.write_spn(&file)?,
        _ => return Err(invalid_output(&file, extension)),
    }
    if !quiet {
        println!("        \x1b[1;92mDone\x1b[0m {:?}", time.elapsed());
    }
    Ok(())
}
