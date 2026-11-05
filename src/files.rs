use std::path::Path;

use crate::error::SorterError;

pub enum FileFormat {}

pub fn read_file(path: &Path) -> Result<(), SorterError> {
    let dir: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|f| match f {
            Ok(k) => Some(k),
            Err(e) => {
                eprintln!("Unable to read {e}");
                None
            }
        })
        .collect();

    for file in dir {
        let file_type = match file.file_type() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Skipping {:?}: {e}", file.path());
                continue;
            }
        };

        if file_type.is_file() {
            let file_name = file.file_name();
            if !file_name.to_string_lossy().starts_with(".") {
                println!("{:?}", file_name)
            }
        }
    }

    Ok(())
}
