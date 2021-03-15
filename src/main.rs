use mint::Vector3;
use cgmath::{InnerSpace, MetricSpace, Vector2, Zero};
use fbxcel_dom::fbxcel::tree::v7400::NodeHandle;
use fbxcel_dom::v7400::object::model::{ModelHandle, TypedModelHandle};
use fbxcel_dom::v7400::object::{
    property::loaders::PrimitiveLoader, ObjectHandle, ObjectId, TypedObjectHandle,
};
use fbxcel_dom::v7400::Document;
use fbxcel_dom::{any::AnyDocument, v7400::object::property::PropertiesHandle};
use std::borrow::Cow;
use std::collections::hash_map::{RandomState, Values};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::iter::FilterMap;
use std::ops::Deref;
use std::path::PathBuf;
use walkdir::WalkDir;

use anyhow::format_err;
use fbxcel_dom::fbxcel::low::v7400::AttributeValue;

fn main() {
    let mut writer =
        BufWriter::new(File::create("output.yml").expect("Failed to open output file"));

    // for file in WalkDir::new(r".")
    for file in WalkDir::new(r"C:\Projects\FBX_Import_Testing\Assets\testcube.fbx")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = file.file_name().to_string_lossy().clone();
        let path = file.clone().into_path();

        if f_name.ends_with(".fbx") {
            let result = check_fbx_file(&path, &mut writer);
            if let Err(e) = result {
                println!("Could not load fbx: {:?}", path);
                println!("{:?}", e);
            }
        }
    }
}

fn check_fbx_file(path: &PathBuf, writer: &mut BufWriter<File>) -> Result<(), anyhow::Error> {
    println!("Parsing file: {:?}", path);
    let file = File::open(path).expect("Failed to open file.");

    // You can also use raw `file`, but do buffering for better efficiency.
    let reader = BufReader::new(file);

    match AnyDocument::from_seekable_reader(reader)? {
        AnyDocument::V7400(fbx_ver, doc) => {
            let mut errors = vec![];
            errors.extend(verify_roots_have_identity_transform(&doc));
            errors.extend(verify_blender_exports_have_correct_axis(&doc)?);
            for e in errors {
                println!("{}", e);
            }
            print_children(writer, &doc.tree().root(), 5);
        }
        _ => panic!("Got FBX document of unsupported version"),
    }

    return Ok(());
}

fn verify_roots_have_identity_transform(doc: &Document) -> Vec<String> {
    let mut errors = vec![];

    for root in get_model_roots(&doc) {
        let name = root.name().unwrap_or("(object has no name)");

        // No rotation implies a zero rotation.
        if let Some(rot) = root.local_rotation() {
            let r: cgmath::Vector3<f64> = rot.into();
            if !r.eq(&cgmath::Vector3::<f64>::zero()) {
                errors.push(format!(
                    "The root object [{}] does not have a zero rotation. It has rotation: [{:?}]",
                    name, r
                ));
            }
        }

        // No scale implies a scale of 1
        if let Some(scl) = root.local_scale() {
            let s: cgmath::Vector3<f64> = scl.into();
            if !s.eq(&cgmath::Vector3::<f64>::zero()) {
                errors.push(format!(
                    "The root object [{}] does not have a scale of 1. It has scale: [{:?}]",
                    name, s
                ));
            }
        }
    }

    return errors;
}

// fn verify_units_are_in_meters(doc: &Document) -> Result<Vec<String>, anyhow::Error> {}

fn verify_blender_exports_have_correct_axis(doc: &Document) -> Result<Vec<String>, anyhow::Error> {
    let node = doc
        .tree()
        .root()
        .children_by_name("Creator")
        .next()
        .ok_or_else(|| format_err!("FBXHeaderExtension not found in file."))?;

    let creator = &node.attributes()[0];
    if let AttributeValue::String(c) = creator {
        if c.contains("Blender") {
            let axis = get_coordinate_axis(doc).ok_or(format_err!("Could not find coordinate axis."))?;

            let correct = CoordinateAxis {
                up: [0f64, 0f64, 1f64].into(),
                front: [0f64, 1f64, 0f64].into(),
                coord: [-1f64, 0f64, 0f64].into(),
            };

            if axis != correct {
                return Ok(vec![format!("File has incorrect Blender Coordinate Axis. Expected: [{:?}] actual [{:?}]", correct, axis)]);
            }
        }
    }

    return Ok(vec![]);
}
// could be worth contributing, not sure

#[derive(Debug, PartialEq)]
struct CoordinateAxis {
    up: Vector3<f64>,
    front: Vector3<f64>,
    coord: Vector3<f64>,
}

fn get_coordinate_axis(doc: &Document) -> Option<CoordinateAxis> {
    let global_settings = doc
        .global_settings()
        .ok_or("Count not find global settings in file.")
        .ok()?;

    let upAxis = get_axis(&global_settings, "UpAxis")?;
    let frontAxis = get_axis(&global_settings, "FrontAxis")?;
    let coordAxis = get_axis(&global_settings, "CoordAxis")?;

    Some(CoordinateAxis {
        up: upAxis,
        front: frontAxis,
        coord: coordAxis,
    })
}

fn get_axis(global_settings: &PropertiesHandle, name: &str) -> Option<Vector3<f64>> {
    let axis = if let AttributeValue::I32(v) = global_settings
        .get_property(name)?
        .value_part()
        .get(0)? {
        v
    } else {
        return None;
    };

    let sign = if let AttributeValue::I32(v) = global_settings
        .get_property(&(name.to_owned() + "Sign"))?
        .value_part()
        .get(0)?
    {
        v
    } else {
        return None;
    };

    Some(match axis {
        0 => [*sign as f64, 0f64, 0f64].into(),
        1 => [0f64, *sign as f64, 0f64].into(),
        2 => [0f64, 0f64, *sign as f64].into(),
        _ => return None
    })
}

fn get_models<'a>(doc: &'a Document) -> impl Iterator<Item=TypedModelHandle<'a>> {
    return doc.objects().filter_map(|o| {
        if let TypedObjectHandle::Model(model) = o.get_typed() {
            return Some(model);
        }
        return None;
    });
}

fn get_model_roots<'a>(doc: &'a Document) -> Vec<TypedModelHandle<'a>> {
    let mut results: HashMap<ObjectId, TypedModelHandle> = HashMap::new();
    for model in get_models(doc) {
        let root = model.root_model();
        results.insert(root.object_id(), root);
    }
    let vec = results.values().cloned().collect();
    return vec;
}

// end

fn print_models(writer: &mut BufWriter<File>, doc: &Document, tabs: i32) {
    for object in doc.objects() {
        if let TypedObjectHandle::Model(model) = object.get_typed() {
            print_children(writer, &object.node(), 0);
        }
    }
}

fn print_children(writer: &mut BufWriter<File>, node: &NodeHandle, tabs: i32) {
    write!(
        writer,
        "{:indent$}Name: {} - ",
        "",
        node.name(),
        indent = tabs as usize
    );
    for attr in node.attributes() {
        write!(writer, "{:?}  ", attr);
    }

    write!(writer, "\n");

    for child in node.children() {
        print_children(writer, &child, tabs + 2);
    }
}
