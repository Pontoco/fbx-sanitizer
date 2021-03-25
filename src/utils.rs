use fbxcel_dom::fbxcel::tree::v7400::NodeHandle;
use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;
use fbxcel_dom::v7400::object::model::TypedModelHandle;
use fbxcel_dom::v7400::object::{ObjectId, TypedObjectHandle};
use fbxcel_dom::v7400::Document;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

/// Returns a useful name for a geometry. Either it's own given name, or the name of the first
/// model that references this geometry.
pub fn geo_name<'a, 'b>(geo: &TypedGeometryHandle<'a>) -> anyhow::Result<&'a str> {
    let model_parent = geo
        .destination_objects()
        .filter(|obj| obj.label().is_none())
        .filter_map(|obj| obj.object_handle())
        .filter_map(|obj| match obj.get_typed() {
            TypedObjectHandle::Model(o) => Some(o),
            _ => None,
        })
        .next();

    if let Some(parent) = model_parent {
        Ok(parent.name().unwrap_or("(no name)"))
    } else {
        Ok(geo.name().unwrap_or("(no name)"))
    }
}

pub fn get_models(doc: &Document) -> impl Iterator<Item = TypedModelHandle<'_>> {
    return doc.objects().filter_map(|o| {
        if let TypedObjectHandle::Model(model) = o.get_typed() {
            return Some(model);
        }
        None
    });
}

/// Gets the roots models of the scene.
pub fn get_model_roots(doc: &Document) -> Vec<TypedModelHandle<'_>> {
    let mut results: HashMap<ObjectId, TypedModelHandle> = HashMap::new();
    for model in get_models(doc) {
        let root = model.root_model();
        results.insert(root.object_id(), root);
    }

    results.values().cloned().collect()
}
#[allow(unused)]
pub fn print_models(writer: &mut BufWriter<File>, doc: &Document, tabs: i32) {
    for object in doc.objects() {
        if let TypedObjectHandle::Model(model) = object.get_typed() {
            print_children(writer, &object.node(), 0);
        }
    }
}

pub fn print_children(
    writer: &mut BufWriter<File>,
    node: &NodeHandle,
    tabs: i32,
) -> anyhow::Result<()> {
    write!(
        writer,
        "{:indent$}Name: {} - ",
        "",
        node.name(),
        indent = tabs as usize
    )?;
    for attr in node.attributes() {
        write!(writer, "{:?}  ", attr)?;
    }

    writeln!(writer,)?;

    for child in node.children() {
        print_children(writer, &child, tabs + 2)?;
    }

    Ok(())
}
