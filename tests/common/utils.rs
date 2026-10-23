use aidget::common::utils::atomic_write;
use std::fs;

use tempfile::TempDir;

#[test]
fn atomic_write_replaces_and_leaves_no_temp_files() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("board.json");

    atomic_write(&path, b"{\"first\":true}").unwrap();
    atomic_write(&path, b"{\"second\":true}").unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"{\"second\":true}");
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with(".tmp-"))
                .unwrap_or(false)
        })
        .collect();
    assert!(leftovers.is_empty());
}
