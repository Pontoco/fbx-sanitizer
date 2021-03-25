use fbxcel_dom::fbxcel::low::v7400::AttributeValue;
use fbxcel_dom::v7400::Document;

pub fn verify(doc: &Document) -> Vec<String> {
    const METERS_SCALE: f64 = 100.0f64;

    let settings = match doc.global_settings() {
        Some(s) => s,
        None => return vec!["File has no units. (No GlobalSettings)".into()],
    };

    let scale_factor_prop = match settings.get_property("UnitScaleFactor") {
        None => return vec!["File has no units. (No UnitScaleFactor)".into()],
        Some(p) => p,
    };

    let scale_factor_value = match scale_factor_prop.value_part().get(0) {
        None => return vec!["File has no units. (No Unit Value)".into()],
        Some(v) => v,
    };

    if let AttributeValue::F64(scale_factor) = scale_factor_value {
        if *scale_factor != METERS_SCALE {
            return vec![format!(
                "File is not in meter units. Units: {}cm. Should be 100.0cm.",
                *scale_factor
            )];
        }
    }

    vec![]
}
