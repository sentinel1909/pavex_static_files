// src/static_server.rs

// dependencies
use crate::config::StaticServerConfig;
use crate::errors::ServeError;
use std::fs::canonicalize;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

// struct type which represents the static file server
pub struct StaticServer {
    mount_path: String,
    root_dir: PathBuf,
    serve_index: bool,
}

// struct type which represents the static file to be served
#[derive(Debug)]
pub struct StaticFile {
    pub body: Vec<u8>,
    pub mime_type: Cow<'static, str>,
    pub path: PathBuf,
}

// methods for the StaticServer type
impl StaticServer {
    // create a static file server from it's configuration values
    pub fn from_config(config: StaticServerConfig) -> Self {
        let mount_path = normalize_mount_path(config.mount_path.as_ref());
        StaticServer {
            mount_path,
            root_dir: config.root_dir,
            serve_index: config.serve_index,
        }
    }

    // resolve the file to be served, using the incoming request path
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

        let canonical_full = canonicalize(&full_path).ok()?;
        let canonical_root = canonicalize(&self.root_dir).ok()?;

        if !canonical_full.starts_with(&canonical_root) {
            return None;
        }

        // Only return it if the file exists and is not a directory
        if canonical_full.exists() && canonical_full.is_file() {
            Some(canonical_full)
        } else {
            None
        }
    }

    // read the file from disk
    pub fn read_file(&self, request_path: &str) -> Result<StaticFile, ServeError> {
        let file_path = self.resolve(request_path).ok_or(ServeError::NotFound)?;

        let body = std::fs::read(&file_path).map_err(ServeError::Io)?;

        let mime_type = guess_mime_type(file_path.as_path());

        Ok(StaticFile {
            body,
            mime_type,
            path: file_path,
        })
    }

    // utility to return the mount path
    pub fn mount_path(&self) -> &str {
        &self.mount_path
    }

    // utility to return the root dir
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    // utility to return whether serving index.html is true or false
    pub fn serve_index(&self) -> bool {
        self.serve_index
    }
}

// helper function to guess the mime type
pub fn guess_mime_type(path: &Path) -> Cow<'static, str> {
    Cow::Owned(
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string(),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn serves_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("hello.txt");

        let mut file = File::create(&file_path).unwrap();
        write!(file, "Hello, world!").unwrap();

        let config = StaticServerConfig {
            mount_path: "/static".into(),
            root_dir: dir.path().to_path_buf(),
            serve_index: false,
        };

        let server = StaticServer::from_config(config);
        let result = server.read_file("/static/hello.txt");

        assert!(result.is_ok(), "Expected file to be served successfully");

        let static_file = result.unwrap();
        assert_eq!(static_file.mime_type, "text/plain");
        assert_eq!(static_file.body, b"Hello, world!");
    }

    #[test]
    fn returns_not_found_for_missing_file() {
        let dir = tempdir().unwrap();

        let config = StaticServerConfig {
            mount_path: "/static".into(),
            root_dir: dir.path().to_path_buf(),
            serve_index: false,
        };

        let server = StaticServer::from_config(config);
        let result = server.read_file("/static/missing.txt");

        assert!(matches!(result, Err(ServeError::NotFound)));
    }

    #[test]
    fn serves_index_file_when_directory_requested() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("docs");
        fs::create_dir(&subdir).unwrap();

        let index_path = subdir.join("index.html");
        let mut index_file = File::create(&index_path).unwrap();
        write!(index_file, "<h1>Index</h1>").unwrap();

        let config = StaticServerConfig {
            mount_path: "/static".into(),
            root_dir: dir.path().to_path_buf(),
            serve_index: true,
        };

        let server = StaticServer::from_config(config);
        let result = server.read_file("/static/docs");

        assert!(result.is_ok(), "Expected index.html to be served");

        let static_file = result.unwrap();
        assert_eq!(static_file.mime_type, "text/html");
        assert_eq!(static_file.body, b"<h1>Index</h1>");
    }

    #[test]
    fn directory_without_index_returns_not_found() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("images");
        fs::create_dir(&subdir).unwrap();

        let config = StaticServerConfig {
            mount_path: "/static".into(),
            root_dir: dir.path().to_path_buf(),
            serve_index: true,
        };

        let server = StaticServer::from_config(config);
        let result = server.read_file("/static/images");

        assert!(matches!(result, Err(ServeError::NotFound)));
    }
}
