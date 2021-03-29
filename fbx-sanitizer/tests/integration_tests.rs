use assert_cmd::Command;
use std::path::PathBuf;

#[test]
fn blender_fbx_passes() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/blender_export_good.fbx");
    command.args(&[d]);
    command.assert().success();
}

#[test]
fn blender_fbx_fails() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/blender_export_bad.fbx");
    command.args(&[d]);
    command.assert().failure();
}

#[test]
fn maya_fbx_passes() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/maya_export_good.fbx");
    command.args(&[d]);
    command.assert().success();
}

#[test]
fn maya_fbx_fails() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/maya_export_bad.fbx");
    command.args(&[d]);
    command.assert().failure();
}
#[test]
fn max_fbx_passes() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/max_export_good.fbx");
    command.args(&[d]);
    command.assert().success();
}

#[test]
fn max_fbx_fails() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/maya_export_bad.fbx");
    command.args(&[d]);
    command.assert().failure();
}
#[test]
fn captial_fbx_passes() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/blender_export_good_caps.FBX");
    command.args(&[d]);
    command.assert().success();
}

#[test]
fn capital_fbx_fails() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/blender_export_bad_caps.FBX");
    command.args(&[d]);
    command.assert().failure();
}
#[test]
fn wrong_extension() {
    let mut command = Command::cargo_bin("fbx_sanitizer").unwrap();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/blender_export_wrong_extension.foo");
    command.args(&[d]);
    command.assert().failure();
}
