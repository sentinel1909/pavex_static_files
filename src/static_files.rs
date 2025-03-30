// src/static_files.rs

// dependencies
use std::borrow::Cow;
use std::path::{Path, PathBuf};

// struct type which represents configuration for a static file server
#[derive(Debug, Clone)]
pub struct StaticServerConfig {
    pub mount_path: Cow<'static, str>,
    pub root_dir: PathBuf,
    pub serve_index: bool,
}

// struct type which represents the static file server
pub struct StaticServer {
    mount_path: String,
    root_dir: PathBuf,
    serve_index: bool,
}

// methods for the StaticServer type
impl StaticServer {
    pub fn from_config(config: StaticServerConfig) -> Self {
        let mount_path = normalize_mount_path(config.mount_path.as_ref());
        StaticServer {
            mount_path,
            root_dir: config.root_dir,
            serve_index: config.serve_index,
        }
    }

    pub fn resolve(&self, request_path: &str) -> Option<PathBuf> {
        if !request_path.starts_with(&self.mount_path) {
            return None;
        }

        // Strip the mount path from the request path
        let relative_path = request_path
            .strip_prefix(&self.mount_path)
            .unwrap_or("")
            .trim_start_matches('/');

        // Join the relative path to the root directory
        let mut full_path = self.root_dir.join(relative_path);

        // If it's a directory and `serve_index` is true, try to serve index.html
        if full_path.is_dir() && self.serve_index {
            full_path = full_path.join("index.html");
        }

        // Only return it if the file exists and is not a directory
        if full_path.exists() && full_path.is_file() {
            Some(full_path)
        } else {
            None
        }
    }

    pub fn guess_mime_type(&self, path: &Path) -> String {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    }
}

// helper function to normalize the mount path of the StaticServer
fn normalize_mount_path(path: &str) -> String {
    if path == "/" {
        return "/".to_string();
    }

    let trimmed = path.trim_end_matches('/');

    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    }
}
