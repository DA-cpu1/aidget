use aidget::widgets::store::Store;
use aidget::widgets::template::{bootstrap_welcome, install, list};

use tempfile::TempDir;

#[test]
fn builtin_welcome_is_listed() {
    let templates = list();
    assert!(templates.iter().any(|item| item.id == "welcome"));
}

#[test]
fn bootstrap_installs_once() {
    let dir = TempDir::new().unwrap();
    let store = Store::new(dir.path());
    store.ensure_layout().unwrap();
    bootstrap_welcome(&store).unwrap();
    assert!(store
        .widget_dir("welcome")
        .unwrap()
        .join(aidget::widgets::store::HTML_FILE)
        .is_file());
    bootstrap_welcome(&store).unwrap();
    assert_eq!(store.widget_ids().unwrap(), vec!["welcome".to_string()]);
}

#[test]
fn install_rejects_existing_id() {
    let dir = TempDir::new().unwrap();
    let store = Store::new(dir.path());
    store.ensure_layout().unwrap();
    install(&store, "welcome", "welcome").unwrap();
    assert!(install(&store, "welcome", "welcome").is_err());
}

#[test]
fn install_copies_agents_and_sets_source() {
    let dir = TempDir::new().unwrap();
    let store = Store::new(dir.path());
    store.ensure_layout().unwrap();
    let meta = install(&store, "welcome", "hello").unwrap();
    assert_eq!(meta.drag_mode, aidget::widgets::model::WidgetDragMode::All);
    assert_eq!(meta.source.unwrap().id, "welcome");
    assert!(store
        .widget_dir("hello")
        .unwrap()
        .join("agents.md")
        .is_file());
}
