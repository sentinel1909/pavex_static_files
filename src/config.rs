// src/config.rs

// dependencies
use serde::Deserialize;
use std::borrow::Cow;
use std::path::PathBuf;

// struct type which represents configuration for a static file server
#[derive(Clone, Debug, Deserialize)]
pub struct StaticServerConfig {
    pub mount_path: Cow<'static, str>,
    pub root_dir: PathBuf,
    pub serve_index: bool,
}