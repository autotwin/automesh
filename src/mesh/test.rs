use super::transformed_name;

#[test]
fn transformed_name_records_the_transform_exactly() {
    let scale = [0.001, 1e-7, 0.30000000000000004];
    let translate = [-12.5, 0.0, 1.0 / 3.0];
    let name = transformed_name("bone", scale, translate);
    let fields = name
        .strip_prefix("bone_scale_")
        .unwrap()
        .strip_suffix(".stl")
        .unwrap()
        .split('_')
        .collect::<Vec<_>>();
    assert_eq!(fields[3], "translate");
    let parsed = [
        fields[0], fields[1], fields[2], fields[4], fields[5], fields[6],
    ]
    .map(|field| field.parse::<f64>().unwrap().to_bits());
    let applied = [scale, translate].concat();
    assert_eq!(
        parsed.to_vec(),
        applied
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
}

#[test]
fn transformed_name_matches_the_issue_example() {
    assert_eq!(
        transformed_name("bone", [0.001; 3], [0.0, 0.0, -12.5]),
        "bone_scale_0.001_0.001_0.001_translate_0_0_-12.5.stl"
    );
}
