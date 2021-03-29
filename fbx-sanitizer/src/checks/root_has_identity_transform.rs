use crate::utils::{get_creator, get_model_roots};
use cgmath::{AbsDiffEq, Zero};
use fbxcel_dom::v7400::Document;

/// Verifies that files with a single root have identity rotation and scale. Having 90 degree rotations
/// on all objects makes it very hard to use them in gameplay scripting.
///
/// Files with multiple roots will be imported with an empty parent in Unity. In those cases
/// non-identity transforms are ok.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    // Some tools like Max will output the correct transforms, with slight error, so we
    // check all of these using an epsilon.
    // However, we try to have as tight a bound as possible, so each epsilon is separate.
    const ROT_EPSILON: f64 = 0.000000000001f64; // 3ds max
    const SCL_EPSILON: f64 = 0.000000000001f64; // 3ds max
    const TRA_EPSILON: f64 = 0.000000000001f64; // 3ds max exports translates as small as this sometimes

    let mut errors = vec![];

    let roots = get_model_roots(&doc);

    // Only files with a single root model are considered for this check.
    if roots.len() != 1 {
        return Ok(errors);
    }
    for root in get_model_roots(&doc) {
        let name = root.name().unwrap_or("(object has no name)");

        // Note(john): Disabling translation check for now. It's not proved to be a big problem in Unity,
        // except having it spawn in a weird place when dragging it into the hierarchy.
        // No translation implies a zero translation.
        // if let Some(translate) = root.local_translation()? {
        //     let t: cgmath::Vector3<f64> = translate.into();
        //     if t.abs_diff_ne(&cgmath::Vector3::<f64>::zero(), TRA_EPSILON) {
        //         errors.push(format!(
        //             "The root object [{}] does not have a zero translation. It has translate: [{:?}]",
        //             name, t
        //         ));
        //     }
        // }

        // No rotation implies a zero rotation.
        if let Some(rot) = root.local_rotation()? {
            let r: cgmath::Vector3<f64> = rot.into();
            if r.abs_diff_ne(&cgmath::Vector3::<f64>::zero(), ROT_EPSILON) {
                errors.push(format!(
                    "The root object [{}] does not have a zero rotation. It has rotation: [{:?}]",
                    name, r
                ));
            }
        }

        // Pre-Rotation is set from 3DS Max on export, usually.
        // No rotation implies a zero rotation.
        // https://download.autodesk.com/us/fbx/20112/FBX_SDK_HELP/index.html?url=WS1a9193826455f5ff1f92379812724681e696651.htm,topicNumber=d0e7429
        if let Some(rot) = root.pre_rotation()? {
            let r: cgmath::Vector3<f64> = rot.into();
            if r.abs_diff_ne(&cgmath::Vector3::<f64>::zero(), ROT_EPSILON) {
                errors.push(format!(
                    "The root object [{}] does not have a zero rotation. It has pre-rotation: [{:?}]",
                    name, r
                ));
            }
        }

        // No scale implies a scale of 1
        if let Some(scl) = root.local_scaling()? {
            let s: cgmath::Vector3<f64> = scl.into();

            if s.abs_diff_ne(&cgmath::Vector3::<f64>::new(1f64, 1f64, 1f64), SCL_EPSILON) {
                errors.push(format!(
                    "The root object [{}] does not have a scale of 1. It has scale: [{:?}]",
                    name, s
                ));
            }
        }
    }

    Ok(errors)
}
