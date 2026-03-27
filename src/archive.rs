use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::Error;
use tap::Pipe;
use zip::{ZipArchive, read::root_dir_common_filter};

pub struct ArchiveContext {
    pub origin_path: PathBuf,
    archive: ZipArchive<File>,
}

impl ArchiveContext {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let archive = fs::File::open(path)?.pipe(ZipArchive::new)?;
        let origin_path = path.to_owned();

        Ok(Self {
            origin_path,
            archive,
        })
    }

    pub fn extract(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        self.archive
            .extract_unwrapped_root_dir(&path, root_dir_common_filter)?;

        Ok(())
    }
}
