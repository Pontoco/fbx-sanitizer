use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_NAMESPACE: Regex = Regex::new(r"^(.*):.*$").unwrap();
}

/// Depending on your setup, Maya may or may not export namespaces within the names of objects in a
/// model. This can result in mismatches if only certain meshes have namespaces. For example, exporting
/// a rig with namespaces, but an animation pointing at that rig *without* namespaces.
///
/// This check ensures that no namespaces are exported with a mesh. This is a sensible default as
/// namespaces are unnecessary in Unity, and just add noise.
pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    for obj in doc.objects() {
        if let TypedObjectHandle::Model(model) = obj.get_typed() {
            let name = model.name();
            match name {
                None => {}
                Some(name) => {
                    if RE_NAMESPACE.is_match(name) {
                        errors.push(format!("Objects should not be exported with namespaces: [{name}]"))
                    }
                }
            }

        }
    }

    Ok(errors)
}
