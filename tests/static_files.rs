use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

use pavex_static_files::static_files::{
    StaticServer, StaticServerConfig,
};

#[test]
fn serves_js_and_css_mime_types() {
    let dir = tempdir().unwrap();

    let js_path = dir.path().join("main.js");
    let mut js_file = File::create(&js_path).unwrap();
    write!(js_file, "console.log('Hello');").unwrap();

    let css_path = dir.path().join("style.css");
    let mut css_file = File::create(&css_path).unwrap();
    write!(css_file, "body {{ margin: 0; }}").unwrap();

    let config = StaticServerConfig {
        mount_path: "/static".into(),
        root_dir: dir.path().to_path_buf(),
        serve_index: false,
    };

    let server = StaticServer::from_config(config);

    let js = server.read_file("/static/main.js").unwrap();
    assert!(js.mime_type == "application/javascript" || js.mime_type == "text/javascript");

    let css = server.read_file("/static/style.css").unwrap();
    assert_eq!(css.mime_type, "text/css");
}
