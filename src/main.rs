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
use fbxcel_dom::v7400::object::geometry::TypedGeometryHandle;


fn main() {
    let model_name = "suzanne_283";
    let mut writer =
        BufWriter::new(File::create(model_name.to_owned() + ".yml").expect("Failed to open output file"));

    // for file in WalkDir::new(r".")
    // for file in WalkDir::new(r"C:\Projects\FBX_Import_Testing\Assets\".to_owned() + model_name + ".fbx")
    for file in WalkDir::new(r"C:\Projects\Clockwork\CloningMain\Assets\Game\Environment\Kitchenette\Bowl\Bowl.fbx")
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
            print_children(writer, &doc.tree().root(), 0);
            let mut errors = vec![];

            errors.extend(verify_roots_have_identity_transform(&doc)?);
            errors.extend(verify_units_are_in_meters(&doc));

            // This check is currently disabled. If you're on a version of Blender <2.9?, you
            // will have to use the coordinate system in the FBX file to automatically 'counter-rotate'
            // the meshes, (to avoid applying a root rotation to the file).
            // errors.extend(verify_blender_exports_have_correct_axis(&doc)?);

            for e in errors {
                println!("{}", e);
            }
            print_children(writer, &doc.tree().root(), 5);
        }
        _ => panic!("Got FBX document of unsupported version"),
    }

    return Ok(());
}

fn verify_roots_have_identity_transform(doc: &Document) -> anyhow::Result<Vec<String>> {
    let mut errors = vec![];

    for root in get_model_roots(&doc) {
        let name = root.name().unwrap_or("(object has no name)");

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
        if let Some(scl) = root.local_scale()? {
            let s: cgmath::Vector3<f64> = scl.into();
            if !s.eq(&cgmath::Vector3::<f64>::zero()) {
                errors.push(format!(
                    "The root object [{}] does not have a scale of 1. It has scale: [{:?}]",
                    name, s
                ));
            }
        }
    }

    return Ok(errors);
}

fn verify_units_are_in_meters(doc: &Document) -> Option<String> {
    let settings = match doc.global_settings() {
        Some(s) => s,
        None => return Some("File has no units. (No GlobalSettings)".into()),
    };

    let scale_factor_prop = match settings.get_property("UnitScaleFactor"){
        None => return Some("File has no units. (No UnitScaleFactor)".into()),
        Some(p) => p
    };

    let scale_factor_value = match scale_factor_prop.value_part().get(0) {
        None => return Some("File has no units. (No Unit Value)".into()),
        Some(v) => v,
    };

    if let AttributeValue::F64(scale_factor) = scale_factor_value {
        if *scale_factor != 100.0f64 {
            return Some(format!("File is not in meter units. Units: {}cm. Should be 100.0cm.", *scale_factor))
        }
    }

    return None;
}

// fn verify_bounding_box_size(doc: &Document) -> Option<String> {
//     for obj in doc.objects() {
//         if let TypedObjectHandle::Geometry(geo) = obj.get_typed() {
//             if let TypedGeometryHandle::Mesh(m) = geo {
//                 // m.polygon_vertices()
//             }
//         }
//     }
//
//     return None;
// }

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
                up: [0, 0, 1].into(),
                front: [0, 1, 0].into(),
                coord: [-1, 0, 0].into(),
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
    up: Vector3<bool>,
    front: Vector3<bool>,
    coord: Vector3<bool>,
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
