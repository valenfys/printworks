use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::formats;

/// A discovered RAW file paired with the directory its path should be
/// considered relative to (used to mirror directory structure into `--output`).
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub base: PathBuf,
}

pub fn discover(inputs: &[PathBuf], recursive: bool) -> Result<Vec<DiscoveredFile>> {
    let mut found = Vec::new();

    for input in inputs {
        let meta =
            std::fs::metadata(input).with_context(|| format!("reading {}", input.display()))?;

        if meta.is_file() {
            let base = input
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            found.push(DiscoveredFile {
                path: input.clone(),
                base,
            });
        } else if meta.is_dir() {
            let mut walker = WalkDir::new(input);
            if !recursive {
                walker = walker.max_depth(1);
            }
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && formats::is_raw(entry.path()) {
                    found.push(DiscoveredFile {
                        path: entry.path().to_path_buf(),
                        base: input.clone(),
                    });
                }
            }
        } else {
            anyhow::bail!("{}: not a file or directory", input.display());
        }
    }

    found.sort_by(|a, b| a.path.cmp(&b.path));
    found.dedup_by(|a, b| a.path == b.path);
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"").unwrap();
    }

    #[test]
    fn single_file_input_is_used_as_is_regardless_of_extension() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("shot.txt");
        touch(&file);

        let found = discover(std::slice::from_ref(&file), false).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, file);
        assert_eq!(found[0].base, dir.path());
    }

    #[test]
    fn directory_input_filters_by_raw_extension_non_recursive() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("a.cr2"));
        touch(&dir.path().join("b.jpg"));
        touch(&dir.path().join("sub/c.nef"));

        let found = discover(&[dir.path().to_path_buf()], false).unwrap();
        let names: Vec<_> = found
            .iter()
            .map(|f| f.path.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.cr2"]);
    }

    #[test]
    fn directory_input_recurses_when_requested() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("a.cr2"));
        touch(&dir.path().join("sub/c.nef"));

        let found = discover(&[dir.path().to_path_buf()], true).unwrap();
        let mut names: Vec<_> = found
            .iter()
            .map(|f| f.path.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a.cr2", "c.nef"]);
        assert!(found.iter().all(|f| f.base == dir.path()));
    }

    #[test]
    fn missing_input_errors() {
        let result = discover(&[PathBuf::from("/no/such/path/hopefully")], false);
        assert!(result.is_err());
    }
}
