use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about = "商标局回文分类器")]
pub struct Args {
    /// 输入文件夹
    #[arg(short, long, default_value = ".")]
    pub input: PathBuf,

    /// 输出文件夹
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// 分类模式
    #[arg(short, long, value_enum, default_value_t = Category::Id)]
    pub mode: Category,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Category {
    /// 文书类型
    Doc,
    /// 注册号
    Id,
}
