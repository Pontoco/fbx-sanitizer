use crate::utils::get_model_roots;
use cgmath::Zero;
use fbxcel_dom::v7400::Document;

/// Verifies that files with a single root have identity rotation and scale.
/// Files with multiple roots will be imported with an empty parent in Unity. In those cases
/// non-identity transforms are ok.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    let roots = get_model_roots(&doc);

    // Only files with a single root model are considered for this check.
    if roots.len() != 1 {
        return Ok(errors);
    }

    for root in get_model_roots(&doc) {
        let name = root.name().unwrap_or("(object has no name)");

        // No translation implies a zero translation.
        if let Some(translate) = root.local_translation()? {
            let t: cgmath::Vector3<f64> = translate.into();
            if !t.eq(&cgmath::Vector3::<f64>::zero()) {
                errors.push(format!(
                    "The root object [{}] does not have a zero rotation. It has rotation: [{:?}]",
                    name, t
                ));
            }
        }

        // No rotation implies a zero rotation.
        if let Some(rot) = root.local_rotation()? {
            let r: cgmath::Vector3<f64> = rot.into();
            if !r.eq(&cgmath::Vector3::<f64>::zero()) {
                errors.push(format!(
                    "The root object [{}] does not have a zero rotation. It has rotation: [{:?}]",
                    name, r
                ));
            }
        }

        // No scale implies a scale of 1
        if let Some(scl) = root.local_scaling()? {
            let s: cgmath::Vector3<f64> = scl.into();
            if !s.eq(&cgmath::Vector3::<f64>::new(1f64, 1f64, 1f64)) {
                errors.push(format!(
                    "The root object [{}] does not have a scale of 1. It has scale: [{:?}]",
                    name, s
                ));
            }
        }
    }

    Ok(errors)
}
