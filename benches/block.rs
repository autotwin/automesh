#![feature(test)]

extern crate test;
use automesh::{Abaqus, Voxels};
use test::Bencher;

const REMOVE: Option<Vec<u8>> = None;
const SCALE: [f64; 3] = [1.0, 1.0, 1.0];
const TRANSLATE: [f64; 3] = [0.0, 0.0, 0.0];

macro_rules! bench_block {
    ($nel:expr) => {
        const NEL: [usize; 3] = [$nel, $nel, $nel];
        #[bench]
        fn calculate_neighboring_nodal_coordinates_average(
            bencher: &mut Bencher,
        ) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let mut fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            fem.calculate_node_element_connectivity()?;
            fem.calculate_node_node_connectivity()?;
            bencher.iter(|| {
                fem.calculate_neighboring_nodal_coordinates_average()
                    .unwrap()
            });
            Ok(())
        }
        #[bench]
        fn calculate_nodal_hierarchy(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let mut fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            fem.calculate_node_element_connectivity()?;
            fem.calculate_node_node_connectivity()?;
            bencher.iter(|| fem.calculate_nodal_hierarchy().unwrap());
            Ok(())
        }
        #[bench]
        fn calculate_node_element_connectivity(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let mut fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            bencher.iter(|| fem.calculate_node_element_connectivity().unwrap());
            Ok(())
        }
        #[bench]
        fn calculate_node_node_connectivity(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let mut fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            fem.calculate_node_element_connectivity()?;
            bencher.iter(|| fem.calculate_node_node_connectivity().unwrap());
            Ok(())
        }
        #[bench]
        fn calculate_node_node_connectivity_boundary(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let mut fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            fem.calculate_node_element_connectivity()?;
            fem.calculate_node_node_connectivity()?;
            fem.calculate_nodal_hierarchy()?;
            bencher.iter(|| fem.calculate_node_node_connectivity_boundary().unwrap());
            Ok(())
        }
        #[bench]
        fn calculate_node_node_connectivity_interior(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let mut fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            fem.calculate_node_element_connectivity()?;
            fem.calculate_node_node_connectivity()?;
            fem.calculate_nodal_hierarchy()?;
            bencher.iter(|| fem.calculate_node_node_connectivity_interior().unwrap());
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
            bencher.iter(|| Voxels::from_spn(&spn, NEL).unwrap());
        }
        #[bench]
        fn from_tif(bencher: &mut Bencher) {
            let tif = format!("benches/block/block_{}.tif", $nel);
            bencher.iter(|| Voxels::from_tif(&tif).unwrap());
        }
        #[bench]
        fn into_finite_elements_from_npy(bencher: &mut Bencher) {
            let npy = format!("benches/block/block_{}.npy", $nel);
            bencher.iter(|| {
                Voxels::from_npy(&npy)
                    .unwrap()
                    .into_finite_elements(REMOVE, &SCALE, &TRANSLATE)
                    .unwrap()
            });
        }
        #[bench]
        fn write_inp(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let fem = voxels.into_finite_elements(REMOVE, &SCALE, &TRANSLATE)?;
            let inp = format!("target/block_{}.inp", $nel);
            bencher.iter(|| fem.write_inp(&inp).unwrap());
            Ok(())
        }
        #[bench]
        fn write_npy(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let npy = format!("target/block_{}.npy", $nel);
            bencher.iter(|| voxels.write_npy(&npy).unwrap());
            Ok(())
        }
        #[bench]
        fn write_spn(bencher: &mut Bencher) -> Result<(), String> {
            let voxels = Voxels::from_spn(&format!("benches/block/block_{}.spn", $nel), NEL)?;
            let spn = format!("target/block_{}.spn", $nel);
            bencher.iter(|| voxels.write_spn(&spn).unwrap());
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
