#![feature(test)]

extern crate test;
use automesh::{
    FiniteElementMethods, HexahedralFiniteElements, Octree, Smoothing, Tree, Voxels, NSD,
};
use std::{
    fs::{read_dir, remove_file},
    path::Path,
};
use test::Bencher;

const REMOVE: Option<Vec<u8>> = None;
const SCALE: [f64; NSD] = [1.0, 1.0, 1.0];
const TRANSLATE: [f64; NSD] = [0.0, 0.0, 0.0];

const SMOOTHING_ITERATIONS: usize = 1;
const SMOOTHING_PASS_BAND: f64 = 0.1;
const SMOOTHING_SCALE_DEFLATE: f64 = 0.6307;

macro_rules! remove_files_with_extension {
    ($ext: expr) => {
        let mut file;
        let mut extension;
        for path in read_dir("target/").unwrap() {
            file = path.unwrap().path();
            extension = Path::new(&file).extension().and_then(|ext| ext.to_str());
            if let Some($ext) = extension {
                remove_file(file).unwrap();
            }
        }
    };
}

macro_rules! bench_block {
    ($nel: expr) => {
        const NEL: [usize; 3] = [$nel, $nel, $nel];
        #[bench]
        fn laplacian(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            fem.node_node_connectivity()?;
            let node_node_connectivity = fem.get_node_node_connectivity();
            bencher.iter(|| fem.laplacian(node_node_connectivity));
            Ok(())
        }
        #[bench]
        fn nodal_hierarchy(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            fem.node_node_connectivity()?;
            bencher.iter(|| fem.nodal_hierarchy().unwrap());
            Ok(())
        }
        #[bench]
        fn nodal_influencers(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            fem.node_node_connectivity()?;
            fem.nodal_hierarchy()?;
            bencher.iter(|| fem.nodal_influencers());
            Ok(())
        }
        #[bench]
        fn node_element_connectivity(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            bencher.iter(|| fem.node_element_connectivity().unwrap());
            Ok(())
        }
        #[bench]
        fn node_node_connectivity(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            bencher.iter(|| fem.node_node_connectivity().unwrap());
            Ok(())
        }
        #[bench]
        fn from_inp(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let inp = format!("target/block_{}.inp", $nel);
            fem.write_inp(&inp).unwrap();
            bencher.iter(|| HexahedralFiniteElements::from_inp(&inp).unwrap());
            Ok(())
        }
        #[bench]
        fn from_npy(bencher: &mut Bencher) {
            let npy = format!("benches/block/block_{}.npy", $nel);
            bencher.iter(|| Voxels::from_npy(&npy).unwrap());
        }
        #[bench]
        fn from_spn(bencher: &mut Bencher) {
            let spn = format!("benches/block/block_{}.spn", $nel);
            bencher.iter(|| Voxels::from_spn(&spn, NEL.into()).unwrap());
        }
        #[bench]
        fn into_finite_elements_from_voxels(bencher: &mut Bencher) {
            let npy = format!("benches/block/block_{}.npy", $nel);
            bencher.iter(|| {
                Voxels::from_npy(&npy)
                    .unwrap()
                    .into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())
                    .unwrap()
            });
        }
        #[bench]
        fn octree_from_voxels_from_npy(bencher: &mut Bencher) {
            let npy = format!("benches/block/block_{}.npy", $nel);
            bencher.iter(|| Octree::from_voxels(Voxels::from_npy(&npy).unwrap()));
        }
        #[bench]
        fn set_prescribed_nodes(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            fem.node_node_connectivity()?;
            fem.nodal_hierarchy()?;
            fem.nodal_influencers();
            let prescribed_nodes = fem.get_boundary_nodes().clone();
            bencher.iter(|| {
                fem.set_prescribed_nodes(Some(prescribed_nodes.clone()), None)
                    .unwrap()
            });
            Ok(())
        }
        #[bench]
        fn smooth_laplace(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            fem.node_node_connectivity()?;
            fem.nodal_influencers();
            bencher.iter(|| {
                fem.smooth(Smoothing::Laplacian(
                    SMOOTHING_ITERATIONS,
                    SMOOTHING_SCALE_DEFLATE,
                ))
                .unwrap()
            });
            Ok(())
        }
        #[bench]
        fn smooth_taubin(bencher: &mut Bencher) -> Result<(), String> {
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            fem.node_element_connectivity()?;
            fem.node_node_connectivity()?;
            fem.nodal_influencers();
            bencher.iter(|| {
                fem.smooth(Smoothing::Taubin(
                    SMOOTHING_ITERATIONS,
                    SMOOTHING_SCALE_DEFLATE,
                    SMOOTHING_PASS_BAND,
                ))
                .unwrap()
            });
            Ok(())
        }
        #[bench]
        fn write_exo(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("exo");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let mut count = 0;
            bencher.iter(|| {
                fem.write_exo(&format!("target/block_{}_{}.exo", $nel, count))
                    .unwrap();
                count += 1;
            });
            Ok(())
        }
        #[bench]
        fn write_inp(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("inp");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let mut count = 0;
            bencher.iter(|| {
                fem.write_inp(&format!("target/block_{}_{}.inp", $nel, count))
                    .unwrap();
                count += 1;
            });
            Ok(())
        }
        #[bench]
        fn write_mesh(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("mesh");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let mut count = 0;
            bencher.iter(|| {
                fem.write_mesh(&format!("target/block_{}_{}.mesh", $nel, count))
                    .unwrap();
                count += 1
            });
            Ok(())
        }
        #[bench]
        fn write_metrics_csv(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("csv");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let mut count = 0;
            bencher.iter(|| {
                fem.write_metrics(&format!("target/block_{}_{}.csv", $nel, count))
                    .unwrap();
                count += 1
            });
            Ok(())
        }
        #[bench]
        fn write_metrics_npy(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("npy");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let mut count = 0;
            bencher.iter(|| {
                fem.write_metrics(&format!("target/block_{}_{}.npy", $nel, count))
                    .unwrap();
                count += 1
            });
            Ok(())
        }
        #[bench]
        fn write_npy(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("npy");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut count = 0;
            bencher.iter(|| {
                voxels
                    .write_npy(&format!("target/block_{}_{}.npy", $nel, count))
                    .unwrap();
                count += 1;
            });
            Ok(())
        }
        #[bench]
        fn write_spn(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("spn");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let mut count = 0;
            bencher.iter(|| {
                voxels
                    .write_spn(&format!("target/block_{}_{}.spn", $nel, count))
                    .unwrap();
                count += 1;
            });
            Ok(())
        }
        #[bench]
        fn write_vtk(bencher: &mut Bencher) -> Result<(), String> {
            remove_files_with_extension!("vtk");
            let voxels =
                Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL.into())?;
            let fem = voxels.into_finite_elements(REMOVE, SCALE.into(), TRANSLATE.into())?;
            let mut count = 0;
            bencher.iter(|| {
                fem.write_vtk(&format!("target/block_{}_{}.vtk", $nel, count))
                    .unwrap();
                count += 1;
            });
            Ok(())
        }
    };
}

mod block_8 {
    use super::*;
    bench_block!(8);
}

mod block_16 {
    use super::*;
    bench_block!(16);
}

mod block_32 {
    use super::*;
    bench_block!(32);
}
