use std::fs;

use anyhow::Error;
use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tap::{Conv, Pipe};
use tempfile::TempDir;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{archive::ArchiveContext, args::Args, trademark::TrademarkReply};

mod archive;
mod args;
mod trademark;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let temp = TempDir::new_in(&args.input)?;

    if args.archive {
        WalkDir::new(&args.input)
            .follow_root_links(false)
            .into_iter()
            .filter_map(|ent| ent.ok())
            .filter(|ent| ent.file_type().is_file())
            .filter(|ent| ent.path().extension().unwrap_or_default().eq("zip"))
            .map(|ent| ent.into_path())
            .collect::<Vec<_>>()
            .par_iter()
            .filter_map(|p| ArchiveContext::new(p).ok())
            .inspect(|reply| eprintln!("Unzip archive: {}", reply.origin_path.display()))
            .for_each(|mut arch| {
                let path = temp.path().join(Uuid::new_v4().to_string());
                fs::create_dir(&path)
                    .map_err(|e| Error::from(e))
                    .and_then(|_| arch.extract(&path))
                    .inspect_err(|e| eprintln!("{}", e))
                    .ok();
            });
    }

    WalkDir::new(&args.input)
        .follow_root_links(false)
        .into_iter()
        .filter_map(|ent| ent.ok())
        .filter(|ent| ent.file_type().is_file())
        .filter(|ent| ent.path().extension().unwrap_or_default().eq("pdf"))
        .map(|ent| ent.into_path())
        .collect::<Vec<_>>()
        .par_iter()
        .filter_map(|p| TrademarkReply::new(p))
        .inspect(|reply| eprintln!("Processing {}", reply.origin_path.display()))
        .map(|reply| {
            let inc = reply
                .gen_md5()
                .and_then(|reply| reply.move_file(args.mode, &args.output))
                .inspect_err(|e| eprintln!("{}", e))
                .is_ok()
                .conv::<i32>();

            (1, inc)
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
        .pipe(|(all, success)| eprintln!("Finish! All:{all}; Success:{success}"));

    Ok(())
}
