// src/static_files.rs

// dependencies
use mime_guess::from_path;
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

pub trait StaticFileHandler {
    /// Given a request path (e.g., `/assets/app.js`), return a response or an error.
    fn handle_request(&self, path: &str) -> Result<StaticFileResponse, StaticFileError>;
}

// struct type to represent a static asset, CCSS, JS, an image, or anything else
#[derive(Debug, Clone)]
pub struct StaticFilesConfig {
    mount_path: Cow<'static, str>,
    directory: PathBuf,
}

#[derive(Debug)]
pub struct StaticFileResponse {
    pub body: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug)]
pub enum StaticFileError {
    NotFound,
    Io(std::io::Error),
}

// helper function which normalizes the mount path
fn normalize_mount_path(path: Cow<'_, str>) -> Cow<'_, str> {
    if path == "/" {
        return Cow::Borrowed("/");
    }

    let trimmed = path.trim_end_matches('/');

    let normalized = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    };

    Cow::Owned(normalized)
}

// methods for the StaticFiles type
impl StaticFilesConfig {
    pub fn new<T, U>(mount_path: T, directory: U) -> Self
    where
        T: Into<Cow<'static, str>>,
        U: Into<PathBuf>,
    {
        let raw_path: Cow<'static, str> = mount_path.into();
        let normalized_path = normalize_mount_path(raw_path);

        StaticFilesConfig {
            mount_path: normalized_path,
            directory: directory.into(),
        }
    }

    pub fn build(mount_path: &'static str, directory: PathBuf) -> Self {
        let raw_path = Cow::Borrowed(mount_path);
        let normalized_path = normalize_mount_path(raw_path);

        StaticFilesConfig { mount_path: normalized_path, directory }
    }

    pub fn resolve_path(&self, request_path: &str) -> Option<PathBuf> {
        if !request_path.starts_with(self.mount_path.as_ref()) {
            return None;
        }

        let relative_path = request_path
            .strip_prefix(self.mount_path.as_ref())
            .unwrap_or("")
            .trim_start_matches('/');
        let full_path = self.directory.join(relative_path);

        Some(full_path)
    }
}

impl StaticFileHandler for StaticFilesConfig {
    fn handle_request(&self, path: &str) -> Result<StaticFileResponse, StaticFileError> {
        let full_path = self.resolve_path(path).ok_or(StaticFileError::NotFound)?;

        let path_to_serve = if full_path.is_dir() {
            full_path.join("index.html")
        } else {
            full_path
        };

        if !path_to_serve.exists() || !path_to_serve.is_file() {
            return Err(StaticFileError::NotFound);
        }

        let body = fs::read(&path_to_serve).map_err(StaticFileError::Io)?;
        let content_type = from_path(&path_to_serve)
            .first_or_octet_stream()
            .to_string();

        Ok(StaticFileResponse { body, content_type })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn serves_static_file_successfully() {
        // Create a temp directory
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("hello.txt");

        // Write a file inside it
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "Hello, world!").unwrap(); // no newline

        // Create StaticFiles with that directory
        let static_files = StaticFilesConfig::build("/static", PathBuf::from(dir.path()));

        // Try to handle a request to the file
        let result = static_files.handle_request("/static/hello.txt");

        assert!(result.is_ok());
        let response = result.unwrap();
        let body_str = String::from_utf8(response.body).unwrap();

        assert_eq!(body_str, "Hello, world!");
        assert_eq!(response.content_type, "text/plain");

        // tempdir cleans up automatically when it goes out of scope
    }

    #[test]
    fn returns_not_found_for_missing_file() {
        // Create a temp directory
        let dir = tempdir().unwrap();

        // Create StaticFiles mounted at "/static" using the temp dir
        let static_files = StaticFilesConfig::build("/static", PathBuf::from(dir.path()));

        // Request a file that doesn't exist
        let result = static_files.handle_request("/static/does_not_exist.txt");

        // Assert that we got a NotFound error
        assert!(matches!(result, Err(StaticFileError::NotFound)));

        // Temp directory is automatically cleaned up
    }
}
