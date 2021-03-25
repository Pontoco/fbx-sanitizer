use fbxcel_dom::v7400::object::TypedObjectHandle;
use fbxcel_dom::v7400::Document;
use lazy_static::lazy_static;
use regex::Regex;

// Invalid mesh names:
lazy_static! {
    static ref CUBE: Regex = Regex::new(r"^Cube\.\d+$").unwrap();
    static ref CYLINDER: Regex = Regex::new(r"^cylinder\d+$").unwrap();
    static ref CYLINDER2: Regex = Regex::new(r"^Cylinder$").unwrap();
}

pub fn verify(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors: Vec<String> = vec![];
    for o in doc.objects() {
        if let TypedObjectHandle::Model(model) = o.get_typed() {
            let name = if let Some(n) = model.name() {
                n
            } else {
                errors.push("Model has no name".into());
                continue;
            };

            if CUBE.is_match(name) || CYLINDER.is_match(name) || CYLINDER2.is_match(name) {
                errors.push(format!("The model [{}] has a default name. Please name it something more specific, so it can be found easily in the Unity Editor.", name))
            }
        }
    }

    return Ok(errors);
}
