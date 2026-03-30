use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Error;
use md5::{Digest, Md5};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use tap::{Conv, Pipe, Tap};

use crate::{args::Category, progress::BAR, utils::FileFinder};

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
        let content = fs::read(&self.origin_path)?;

        self.md5 = Md5::new()
            .tap_mut(|hasher| hasher.update(&content))
            .pipe(|hasher| format!("{:x}", hasher.finalize()));

        Ok(self)
    }

    pub fn move_file(self, mode: Category, dir: &PathBuf) -> Result<Self, Error> {
        let aim_dir = match mode {
            Category::Doc => dir.join(&self.document_name),
            Category::Id => dir.join(&self.trademark_id),
        };

        let name = format!(
            "{}_{}_{}_{}.pdf",
            self.document_name, self.trademark_id, self.application_id, self.md5
        );

        fs::create_dir_all(&aim_dir)?;
        fs::rename(&self.origin_path, aim_dir.join(name))?;

        Ok(self)
    }
}

pub fn process_trademarks(dir: &Path, mode: Category, output: &PathBuf) {
    FileFinder::new(dir)
        .by_ext("pdf")
        .collect::<Vec<_>>()
        .tap(|_| BAR.set_position(0))
        .tap(|v| BAR.set_length(v.len() as u64))
        .par_iter()
        .inspect(|_| BAR.inc(1))
        .filter_map(|p| TrademarkReply::new(p))
        .inspect(|reply| BAR.set_message(format!("Processing {}", reply.origin_path.display())))
        .map(|reply| {
            reply
                .gen_md5()
                .and_then(|reply| reply.move_file(mode, output))
                .inspect_err(|e| BAR.println(format!("{}", e)))
                .is_ok()
                .conv::<i32>()
                .pipe(|inc| (1, inc))
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
        .pipe(|(all, success)| {
            BAR.finish_with_message(format!("Finish! All:{all}; Success:{success}"))
        });
}
