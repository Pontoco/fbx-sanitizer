use fbxcel_dom::fbxcel::low::v7400::AttributeValue;
use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;

/// Maya uses 'Scale Compensation' when scaling joints in an animation rig.
/// This means that when you scale a bone by 2x, children are *not* scaled. They are translated instead. This makes animating, more comfortable
/// because usually you don't want parent scales to translate to the children (ie. expanding the middle bone in a tale)
///
/// Unfortunately, this is encoded in the FBX file by embedding "Scale Compensation" as a counter-animation. All bones inherit scale from their parents,
/// and the scale compensation parameter adjusts the final scale after the fact to fix it up.
///
/// As far as I know, only Maya supports this property in their FBX importer. Blender, Unity, and 3DSMax do not support this attribute.
/// Instead child bones inherit the scale from their parents, causing wonky scaling issues.
///
/// This check requires scale compensation to be disabled on all bones.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    for obj in doc.objects() {
        if let TypedObjectHandle::Model(model) = obj.get_typed() {
            if let Some(props) = model.direct_properties() {
                if let Some(inherit_type_prop) = props.get_property("InheritType") {
                    if let AttributeValue::I32(inherit_type) =
                        inherit_type_prop.value_part().get(0).expect("no value found for attribute InheritType")
                    {
                        // InheritType 2 is used for scale compensation.
                        // See: https://help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_class_fbx_anim_curve_filter_scale_compensate_html
                        // See: https://help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_fbxtransforms_8h_source_html
                        if *inherit_type == 2 {
                            errors.push(format!("The bone [{}] has scale compensation enabled. Disable it in Maya before importing into Unity. (InheritType==eInheritRrs)", model.name().unwrap_or("(no name)")));
                        }
                    }
                }
            }
        }
    }

    Ok(errors)
}
