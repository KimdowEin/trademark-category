use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::Error;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tap::{Pipe, Tap};
use tempfile::TempDir;
use uuid::Uuid;
use zip::{ZipArchive, read::root_dir_common_filter};

use crate::{progress::BAR, utils::FileFinder};

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

pub fn process_archives(dir: &Path, temp: &TempDir) {
    FileFinder::new(dir)
        .by_ext("zip")
        .collect::<Vec<_>>()
        .tap(|v| BAR.set_length(v.len() as u64))
        .par_iter()
        .inspect(|_| BAR.inc(1))
        .filter_map(|p| ArchiveContext::new(p).ok())
        .inspect(|reply| BAR.set_message(format!("Unzip archive: {}", reply.origin_path.display())))
        .for_each(|mut arch| {
            let path = temp.path().join(Uuid::new_v4().to_string());
            std::fs::create_dir(&path)
                .map_err(|e| Error::from(e))
                .and_then(|_| arch.extract(&path))
                .inspect_err(|e| BAR.println(format!("{}", e)))
                .ok();
        });
}
