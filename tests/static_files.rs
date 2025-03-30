use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

// Bring your module into scope
use pavex_static_files::static_files::{StaticFileError, StaticFileHandler, StaticFiles};

#[test]
fn integration_serves_file_from_tempdir() {
    // Create a temporary directory
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("hello.txt");

    // Write a test file
    let mut file = File::create(&file_path).unwrap();
    write!(file, "Integration says hi!").unwrap();
    file.flush().unwrap();

    // Create StaticFiles instance
    let static_files = StaticFiles::new("/static", dir.path());

    // Simulate a request
    let result = static_files.handle_request("/static/hello.txt");

    // Validate result
    assert!(result.is_ok(), "Expected Ok, got {:?}", result);

    let response = result.unwrap();
    let body = String::from_utf8(response.body).unwrap();

    assert_eq!(body, "Integration says hi!");
    assert_eq!(response.content_type, "text/plain");
}

#[test]
fn integration_returns_not_found_for_missing_file() {
    // Create a temporary directory with no files
    let dir = tempdir().unwrap();

    // Set up the static file handler
    let static_files = StaticFiles::new("/static", dir.path());

    // Simulate a request to a non-existent file
    let result = static_files.handle_request("/static/missing.txt");

    // Check that we get the NotFound error
    assert!(matches!(result, Err(StaticFileError::NotFound)));
}
