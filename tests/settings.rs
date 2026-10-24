use std::fs;

use aidget::settings::{
    ensure, load, save_preferences, Language, Theme, TrayClickAction, UiPreferences, SETTINGS_FILE,
};
use tempfile::TempDir;

fn root() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    ensure(&root).unwrap();
    (dir, root)
}

#[test]
fn settings_are_created_with_retention_and_preference_defaults() {
    let (_dir, root) = root();
    assert_eq!(SETTINGS_FILE, "settings.toml");

    let settings = load(&root).unwrap();
    assert_eq!(settings.screenshot_retention.get(), 30);
    assert_eq!(settings.trash_retention.get(), 20);
    assert_eq!(settings.theme, Theme::System);
    assert_eq!(settings.language, Language::System);
    assert_eq!(settings.tray_click_action, TrayClickAction::Window);

    let document = fs::read_to_string(root.join(SETTINGS_FILE)).unwrap();
    assert!(document.contains("screenshot_retention = 30"));
    assert!(document.contains("trash_retention = 20"));
    assert!(document.contains("theme = \"system\""));
    assert!(document.contains("language = \"system\""));
    assert!(document.contains("tray_click_action = \"window\""));
}

#[test]
fn ensure_adds_missing_preferences_without_losing_retention_comments() {
    let (dir, root) = root();
    let path = root.join(SETTINGS_FILE);
    let custom = "\
# Keep enough screenshots for the current debugging session.\n\
screenshot_retention = 12\n\
\n\
# Pruned widgets remain recoverable for a shorter period.\n\
trash_retention = 7\n";
    fs::write(&path, custom).unwrap();

    ensure(&root).unwrap();

    let document = fs::read_to_string(&path).unwrap();
    assert!(document.contains("# Keep enough screenshots"));
    assert!(document.contains("screenshot_retention = 12"));
    assert!(document.contains("# Pruned widgets remain"));
    assert!(document.contains("trash_retention = 7"));
    assert!(document.contains("theme = \"system\""));
    assert!(document.contains("language = \"system\""));
    assert!(document.contains("tray_click_action = \"window\""));
    drop(dir);
}

#[test]
fn saving_preferences_preserves_retention_and_unrelated_comments() {
    let (dir, root) = root();
    let path = root.join(SETTINGS_FILE);
    fs::write(
        &path,
        "\
# Captured screenshots are retained briefly.\n\
screenshot_retention = 9\n\
trash_retention = 4\n\
\n\
# The UI can follow the operating system.\n\
theme = \"system\"\n\
language = \"system\"\n\
tray_click_action = \"window\"\n",
    )
    .unwrap();

    let preferences = UiPreferences {
        theme: Theme::Dark,
        language: Language::ZhCn,
        tray_click_action: TrayClickAction::Menu,
    };
    save_preferences(&root, preferences).unwrap();

    let document = fs::read_to_string(&path).unwrap();
    assert!(document.contains("# Captured screenshots are retained briefly."));
    assert!(document.contains("# The UI can follow the operating system."));
    assert!(document.contains("screenshot_retention = 9"));
    assert!(document.contains("trash_retention = 4"));
    assert!(document.contains("theme = \"dark\""));
    assert!(document.contains("language = \"zh-CN\""));
    assert!(document.contains("tray_click_action = \"menu\""));
    assert_eq!(load(&root).unwrap().preferences(), preferences);
    drop(dir);
}

#[test]
fn settings_reject_invalid_retention_or_preference_shapes() {
    let (dir, root) = root();
    let path = root.join(SETTINGS_FILE);
    for invalid in [
        "",
        "screenshot_retention = 30\ntrash_retention = 0\n",
        "screenshot_retention = 30\ntrash_retention = 20\nunknown = true\n",
        "screenshot_retention = 30\ntrash_retention = 20\ntheme = \"sepia\"\n",
        "screenshot_retention = 30\ntrash_retention = 20\nlanguage = \"fr\"\n",
        "screenshot_retention = 30\ntrash_retention = 20\ntray_click_action = \"popup\"\n",
        "not = [valid",
    ] {
        fs::write(&path, invalid).unwrap();
        assert!(load(&root).is_err(), "{invalid}");
    }
    drop(dir);
}
