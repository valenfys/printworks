use std::path::Path;

/// Extensions of RAW formats `rawloader` knows how to decode.
const RAW_EXTENSIONS: &[&str] = &[
    "ari", "arw", "cr2", "crw", "dcr", "dcs", "dng", "erf", "iiq", "kdc", "mef", "mos", "mrw",
    "nef", "nrw", "orf", "pef", "raf", "rw2", "srw", "x3f",
];

pub fn is_raw(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| RAW_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_raw_extensions() {
        assert!(is_raw(Path::new("photo.CR2")));
        assert!(is_raw(Path::new("photo.nef")));
        assert!(is_raw(Path::new("photo.ARW")));
        assert!(is_raw(Path::new("dir/photo.dng")));
    }

    #[test]
    fn rejects_non_raw_extensions() {
        assert!(!is_raw(Path::new("photo.jpg")));
        assert!(!is_raw(Path::new("photo.png")));
        assert!(!is_raw(Path::new("noext")));
    }
}
