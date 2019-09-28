use crate::core;
use std::path::Path;

pub fn create_key(name: &str) -> Result<String, core::Error> {
    let name_path = format!("{}.key", name);
    let path = Path::new(&name_path);
    let path_display = format!("{}", path.display());
    let _key = core::Key::create().write_to_file(&path)?;
    Ok(path_display)
}
