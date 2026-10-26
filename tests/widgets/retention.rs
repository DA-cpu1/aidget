use crate::common::utils::set_modified;
use aidget::settings::{ensure, load};
use aidget::widgets::retention;
use std::fs;

use tempfile::TempDir;

fn root() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    fs::create_dir_all(root.join("state/screenshots")).unwrap();
    fs::create_dir_all(root.join("trash")).unwrap();
    ensure(&root).unwrap();
    (dir, root)
}

#[test]
fn screenshot_retention_keeps_newest_managed_pngs() {
    let (_dir, root) = root();
    let screenshots = root.join("state/screenshots");
    for index in 0..31 {
        let path = screenshots.join(format!("widget-20261022T0000{index:02}.png"));
        fs::write(&path, b"png").unwrap();
        set_modified(&path, 1_000 + index);
    }
    let note = screenshots.join("keep.txt");
    fs::write(&note, b"not a screenshot").unwrap();
    let outside = root.join("outside.png");
    fs::write(&outside, b"custom output").unwrap();
    let settings = load(&root).unwrap();

    retention::prune_screenshots(&screenshots, settings.screenshot_retention).unwrap();
    retention::prune_screenshots(&screenshots, settings.screenshot_retention).unwrap();

    let screenshots: Vec<_> = fs::read_dir(&screenshots)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
        .collect();
    assert_eq!(screenshots.len(), 30);
    assert!(!screenshots
        .iter()
        .any(|entry| entry.file_name() == "widget-20261022T000000.png"));
    assert!(screenshots
        .iter()
        .any(|entry| entry.file_name() == "widget-20261022T000030.png"));
    assert!(note.exists());
    assert!(outside.exists());
}

#[test]
fn write_screenshot_prunes_managed_directory() {
    let (_dir, root) = root();
    let screenshots = root.join("state/screenshots");
    for index in 0..31 {
        let path = screenshots.join(format!("old-20261022T0000{index:02}.png"));
        fs::write(&path, b"png").unwrap();
        set_modified(&path, 1_000 + index);
    }
    let outside = root.join("custom.png");

    retention::write_screenshot(&root, &screenshots, &outside, b"new custom png").unwrap();

    let managed: Vec<_> = fs::read_dir(&screenshots)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
        .collect();
    assert_eq!(managed.len(), 30);
    assert!(!screenshots.join("old-20261022T000000.png").exists());
    assert!(outside.exists());
}

#[test]
fn trash_retention_keeps_newest_directories() {
    let (_dir, root) = root();
    let trash = root.join("trash");
    for index in 0..21 {
        let path = trash.join(format!("widget-20261022T0000{index:02}"));
        fs::create_dir(&path).unwrap();
        set_modified(&path, 1_000 + index);
    }
    let note = trash.join("keep.txt");
    fs::write(&note, b"not a trashed widget").unwrap();
    let settings = load(&root).unwrap();

    retention::prune_trash(&trash, settings.trash_retention).unwrap();
    retention::prune_trash(&trash, settings.trash_retention).unwrap();

    let directories = fs::read_dir(&trash)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .count();
    assert_eq!(directories, 20);
    assert!(!trash.join("widget-20261022T000000").exists());
    assert!(trash.join("widget-20261022T000020").exists());
    assert!(note.exists());
}
