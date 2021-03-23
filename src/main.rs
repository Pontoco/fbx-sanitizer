use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::format_err;
use cgmath::{Zero};
use fbxcel_dom::fbxcel::low::v7400::AttributeValue;
use fbxcel_dom::fbxcel::tree::v7400::NodeHandle;
use fbxcel_dom::v7400::object::model::TypedModelHandle;
use fbxcel_dom::v7400::object::{ObjectId, TypedObjectHandle};
use fbxcel_dom::v7400::Document;
use fbxcel_dom::{any::AnyDocument, v7400::object::property::PropertiesHandle};
use mint::Vector3;
use walkdir::WalkDir;

fn main() {
    let model_name = "suzanne_283";
    let mut writer = BufWriter::new(
        File::create(model_name.to_owned() + ".yml").expect("Failed to open output file"),
    );

    // for file in WalkDir::new(r".")
    // for file in WalkDir::new(r"C:\Projects\FBX_Import_Testing\Assets\".to_owned() + model_name + ".fbx")
    for file in WalkDir::new(
        r"C:\Projects\Clockwork\CloningMain\Assets\Game\Environment\Kitchenette\Bowl\Bowl.fbx",
    )
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
        AnyDocument::V7400(_, doc) => {
            print_children(writer, &doc.tree().root(), 0)?;
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
            print_children(writer, &doc.tree().root(), 5)?;
        }
        _ => panic!("Got FBX document of unsupported version"),
    }

    Ok(())
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

fn verify_units_are_in_meters(doc: &Document) -> Option<String> {
    const METERS_SCALE: f64 = 100.0f64;

    let settings = match doc.global_settings() {
        Some(s) => s,
        None => return Some("File has no units. (No GlobalSettings)".into()),
    };

    let scale_factor_prop = match settings.get_property("UnitScaleFactor") {
        None => return Some("File has no units. (No UnitScaleFactor)".into()),
        Some(p) => p,
    };

    let scale_factor_value = match scale_factor_prop.value_part().get(0) {
        None => return Some("File has no units. (No Unit Value)".into()),
        Some(v) => v,
    };

    if let AttributeValue::F64(scale_factor) = scale_factor_value {
        if *scale_factor != METERS_SCALE {
            return Some(format!(
                "File is not in meter units. Units: {}cm. Should be 100.0cm.",
                *scale_factor
            ));
        }
    }

    None
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

#[allow(unused)]
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
            let axis = get_coordinate_axis(doc)
                .ok_or_else(|| format_err!("Could not find coordinate axis."))?;

            let correct = CoordinateAxis {
                up: [0, 0, 1].into(),
                front: [0, 1, 0].into(),
                coord: [-1, 0, 0].into(),
            };

            if axis != correct {
                return Ok(vec![format!(
                    "File has incorrect Blender Coordinate Axis. Expected: [{:?}] actual [{:?}]",
                    correct, axis
                )]);
            }
        }
    }

    Ok(vec![])
}
// could be worth contributing, not sure

#[derive(Debug, PartialEq)]
struct CoordinateAxis {
    up: Vector3<i8>,
    front: Vector3<i8>,
    coord: Vector3<i8>,
}

fn get_coordinate_axis(doc: &Document) -> Option<CoordinateAxis> {
    let global_settings = doc
        .global_settings()
        .ok_or("Count not find global settings in file.")
        .ok()?;

    let up_axis = get_axis(&global_settings, "UpAxis")?;
    let front_axis = get_axis(&global_settings, "FrontAxis")?;
    let coord_axis = get_axis(&global_settings, "CoordAxis")?;

    Some(CoordinateAxis {
        up: up_axis,
        front: front_axis,
        coord: coord_axis,
    })
}

fn get_axis(global_settings: &PropertiesHandle, name: &str) -> Option<Vector3<i8>> {
    let axis =
        if let AttributeValue::I32(v) = global_settings.get_property(name)?.value_part().get(0)? {
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
        0 => [*sign as i8, 0, 0].into(),
        1 => [0, *sign as i8, 0].into(),
        2 => [0, 0, *sign as i8].into(),
        _ => return None,
    })
}

fn get_models(doc: &Document) -> impl Iterator<Item = TypedModelHandle<'_>> {
    return doc.objects().filter_map(|o| {
        if let TypedObjectHandle::Model(model) = o.get_typed() {
            return Some(model);
        }
        None
    });
}

fn get_model_roots(doc: &Document) -> Vec<TypedModelHandle<'_>> {
    let mut results: HashMap<ObjectId, TypedModelHandle> = HashMap::new();
    for model in get_models(doc) {
        let root = model.root_model();
        results.insert(root.object_id(), root);
    }

    results.values().cloned().collect()
}

// end

#[allow(unused)]
fn print_models(writer: &mut BufWriter<File>, doc: &Document, tabs: i32) {
    for object in doc.objects() {
        if let TypedObjectHandle::Model(model) = object.get_typed() {
            print_children(writer, &object.node(), 0);
        }
    }
}

fn print_children(
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
