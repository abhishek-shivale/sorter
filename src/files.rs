use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::SorterError;

const MIN_FILE_AGE: Duration = Duration::from_secs(60);

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
        if !is_settled(&file) {
            continue;
        }
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

// A placeholder next to a .part file can be old while its download is still running.
fn is_settled(file: &DirEntry) -> bool {
    let mut part = file.path().into_os_string();
    part.push(".part");
    if Path::new(&part).exists() {
        return false;
    }

    file.metadata()
        .and_then(|metadata| metadata.modified())
        .is_ok_and(|modified| modified.elapsed().is_ok_and(|age| age >= MIN_FILE_AGE))
}

fn move_into(file: &DirEntry, dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;

    let target = free_target(dir, &file.path());
    fs::rename(file.path(), &target)?;
    println!("Moved {:?} to {:?}", file.path(), target);
    Ok(())
}

// fs::rename silently replaces an existing target on Unix.
fn free_target(dir: &Path, file: &Path) -> PathBuf {
    let file_name = file.file_name().unwrap_or_default();
    let mut stem = Path::new(file.file_stem().unwrap_or_default());
    if stem.extension().is_some_and(|e| e.eq_ignore_ascii_case("tar")) {
        stem = Path::new(stem.file_stem().unwrap_or_default());
    }
    let suffix = OsStr::from_bytes(&file_name.as_bytes()[stem.as_os_str().len()..]);

    let mut target = dir.join(file_name);
    let mut copy = 1;

    while target.exists() {
        let mut name = stem.as_os_str().to_os_string();
        name.push(format!(" ({copy})"));
        name.push(suffix);
        target = dir.join(name);
        copy += 1;
    }

    target
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

    fn write_old(path: &Path, text: &str) {
        fs::write(path, text).unwrap();
        let old = std::time::SystemTime::now() - Duration::from_secs(120);
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(old)
            .unwrap();
    }

    #[test]
    fn leaves_files_in_progress_until_settled() {
        let root = std::env::temp_dir().join(format!("sorter-settle-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("fresh.pdf"), "writing").unwrap();
        write_old(&root.join("ff.pdf"), "");
        write_old(&root.join("ff.pdf.part"), "data");

        sort_dir(&root, &[]).unwrap();

        assert!(root.join("fresh.pdf").exists());
        assert!(root.join("ff.pdf").exists());
        assert!(root.join("ff.pdf.part").exists());
        assert!(!root.join("Documents").exists());

        fs::rename(root.join("ff.pdf.part"), root.join("ff.pdf")).unwrap();
        write_old(&root.join("ff.pdf"), "data");
        sort_dir(&root, &[]).unwrap();

        assert_eq!(
            fs::read_to_string(root.join("Documents/ff.pdf")).unwrap(),
            "data"
        );
        assert!(root.join("fresh.pdf").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn clash_names_keep_full_extension() {
        let root = std::env::temp_dir().join(format!("sorter-clash-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let cases = [
            ("arc.tar.gz", "arc (1).tar.gz"),
            ("ARC.TAR.BZ2", "ARC (1).TAR.BZ2"),
            ("plain.tar", "plain (1).tar"),
            ("my.report.pdf", "my.report (1).pdf"),
            ("noextension", "noextension (1)"),
        ];
        for (name, expected) in cases {
            assert_eq!(free_target(&root, Path::new(name)), root.join(name));
            fs::write(root.join(name), "").unwrap();
            assert_eq!(free_target(&root, Path::new(name)), root.join(expected));
        }

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn sorts_without_overwriting() {
        let root = std::env::temp_dir().join(format!("sorter-test-{}", std::process::id()));
        fs::create_dir_all(root.join("Documents")).unwrap();
        write_old(&root.join("photo.PNG"), "image");
        write_old(&root.join("unknown.xyz"), "unknown");
        write_old(&root.join(".hidden.png"), "hidden");
        write_old(&root.join("report.pdf"), "new");
        fs::write(root.join("Documents/report.pdf"), "old").unwrap();
        fs::write(root.join("Documents/report (1).pdf"), "older").unwrap();
        write_old(&root.join("keep.zip"), "ignored");

        sort_dir(&root, &[".ZIP".to_string()]).unwrap();

        assert!(root.join("keep.zip").exists());
        assert!(!root.join("Archives").exists());
        assert!(root.join("Images/photo.PNG").exists());
        assert!(root.join("unknown.xyz").exists());
        assert!(root.join(".hidden.png").exists());
        assert!(!root.join("report.pdf").exists());
        assert_eq!(
            fs::read_to_string(root.join("Documents/report (2).pdf")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(root.join("Documents/report.pdf")).unwrap(),
            "old"
        );
        assert_eq!(
            fs::read_to_string(root.join("Documents/report (1).pdf")).unwrap(),
            "older"
        );

        fs::remove_dir_all(&root).unwrap();
    }
}
