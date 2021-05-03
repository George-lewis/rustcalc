use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref HISTORY_FILE: Option<PathBuf> = dirs::cache_dir().map(|mut dir| {
        dir.push("rustcalc-history.txt");
        dir
    });
    pub static ref RCFILE: Option<PathBuf> = dirs::config_dir().map(|mut dir| {
        dir.push("rustcalc.rc");
        dir
    });
}

pub const DEFAULT_RCFILE: &str = include_str!("../../res/rustcalc.rc");
