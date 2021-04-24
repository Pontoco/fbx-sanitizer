use crate::utils::{get_application_name, ApplicationName};
use fbxcel_dom::v7400::Document;

/// If units are not in meters, Unity will apply a scale when loading.
/// If units are not set at all, generates an error because some software will assume CM (Blender) while others will not (Unity).
#[allow(clippy::float_cmp)]
pub fn verify(doc: &Document) -> Vec<String> {
    let correct_unit: f64 = match get_application_name(doc) {
        Some(ApplicationName::Maya) => 1f64, // Maya cannot export in meters properly. This is the cm 'hack'
        Some(ApplicationName::Houdini) => 1f64, // Houdini cannot export in anything but centimeters. We use the same 'cm' hack.
        Some(_) | None => 100f64,               // all other applications should export meters
    };

    let file_unit = match doc.global_settings() {
        Some(settings) => settings.unit_scale_factor(),
        None => 1.0,
    };

    if file_unit != correct_unit {
        return vec![format!(
            "File is not in the correct units. Units: {}cm. Should be {}cm.",
            file_unit, correct_unit
        )];
    }

    vec![]
}
