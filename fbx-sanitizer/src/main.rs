use clap::Arg;
use fbxcel_dom::any::AnyDocument;
use fbxcel_dom::fbxcel::low::v7400::AttributeValue;
use fbxcel_dom::v7400::object::model::TypedModelHandle;
use fbxcel_dom::v7400::object::{ObjectId, TypedObjectHandle};
use fbxcel_dom::v7400::Document;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod checks;
mod utils;

use checks::blender_exports_have_correct_axis;
use checks::bounding_box_size;
use checks::is_fbx_binary;
use checks::mesh_naming;
use checks::meshes_have_normals;
use checks::no_quads;
use checks::root_has_identity_transform;
use checks::units_are_in_meters;
use itertools::Itertools;

fn main() {
    // Custom logging formatting: "[ERROR] Error text."
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    let cli_matches = clap::App::new("FBX Unity Sanitizer")
        .version("1.0")
        .author("John Austin")
        .about("Checks fbx files (binary only) to make sure they will import cleanly into Unity.")
        .arg(Arg::with_name("summary").long("summary").takes_value(false).help(
            "Outputs a one-line summary for each fbx file passed in, rather than all errors.",
        ))
        .get_matches();

    // let fbx_file = Path::new(
    //     r"C:\Projects\Clockwork\CloningMain\Assets\Game\Environment\Gardening\Pots\Pot3.fbx",
    // );
    //
    // let stem = fbx_file.file_stem().unwrap().to_str().unwrap();
    // let mut yml_output = fbx_file.to_owned();
    // yml_output.set_file_name(format!("{}_output.yaml", stem));
    //
    // let mut writer = BufWriter::new(File::create(yml_output).expect("Failed to open output file"));

    let fbx_file = Path::new(r"C:\Projects\Clockwork\CloningMain\Assets");

    if cli_matches.is_present("summary") {}

    for file in WalkDir::new(fbx_file)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = file.file_name().to_string_lossy().clone();
        let path = file.clone().into_path();

        if f_name.ends_with(".fbx") {
            let result = check_fbx_file(&path, &cli_matches);
            if let Err(e) = result {
                log::warn!("Could not parse fbx: {:?}", path);
                log::warn!("{:?}", e);
            }
        }
    }
}

fn check_fbx_file(path: &PathBuf, args: &clap::ArgMatches) -> Result<(), anyhow::Error> {
    // println!("Parsing file: {}", path.display());
    let file = File::open(path).expect("Failed to open file.");

    // You can also use raw `file`, but do buffering for better efficiency.
    let reader = BufReader::new(file);
    let mut errors = IndexMap::<&str, Vec<String>>::new();

    // Check file
    if !is_fbx_binary::verify(path)? {
        errors
            .entry("ASCII Format")
            .or_insert(vec![])
            .push("File is not saved in FBX binary format.".to_owned());
    } else {
        match AnyDocument::from_seekable_reader(reader)? {
            AnyDocument::V7400(_, doc) => {
                errors
                    .entry("Units not in meters")
                    .or_insert(vec![])
                    .extend(units_are_in_meters::verify(&doc));
                errors
                    .entry("No normals")
                    .or_insert(vec![])
                    .extend(meshes_have_normals::verify(&doc)?);
                errors
                    .entry("Root does not have zero transform")
                    .or_insert(vec![])
                    .extend(root_has_identity_transform::verify(&doc)?);
                errors
                    .entry("Mesh size is wrong")
                    .or_insert(vec![])
                    .extend(bounding_box_size::verify(&doc)?);
                errors
                    .entry("Contains quads")
                    .or_insert(vec![])
                    .extend(no_quads::verify(&doc)?);
                errors
                    .entry("Bad mesh naming")
                    .or_insert(vec![])
                    .extend(mesh_naming::verify(&doc)?);

                // This check is currently disabled. If you're on a version of Blender <2.9?, you
                // will have to use the coordinate system in the FBX file to automatically 'counter-rotate'
                // the meshes, (to avoid applying a root rotation to the file).
                // errors
                //     .entry("Incorrect Axis")
                //     .or_insert(vec![])
                //     .extend(blender_exports_have_correct_axis::verify(&doc)?);

                // errors.extend(verify_blender_exports_have_correct_axis(&doc)?);
            }
            _ => panic!("Got FBX document of unsupported version"),
        }
    }

    // Print output
    if args.is_present("summary") {
        let issues = errors
            .iter()
            .filter(|(issue, errors)| errors.len() > 0)
            .map(|(issue, errors)| issue)
            .join(",");
        let total_errors: usize = errors.iter().map(|(_, errors)| errors.len()).sum();
        if total_errors > 0 {
            log::error!("{},{},{}", path.display(), total_errors, issues);
        }
    } else {
        if errors.len() > 0 {
            log::error!("The file {} has {} errors:", path.display(), errors.len());
            for (issue, errors) in errors {
                for error in errors {
                    log::error!("{} - {}", issue, error);
                }
            }
        }
    }

    Ok(())
}
