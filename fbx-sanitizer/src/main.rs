use clap::Arg;
use fbxcel_dom::any::AnyDocument;
use fbxcel_dom::fbxcel::low::v7400::AttributeValue;
use fbxcel_dom::v7400::object::model::TypedModelHandle;
use fbxcel_dom::v7400::object::{ObjectId, TypedObjectHandle};
use fbxcel_dom::v7400::Document;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod checks;
mod utils;

use crate::utils::print_children;
use checks::bounding_box_size;
use checks::correct_coordinate_axis;
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
        .filter_level(log::LevelFilter::Error)
        .init();

    let cli_matches = clap::App::new("FBX Unity Sanitizer")
        .version("1.0")
        .author("John Austin")
        .about("Checks fbx files (binary only) to make sure they will import cleanly into Unity.")
        .arg(Arg::with_name("summary").long("summary").takes_value(false).help(
            "Outputs a one-line summary for each fbx file passed in, rather than all errors.",
        ))
        .arg(Arg::with_name("dump-structure")
                 .long("dump-structure")
                 .takes_value(false)
                 .help("Writes a loosely yaml-structured version of the binary file to <file>_structure.yml. Suitable for debugging and inspection."), )
        .arg(Arg::with_name("files").multiple(true).takes_value(true).help("A set of fbx files to analyze.").required(true))
        .get_matches();

    // let fbx_file = Path::new(
    //     r"C:\Projects\Clockwork\CloningMain\Assets\Game\Environment\Gardening\Pots\Pot3.fbx",
    // );
    //

    // let fbx_file = Path::new(r"C:\Projects\Clockwork\CloningMain\Assets");

    // let directory_files = WalkDir::new(files)
    //     .follow_links(true)
    //     .into_iter()
    //     .filter_map(|e| e.ok());

    let files: Vec<&Path> = cli_matches
        .values_of("files")
        .unwrap()
        .map(|f| Path::new(f))
        .collect();

    let mut any_errs = false;

    for path in files {
        let f_name_opt = path.file_name();

        let f_name = match f_name_opt {
            None => {
                log::error!("File path was not a valid fbx: {}", path.display());
                continue;
            }
            Some(f) => f,
        };

        let f_name = f_name.to_string_lossy().clone();

        if f_name.ends_with(".fbx") {
            let result = check_fbx_file(&path.to_path_buf(), &cli_matches);

            if cli_matches.is_present("dump-structure") {}

            match result {
                Err(e) => {
                    log::warn!("Could not parse fbx: {:?}", path);
                    log::warn!("{:?}", e);
                }
                Ok(success) => {
                    any_errs |= !success;
                }
            }
        }
    }

    if any_errs {
        std::process::exit(1);
    }
}

/// Runs checks on the fbx file at the specified path.
/// Returns true if there were no errors.
fn check_fbx_file(path: &PathBuf, args: &clap::ArgMatches) -> Result<bool, anyhow::Error> {
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
                // Write out a loose yaml-like file for debugging.
                if args.is_present("dump-structure") {
                    let stem = path.file_stem().unwrap().to_str().unwrap();
                    let mut yml_output = path.to_owned();
                    yml_output.set_file_name(format!("{}_output.yml", stem));
                    let mut writer = BufWriter::new(
                        File::create(&yml_output).expect("Failed to open output file"),
                    );
                    print_children(&mut writer, &doc.tree().root(), 0)?;
                    log::info!(
                        "Dumped {} struct to {}",
                        path.display(),
                        yml_output.display()
                    );
                }

                // Apply each error checker.

                // Disabled for now. Maya has to output file that are not in Meters, in order to
                // import into Unity without a scale.
                // errors
                //     .entry("Units not in meters")
                //     .or_insert(vec![])
                //     .extend(units_are_in_meters::verify(&doc));
                errors
                    .entry("Incorrect axis")
                    .or_insert(vec![])
                    .extend(correct_coordinate_axis::verify(&doc)?);
                errors
                    .entry("Root does not have zero transform")
                    .or_insert(vec![])
                    .extend(root_has_identity_transform::verify(&doc)?);
                errors
                    .entry("Mesh size is wrong")
                    .or_insert(vec![])
                    .extend(bounding_box_size::verify(&doc)?);
                errors
                    .entry("No normals")
                    .or_insert(vec![])
                    .extend(meshes_have_normals::verify(&doc)?);
                errors
                    .entry("Contains quads")
                    .or_insert(vec![])
                    .extend(no_quads::verify(&doc)?);
                // Disabled for now, until we can find a better way to report warnings.
                // errors
                //     .entry("Bad mesh naming")
                //     .or_insert(vec![])
                //     .extend(mesh_naming::verify(&doc)?);

                // This check is currently disabled. See documentation, unity does not support this path.
                // errors
                //     .entry("Incorrect Axis")
                //     .or_insert(vec![])
                //     .extend(blender_exports_have_correct_axis::verify(&doc)?);
            }
            _ => panic!("Got FBX document of unsupported version"),
        }
    }

    // Print output
    let total_errors: usize = errors.iter().map(|(_, errors)| errors.len()).sum();
    if args.is_present("summary") {
        let issues = errors
            .iter()
            .filter(|(issue, errors)| errors.len() > 0)
            .map(|(issue, errors)| issue)
            .join(",");
        if total_errors > 0 {
            log::error!("{},{},{}", path.display(), total_errors, issues);
        }
    } else {
        if total_errors > 0 {
            log::error!("The file {} has {} errors:", path.display(), total_errors);
            for (_issue, errors) in errors {
                for error in errors {
                    log::error!("{}", error);
                }
            }
            println!();
        }
    }

    Ok(total_errors == 0)
}
