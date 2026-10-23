use std::collections::{HashMap, HashSet};
use std::fs;

use aidget::runtime::main_window_state;
use tempfile::TempDir;

fn ids(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

/// Read the state file as a plain map so tests assert the persisted shape.
fn read_state(root: &std::path::Path) -> HashMap<String, serde_json::Value> {
    let bytes = fs::read(main_window_state::state_path(root)).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn archived_widgets_reports_only_archived_ids() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let all = ids(&["alpha", "zeta"]);
    main_window_state::set_archived(root, &all, "zeta", true).unwrap();

    assert_eq!(
        main_window_state::archived_widgets(root, &all).unwrap(),
        HashSet::from(["zeta".to_string()])
    );

    main_window_state::set_archived(root, &all, "zeta", false).unwrap();
    assert!(main_window_state::archived_widgets(root, &all)
        .unwrap()
        .is_empty());

    // Both widgets keep a record; archiving is expressed by the field, not by
    // presence in the file.
    let file = read_state(root);
    assert_eq!(file.len(), 2);
    assert_eq!(file["zeta"]["archived"], serde_json::Value::Bool(false));
    assert_eq!(file["alpha"]["archived"], serde_json::Value::Bool(false));

    // Keys are written in sorted order so the file stays stable.
    let text = fs::read_to_string(main_window_state::state_path(root)).unwrap();
    assert!(text.find("\"alpha\"").unwrap() < text.find("\"zeta\"").unwrap());
}

#[test]
fn state_file_has_one_entry_per_widget_and_drops_removed_ids() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    main_window_state::set_archived(root, &ids(&["alpha", "zeta"]), "zeta", true).unwrap();

    // "alpha" is gone from the live list, so the next write drops it.
    main_window_state::set_archived(root, &ids(&["zeta"]), "zeta", false).unwrap();

    assert_eq!(
        read_state(root).keys().cloned().collect::<Vec<_>>(),
        vec!["zeta".to_string()]
    );
    assert_eq!(
        read_state(root)["zeta"]["archived"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        main_window_state::archived_widgets(root, &ids(&["zeta"])).unwrap(),
        HashSet::new()
    );
}

#[test]
fn stale_entries_are_ignored_until_the_next_write() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    main_window_state::set_archived(root, &ids(&["alpha", "ghost"]), "ghost", true).unwrap();
    let live = ids(&["alpha"]);

    // A record for a widget that no longer exists neither errors nor shows up.
    assert_eq!(
        main_window_state::archived_widgets(root, &live).unwrap(),
        HashSet::new()
    );

    main_window_state::set_archived(root, &live, "alpha", false).unwrap();
    assert_eq!(
        read_state(root).keys().cloned().collect::<Vec<_>>(),
        vec!["alpha".to_string()]
    );
}

#[test]
fn missing_state_file_reads_as_unarchived() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        main_window_state::archived_widgets(dir.path(), &ids(&["alpha"])).unwrap(),
        HashSet::new()
    );
}

#[test]
fn preview_is_cached_separately_and_removed_with_widget() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    main_window_state::save_preview(root, "weather", b"png bytes").unwrap();
    assert_eq!(
        main_window_state::load_preview(root, "weather").unwrap(),
        Some(b"png bytes".to_vec())
    );

    main_window_state::set_archived(root, &ids(&["weather"]), "weather", true).unwrap();
    main_window_state::forget_widget(root, &ids(&[]), "weather").unwrap();

    assert_eq!(
        main_window_state::load_preview(root, "weather").unwrap(),
        None
    );
    assert!(read_state(root).is_empty());
}

#[test]
fn rejects_invalid_widget_ids_before_building_paths() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    assert!(main_window_state::load_preview(root, "../outside").is_err());
    assert!(main_window_state::preview_path(root, "../outside").is_err());
    assert!(main_window_state::set_archived(root, &ids(&["alpha"]), "../outside", true).is_err());
    assert!(main_window_state::set_archived(root, &ids(&["../outside"]), "alpha", true).is_err());
    assert!(main_window_state::forget_widget(root, &ids(&[]), "../outside").is_err());
}

#[test]
fn malformed_state_files_fail_loudly() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let live = ids(&["alpha"]);
    let path = main_window_state::state_path(root);
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // An id that escapes the data directory is a hard error, not a skip.
    fs::write(&path, br#"{"../outside": {"archived": true}}"#).unwrap();
    assert!(main_window_state::archived_widgets(root, &live).is_err());

    // Unknown fields are rejected as well.
    fs::write(&path, br#"{"alpha": {"archived": true, "pinned": false}}"#).unwrap();
    assert!(main_window_state::archived_widgets(root, &live).is_err());
}
