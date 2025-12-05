use super::{super::Coordinates, FiniteElementSpecifics, TriangularFiniteElements};

const EPSILON: f64 = 1.0e-14;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const RAD_TO_DEG: f64 = 1.0 / DEG_TO_RAD;

#[test]
fn triangular_unit_tests() {
    // https://autotwin.github.io/automesh/cli/metrics_triangular.html#unit-tests
    // The first twelve triangles come from
    // tests/input/single_valence_04_noise2.inp.
    // We use
    // tests/tesselation.rs::from_stl::file_single_valence_04_noise2()
    // test to generate the coordinates and connectivity
    // from the stl file.
    // The next triangle comes from
    // tests/input/one_facet.stl.
    // The next triangle is an equilateral triangle of side length 4.0.

    // Gold values are not from Cubit, which uses "Aspect Ratio" instead of Edge Ratio
    // Turns out these are NOT the same thing!

    // Gold values from ~/autotwin/automesh/sandbox/metrics.py
    let maximum_edge_ratios_gold = [
        1.5078464057882237,
        1.5501674700560748,
        1.7870232669806838,
        1.915231466534568,
        2.230231996264181,
        1.6226774766497245,
        1.240081839656528,
        1.3849480786032335,
        1.6058086747499203,
        1.4288836646598568,
        1.2752274437112696,
        1.4361231071914424,
        std::f64::consts::SQRT_2, // 1.4142135623730951,
        1.0,
        1.0,
        1.2559260603991087,
    ];

    // Gold values from ~/autotwin/automesh/sandbox/metrics.py
    let minimum_angles_gold_deg = [
        41.20248899996187,
        39.796107567803936,
        33.61245209189106,
        31.00176761760843,
        21.661723789672273,
        37.33286786833477,
        51.03508450304211,
        46.05826353883047,
        38.512721702731355,
        44.27219859255808,
        49.65307785734987,
        44.12050798480872,
        45.00000000000001,
        59.99999999999999,
        59.99999999999999,
        48.794845448004004,
    ];
    // Gold values from ~/autotwin/automesh/sandbox/metrics.py
    let maximum_skews_gold = [
        0.3132918500006357,
        0.33673154053660104,
        0.4397924651351493,
        0.4833038730398595,
        0.6389712701721287,
        0.3777855355277538,
        0.14941525828263144,
        0.23236227435282555,
        0.35812130495447764,
        0.2621300234573654,
        0.17244870237750215,
        0.26465820025318804,
        0.2499999999999999,
        1.1842378929335003e-16,
        1.1842378929335003e-16,
        0.18675257586659993,
    ];
    // Gold values from ~/autotwin/automesh/sandbox/metrics.py and verified with Cubit
    let element_areas_gold = [
        0.6095033546646715,
        0.5498378247859254,
        0.5694533921062239,
        0.40221065958198676,
        0.34186812150301454,
        0.5705779745135626,
        0.42437710997648254,
        0.44293952755957805,
        0.6481635557480845,
        0.7040835887875813,
        0.6678959888148756,
        0.5158240173499096,
        0.5,
        6.928203230275509,
        0.43301270189221946,
        3.27324023180972,
    ];

    let minimum_scaled_jacobians_gold = [
        0.7606268158630964,
        0.7390747445600853,
        0.6392105272305011,
        0.5947452772930936,
        0.4262299581513255,
        0.700261936023385,
        0.8978156650410265,
        0.8314372958409268,
        0.7190186170534589,
        0.8060594150976131,
        0.8800416071493331,
        0.8038676339586197,
        0.8164965809277261,
        1.0,
        1.0,
        0.8687454713083852,
    ];

    let element_node_connectivity = vec![
        [0, 1, 2], // single_valence_04_noise2.inp begin
        [3, 1, 4],
        [0, 5, 1],
        [3, 2, 1],
        [3, 0, 2],
        [3, 6, 0],
        [1, 7, 4],
        [5, 7, 1],
        [6, 7, 5],
        [0, 6, 5],
        [3, 4, 7],
        [6, 3, 7],    // single_valence_04_noise2.inp end
        [8, 9, 10],   // one_facet.stl
        [11, 12, 13], // equilateral triangle of side length 4.0
        [14, 15, 16], // equilateral triangle of side length 1.0
        [17, 18, 19], // tilt.stl
    ];

    let nodal_coordinates = Coordinates::from([
        [-0.2, 1.2, -0.1], // single_valence_04_noise2.inp begin
        [1.180501, 0.39199, 0.3254445],
        [0.1, 0.2, 0.3],
        [-0.001, -0.021, 1.002],
        [1.2, -0.1, 1.1],
        [1.03, 1.102, -0.25],
        [0.0, 1.0, 1.0],
        [1.01, 1.02, 1.03],               // single_valence_04_noise2.inp end
        [0.0, 0.0, 1.0],                  // one_facet.stl begin
        [0.0, 0.0, 0.0],                  // ...
        [1.0, 0.0, 0.0],                  // one_facet.stl end
        [-2.0, 0.0, 0.0],                 // equilateral with edge length 4.0 start
        [2.0, 0.0, 0.0],                  // ...
        [0.0, 2.0 * 3.0_f64.sqrt(), 0.0], // equilateral with edge length 4.0 end
        [-0.5, 0.0, 0.0],                 // equilateral with edge length 1.0 start
        [0.5, 0.0, 0.0],                  // ...
        [0.0, 3.0_f64.sqrt() / 2.0, 0.0], // equilateral with edge length 1.0 end
        [0.0, 1.0, 3.0],                  // tilt.stl begin
        [2.0, 0.0, 2.0],
        [1.0, 1.0 + 3.0_f64.sqrt(), 1.0], // tile.stl end
    ]);

    let element_blocks = element_node_connectivity.iter().map(|_| 1).collect();

    let block = TriangularFiniteElements::from((
        element_blocks,
        element_node_connectivity,
        nodal_coordinates,
    ));

    block
        .maximum_edge_ratios()
        .iter()
        .zip(maximum_edge_ratios_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    let minimum_angles = block.minimum_angles();

    let minimum_angles_deg: Vec<f64> = minimum_angles
        .iter()
        .map(|angle| angle * RAD_TO_DEG)
        .collect();

    minimum_angles_deg
        .iter()
        .zip(minimum_angles_gold_deg.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    block
        .maximum_skews()
        .iter()
        .zip(maximum_skews_gold.iter())
        .for_each(|(calculated, gold)| {
            assert!((calculated - gold).abs() < EPSILON,);
        });

    block
        .areas()
        .iter()
        .zip(element_areas_gold.iter())
        .for_each(|(calculated, gold)| assert!((calculated - gold).abs() < EPSILON,));

    block
        .minimum_scaled_jacobians()
        .iter()
        .zip(minimum_scaled_jacobians_gold.iter())
        .for_each(|(calculated, gold)| assert!((calculated - gold).abs() < EPSILON,));
}
