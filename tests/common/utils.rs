use aidget::common::utils::atomic_write;
use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use tempfile::TempDir;

/// Set a file or directory's modified time through a handle with the access
/// rights Windows requires for timestamp updates.
pub(crate) fn set_modified(path: &Path, timestamp: u64) {
    let modified = UNIX_EPOCH + Duration::from_secs(timestamp);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        const FILE_WRITE_ATTRIBUTES: u32 = 0x0000_0100;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

        fs::OpenOptions::new()
            .access_mode(FILE_WRITE_ATTRIBUTES)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
    }
    #[cfg(not(windows))]
    fs::File::open(path)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}

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
