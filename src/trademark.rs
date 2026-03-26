use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Error;
use md5::{Digest, Md5};
use regex::Regex;
use tap::{Pipe, Tap};

use crate::args::Category;

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[（(][^）)]+[)）]").unwrap());

pub struct TrademarkReply {
    pub origin_path: PathBuf,
    document_name: String,
    trademark_id: String,
    application_id: String,
    md5: String,
}

impl TrademarkReply {
    pub fn new(origin_path: impl AsRef<Path>) -> Option<Self> {
        let origin_path = origin_path.as_ref().to_path_buf();

        let filename = origin_path
            .file_prefix()
            .and_then(|s| s.to_str())
            .map(|haystack| RE.replace_all(&haystack, ""))
            .unwrap_or_default();

        let parts = filename.split("_").collect::<Vec<_>>();

        let document_name = parts.get(0)?.to_string();
        let trademark_id = parts.get(1)?.to_string();
        let application_id = parts.get(2).unwrap_or(&"00000000").to_string();
        let md5 = parts.get(3).unwrap_or(&"MD5").to_string();

        Self {
            origin_path,
            document_name,
            trademark_id,
            application_id,
            md5,
        }
        .into()
    }

    pub fn gen_md5(mut self) -> Result<Self, Error> {
        let content = std::fs::read(&self.origin_path)?;

        self.md5 = Md5::new()
            .tap_mut(|hasher| hasher.update(&content))
            .pipe(|hasher| format!("{:x}", hasher.finalize()));

        Ok(self)
    }

    pub fn move_file(self, mode: Category, dir: &PathBuf) -> Result<(), Error> {
        let aim_dir = match mode {
            Category::Doc => dir.join(&self.document_name),
            Category::Id => dir.join(&self.trademark_id),
        };

        let name = format!(
            "{}_{}_{}_{}.pdf",
            self.document_name, self.trademark_id, self.application_id, self.md5
        );

        std::fs::create_dir_all(&aim_dir)?;
        std::fs::copy(&self.origin_path, aim_dir.join(name))?;
        std::fs::remove_file(&self.origin_path)?;

        Ok(())
    }
}
