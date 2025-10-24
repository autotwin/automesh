use super::ErrorWrapper;
use automesh::{FiniteElementMethods, Nel, Remove, Scale, Tessellation, Translate, Voxels};
use std::{path::Path, time::Instant};

pub fn invalid_input(file: &str, extension: Option<&str>) -> ErrorWrapper {
    ErrorWrapper {
        message: format!(
            "Invalid extension .{} from input file {}",
            extension.unwrap_or("UNDEFINED"),
            file
        ),
    }
}

pub struct Input<'a> {
    file: &'a str,
    nel: Nel,
    remove: Remove,
    scale: Scale,
    translate: Translate,
}

pub struct FiniteElementInput<const M: usize, const N: usize, T>(T)
where
    T: FiniteElementMethods<M, N> + From<Tessellation>;

impl<const M: usize, const N: usize, T> TryFrom<&str> for FiniteElementInput<M, N, T>
where
    T: FiniteElementMethods<M, N> + From<Tessellation>,
{
    type Error = ErrorWrapper;
    fn try_from(file: &str) -> Result<Self, Self::Error> {
        let extension = Path::new(file).extension().and_then(|ext| ext.to_str());
        match extension {
            Some("exo") => Ok(Self(T::from_exo(file)?)),
            Some("inp") => Ok(Self(T::from_inp(file)?)),
            Some("stl") => Ok(Self(T::from(Tessellation::try_from(file)?))),
            _ => Err(invalid_input(file, extension)),
        }
    }
}

impl<const M: usize, const N: usize, T> TryFrom<String> for FiniteElementInput<M, N, T>
where
    T: FiniteElementMethods<M, N> + From<Tessellation>,
{
    type Error = ErrorWrapper;
    fn try_from(file: String) -> Result<Self, Self::Error> {
        Self::try_from(file.as_str())
    }
}

impl<'a> TryFrom<Input<'a>> for Voxels {
    type Error = ErrorWrapper;
    fn try_from(
        Input {
            file,
            nel,
            remove,
            scale,
            translate,
        }: Input,
    ) -> Result<Self, Self::Error> {
        let extension = Path::new(&file).extension().and_then(|ext| ext.to_str());
        match extension {
            Some("npy") => Ok(Voxels::from_npy(file, remove, scale, translate)?),
            Some("spn") => Ok(Voxels::from_spn(file, nel, remove, scale, translate)?),
            _ => Err(invalid_input(file, extension)),
        }
    }
}

pub fn read_finite_elements<const M: usize, const N: usize, T>(
    file: &str,
    quiet: bool,
    title: bool,
) -> Result<T, ErrorWrapper>
where
    T: FiniteElementMethods<M, N> + From<Tessellation>,
{
    let time = Instant::now();
    if !quiet {
        if title {
            println!(
                "\x1b[1m    {} {}\x1b[0m",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            );
        }
        print!("     \x1b[1;96mReading\x1b[0m {file}");
    }
    let finite_elements = FiniteElementInput::<M, N, T>::try_from(file)?.0;
    if !quiet {
        println!(
            "\x1b[0m\n        \x1b[1;92mDone\x1b[0m {:?}",
            time.elapsed()
        );
    }
    Ok(finite_elements)
}

#[allow(clippy::too_many_arguments)]
pub fn read_segmentation(
    file: String,
    nelx: Option<usize>,
    nely: Option<usize>,
    nelz: Option<usize>,
    remove: Remove,
    scale: Scale,
    translate: Translate,
    quiet: bool,
    title: bool,
) -> Result<Voxels, ErrorWrapper> {
    let time = Instant::now();
    if !quiet {
        if title {
            println!(
                "\x1b[1m    {} {}\x1b[0m",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            );
        }
        print!("     \x1b[1;96mReading\x1b[0m {file}");
    }
    let nel = match Path::new(&file).extension().and_then(|ext| ext.to_str()) {
        Some("spn") => Nel::from_input([nelx, nely, nelz])?,
        _ => Nel::default(),
    };
    let input = Input {
        file: file.as_str(),
        nel,
        remove,
        scale,
        translate,
    };
    let voxels = Voxels::try_from(input)?;
    if !quiet {
        let data = voxels.get_data();
        let mut materials = vec![false; u8::MAX as usize + 1];
        data.iter()
            .for_each(|&voxel| materials[voxel as usize] = true);
        let num_voxels = data.iter().count();
        let num_materials = materials.iter().filter(|&&entry| entry).count();
        print!(
            "\x1b[0m\n        \x1b[1;92mDone\x1b[0m {:?}",
            time.elapsed()
        );
        println!(" \x1b[2m[{num_materials} materials, {num_voxels} voxels]\x1b[0m",);
    }
    Ok(voxels)
}

pub fn read_tessellation(
    file: &str,
    quiet: bool,
    title: bool,
) -> Result<Tessellation, ErrorWrapper> {
    let time = Instant::now();
    if !quiet {
        if title {
            println!(
                "\x1b[1m    {} {}\x1b[0m",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            );
        }
        print!("     \x1b[1;96mReading\x1b[0m {file}");
    }
    let tessellation = Tessellation::try_from(file)?;
    if !quiet {
        println!(
            "\x1b[0m\n        \x1b[1;92mDone\x1b[0m {:?}",
            time.elapsed()
        );
    }
    Ok(tessellation)
}
