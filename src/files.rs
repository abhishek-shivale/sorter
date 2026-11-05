use std::fs::{self, DirEntry};
use std::io::{self, ErrorKind};
use std::path::Path;

use crate::error::SorterError;

#[derive(Debug, PartialEq)]
pub enum FileFormat {
    Image,
    Video,
    Audio,
    Document,
    Archive,
}

impl FileFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "heic" | "bmp" | "tiff" => {
                Some(Self::Image)
            }
            "mp4" | "mov" | "mkv" | "avi" | "webm" | "m4v" => Some(Self::Video),
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => Some(Self::Audio),
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "csv" => {
                Some(Self::Document)
            }
            "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => Some(Self::Archive),
            _ => None,
        }
    }

    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Image => "Images",
            Self::Video => "Videos",
            Self::Audio => "Audio",
            Self::Document => "Documents",
            Self::Archive => "Archives",
        }
    }
}

pub fn sort_dir(path: &Path, ignore: &[String]) -> Result<(), SorterError> {
    for file in visible_files(path)? {
        if is_ignored(&file.path(), ignore) {
            continue;
        }
        let Some(format) = FileFormat::from_path(&file.path()) else {
            continue;
        };
        if let Err(e) = move_into(&file, &path.join(format.dir_name())) {
            eprintln!("Skipping {:?}: {e}", file.path());
        }
    }

    Ok(())
}

fn is_ignored(path: &Path, ignore: &[String]) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|extension| {
        ignore
            .iter()
            .any(|i| i.trim_start_matches('.').eq_ignore_ascii_case(extension))
    })
}

fn move_into(file: &DirEntry, dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;

    let target = dir.join(file.file_name());
    // fs::rename silently replaces an existing target on Unix.
    if target.exists() {
        return Err(ErrorKind::AlreadyExists.into());
    }
    fs::rename(file.path(), &target)?;
    println!("Moved {:?} to {:?}", file.path(), target);
    Ok(())
}

fn visible_files(path: &Path) -> Result<Vec<DirEntry>, SorterError> {
    let dir: Vec<_> = fs::read_dir(path)?
        .filter_map(|f| match f {
            Ok(k) => Some(k),
            Err(e) => {
                eprintln!("Unable to read {e}");
                None
            }
        })
        .collect();

    let mut files = Vec::new();
    for file in dir {
        let file_type = match file.file_type() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Skipping {:?}: {e}", file.path());
                continue;
            }
        };

        if file_type.is_file() && !file.file_name().to_string_lossy().starts_with(".") {
            files.push(file);
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_format_case_insensitively() {
        assert_eq!(
            FileFormat::from_path(Path::new("a.JPG")),
            Some(FileFormat::Image)
        );
        assert_eq!(FileFormat::from_path(Path::new("a.pdf.part")), None);
        assert_eq!(FileFormat::from_path(Path::new("noextension")), None);
    }

    #[test]
    fn sorts_without_overwriting() {
        let root = std::env::temp_dir().join(format!("sorter-test-{}", std::process::id()));
        fs::create_dir_all(root.join("Documents")).unwrap();
        fs::write(root.join("photo.PNG"), "image").unwrap();
        fs::write(root.join("unknown.xyz"), "unknown").unwrap();
        fs::write(root.join(".hidden.png"), "hidden").unwrap();
        fs::write(root.join("report.pdf"), "new").unwrap();
        fs::write(root.join("Documents/report.pdf"), "old").unwrap();
        fs::write(root.join("keep.zip"), "ignored").unwrap();

        sort_dir(&root, &[".ZIP".to_string()]).unwrap();

        assert!(root.join("keep.zip").exists());
        assert!(!root.join("Archives").exists());
        assert!(root.join("Images/photo.PNG").exists());
        assert!(root.join("unknown.xyz").exists());
        assert!(root.join(".hidden.png").exists());
        assert_eq!(fs::read_to_string(root.join("report.pdf")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(root.join("Documents/report.pdf")).unwrap(),
            "old"
        );

        fs::remove_dir_all(&root).unwrap();
    }
}
