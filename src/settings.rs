use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml_edit::{value, DocumentMut};

use crate::common::error::{Error, Result};
use crate::common::utils::atomic_write;

pub const SETTINGS_FILE: &str = "settings.toml";
pub const DEFAULT_SCREENSHOT_RETENTION: usize = 30;
pub const DEFAULT_TRASH_RETENTION: usize = 20;

/// Main-window navigation destinations persisted between launches.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MainPage {
    /// Installed desktop widgets.
    LocalWidgets,
    /// Community widget directory.
    CommunityWidgets,
    /// Agent MCP and skill integrations.
    #[default]
    Integrations,
    /// General preferences.
    General,
    /// Application information.
    About,
}

impl MainPage {
    fn as_str(self) -> &'static str {
        match self {
            MainPage::LocalWidgets => "local-widgets",
            MainPage::CommunityWidgets => "community-widgets",
            MainPage::Integrations => "integrations",
            MainPage::General => "general",
            MainPage::About => "about",
        }
    }
}

/// Selected UI color theme.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    /// Follow the operating system color scheme.
    #[default]
    System,
    /// Always use the light color scheme.
    Light,
    /// Always use the dark color scheme.
    Dark,
}

impl Theme {
    fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

/// Selected UI language.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Language {
    /// Follow the operating system language.
    #[default]
    #[serde(rename = "system")]
    System,
    /// English UI.
    #[serde(rename = "en")]
    En,
    /// Simplified Chinese UI.
    #[serde(rename = "zh-CN")]
    ZhCn,
}

impl Language {
    fn as_str(self) -> &'static str {
        match self {
            Language::System => "system",
            Language::En => "en",
            Language::ZhCn => "zh-CN",
        }
    }
}

/// The action performed by a primary click on the tray icon.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TrayClickAction {
    /// Open the tray menu, which remains available on right click.
    Menu,
    /// Create, reveal, and focus the main window.
    #[default]
    Window,
}

impl TrayClickAction {
    /// Linux tray backends always own the primary-click menu, so the effective
    /// action there is the menu even if a preference was written elsewhere.
    pub fn effective(self) -> Self {
        if cfg!(target_os = "linux") {
            Self::Menu
        } else {
            self
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            TrayClickAction::Menu => "menu",
            TrayClickAction::Window => "window",
        }
    }
}

/// Preferences edited by the main window and persisted in `settings.toml`.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiPreferences {
    /// Main-window color theme.
    pub theme: Theme,
    /// Main-window language.
    pub language: Language,
    /// Primary-click behavior for the tray icon.
    pub tray_click_action: TrayClickAction,
    /// Most recently opened main-window destination.
    pub last_page: MainPage,
}

/// User-editable settings stored at the root of the aidget data directory.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Maximum number of managed PNG files under `state/screenshots/`.
    pub screenshot_retention: NonZeroUsize,
    /// Maximum number of widget directories under `trash/`.
    pub trash_retention: NonZeroUsize,
    /// Main-window color theme.
    #[serde(default)]
    pub theme: Theme,
    /// Main-window language.
    #[serde(default)]
    pub language: Language,
    /// Primary-click behavior for the tray icon.
    #[serde(default)]
    pub tray_click_action: TrayClickAction,
    /// Most recently opened main-window destination.
    #[serde(default)]
    pub last_page: MainPage,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            screenshot_retention: NonZeroUsize::new(DEFAULT_SCREENSHOT_RETENTION)
                .expect("default screenshot retention is non-zero"),
            trash_retention: NonZeroUsize::new(DEFAULT_TRASH_RETENTION)
                .expect("default trash retention is non-zero"),
            theme: Theme::default(),
            language: Language::default(),
            tray_click_action: TrayClickAction::default(),
            last_page: MainPage::default(),
        }
    }
}

impl Settings {
    /// Return the preference subset exposed to the main window.
    pub fn preferences(&self) -> UiPreferences {
        UiPreferences {
            theme: self.theme,
            language: self.language,
            tray_click_action: self.tray_click_action,
            last_page: self.last_page,
        }
    }
}

/// Create `settings.toml` when absent and add missing UI preference defaults
/// without replacing retention values or comments already in the document.
pub fn ensure(root: &Path) -> Result<()> {
    let path = settings_path(root);
    if !path.exists() {
        atomic_write(&path, default_document().as_bytes())?;
        return Ok(());
    }

    load(root)?;
    let text = fs::read_to_string(&path)?;
    let mut document = text
        .parse::<DocumentMut>()
        .map_err(|error| Error::Other(format!("toml: {error}")))?;
    insert_missing(&mut document, "theme", Theme::System.as_str());
    insert_missing(&mut document, "language", Language::System.as_str());
    insert_missing(
        &mut document,
        "tray_click_action",
        TrayClickAction::Window.as_str(),
    );
    insert_missing(&mut document, "last_page", MainPage::Integrations.as_str());
    let next = document.to_string();
    if next != text {
        atomic_write(&path, next.as_bytes())?;
    }
    Ok(())
}

/// Read and strictly validate `settings.toml`.
pub fn load(root: &Path) -> Result<Settings> {
    let bytes = fs::read(settings_path(root))?;
    let text =
        std::str::from_utf8(&bytes).map_err(|error| Error::Other(format!("toml: {error}")))?;
    toml::from_str(text).map_err(|error| Error::Other(format!("toml: {error}")))
}

/// Update only the UI preferences while preserving retention settings
/// and the rest of the TOML document.
pub fn save_preferences(root: &Path, preferences: UiPreferences) -> Result<()> {
    load(root)?;
    let path = settings_path(root);
    let text = fs::read_to_string(&path)?;
    let mut document = text
        .parse::<DocumentMut>()
        .map_err(|error| Error::Other(format!("toml: {error}")))?;
    document["theme"] = value(preferences.theme.as_str());
    document["language"] = value(preferences.language.as_str());
    document["tray_click_action"] = value(preferences.tray_click_action.as_str());
    document["last_page"] = value(preferences.last_page.as_str());
    atomic_write(&path, document.to_string().as_bytes())
}

fn settings_path(root: &Path) -> PathBuf {
    root.join(SETTINGS_FILE)
}

fn insert_missing(document: &mut DocumentMut, key: &str, value: &str) {
    if !document.contains_key(key) {
        document[key] = toml_edit::value(value);
    }
}

fn default_document() -> String {
    format!(
        "# Maximum number of screenshots kept in state/screenshots.\n\
screenshot_retention = {DEFAULT_SCREENSHOT_RETENTION}\n\
\n\
# Maximum number of deleted widgets kept in trash.\n\
trash_retention = {DEFAULT_TRASH_RETENTION}\n\
\n\
# Main-window color theme: system, light, or dark.\n\
theme = \"system\"\n\
\n\
# Main-window language: system, en, or zh-CN.\n\
language = \"system\"\n\
\n\
# Primary tray click: window or menu.\n\
tray_click_action = \"window\"\n\
\n\
# Main-window destination restored on launch.\n\
last_page = \"integrations\"\n"
    )
}
