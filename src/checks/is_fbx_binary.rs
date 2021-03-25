use content_inspector::ContentType::BINARY;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn verify(path: &PathBuf) -> anyhow::Result<bool> {
    let mut bytes = Vec::<u8>::new();
    File::open(path)?.read_to_end(&mut bytes)?;
    let t = content_inspector::inspect(&bytes);
    Ok(t == BINARY)
}
