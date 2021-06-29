use content_inspector::ContentType::BINARY;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Blender cannot load the FBX ASCII format. Reject files if they are not in the binary format.
pub fn verify(path: &Path) -> anyhow::Result<bool> {
    let mut bytes = Vec::<u8>::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    let t = content_inspector::inspect(&bytes);
    Ok(t == BINARY)
}
