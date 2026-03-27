use std::sync::LazyLock;

use indicatif::{ProgressBar, ProgressStyle};

pub static BAR: LazyLock<ProgressBar> = LazyLock::new(|| {
    let style = ProgressStyle::with_template("[{bar:60.cyan/blue}] {pos}/{len} \n{msg}")
        .unwrap()
        .progress_chars("#$=");

    ProgressBar::no_length().with_style(style)
});
