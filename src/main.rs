use anyhow::Error;
use clap::Parser;
use tempfile::TempDir;
use trademark_reply_category::{
    archive::process_archives, args::Args, trademark::process_trademarks,
};

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let temp = TempDir::new_in(&args.input)?;

    if args.archive {
        process_archives(&args.input, &temp);
    }

    process_trademarks(&args.input, args.mode, &args.output);

    drop(temp);

    Ok(())
}
