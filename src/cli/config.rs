use lazy_static::lazy_static;
use std::path::PathBuf;

use crate::utils::Effect;

lazy_static! {
    pub static ref HISTORY_FILE: Option<PathBuf> =
        dirs::cache_dir().effect(|dir| dir.push("rustcalc-history.txt"));
    pub static ref RCFILE: Option<PathBuf> =
        dirs::config_dir().effect(|dir| dir.push("rustcalc.rc"));
}

pub const DEFAULT_RCFILE: &str = include_str!("../../res/rustcalc.rc");
