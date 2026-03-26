use anyhow::Error;
use clap::Parser;
use tap::Pipe;
use walkdir::WalkDir;

use crate::{args::Args, trademark::TrademarkReply};

mod args;
mod trademark;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    WalkDir::new(&args.input)
        .max_depth(1)
        .contents_first(false)
        .follow_links(false)
        .follow_root_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().unwrap_or_default().eq("pdf"))
        .fold((0, 0), |(mut all, mut success), entry| {
            TrademarkReply::new(entry.path())
                .ok_or(Error::msg("File name is not in format"))
                .and_then(|trademark| trademark.gen_md5())
                .and_then(|trademark| trademark.move_file(args.mode, &args.output))
                .inspect_err(|e| eprintln!("Error processing {}: {}", entry.path().display(), e))
                .is_ok()
                .then(|| success += 1);

            all += 1;
            (all, success)
        })
        .pipe(|(all, success)| eprintln!("Finish! Process:{all}; Sucess:{success}"));

    Ok(())
}
