use std::path::{Path, PathBuf};

use walkdir::{IntoIter, WalkDir};

pub struct FileFinder {
    walker: IntoIter,
}

impl FileFinder {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        let walker = WalkDir::new(dir).follow_root_links(false).into_iter();
        Self { walker }
    }
    pub fn by_ext(self, ext: &str) -> impl Iterator<Item = PathBuf> {
        self.walker
            .filter_map(|ent| ent.ok())
            .filter(|ent| ent.file_type().is_file())
            .filter(|ent| {
                ent.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .eq(&Some(ext))
            })
            .map(|ent| ent.into_path())
    }
}
