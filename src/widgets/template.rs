use include_dir::{include_dir, Dir};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::common::error::{Error, Result};
use crate::common::utils::atomic_write;
use crate::widgets::model::{validate_id, WidgetDragMode, WidgetMeta, WidgetSource, MAX_WIDGETS};
use crate::widgets::store::{Store, BOOTSTRAP_MARKER, CONFIG_FILE};

static BUILTIN: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/widgets");

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub version: String,
}

pub fn list() -> Vec<TemplateSummary> {
    let mut templates: BTreeMap<String, TemplateSummary> = BTreeMap::new();
    for dir in BUILTIN.dirs() {
        let Some(id) = dir.path().file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if validate_id(id).is_err() {
            continue;
        }
        let Some(file) = dir.get_file(dir.path().join("template.json")) else {
            continue;
        };
        let Ok(parsed) = serde_json::from_slice::<TemplateManifest>(file.contents()) else {
            continue;
        };
        templates.insert(
            id.to_string(),
            TemplateSummary {
                id: id.to_string(),
                title: parsed.title,
                description: parsed.description,
                version: parsed.version,
            },
        );
    }
    templates.into_values().collect()
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateManifest {
    title: String,
    description: String,
    version: String,
    #[serde(rename = "allowScripts")]
    allow_scripts: bool,
    #[serde(rename = "dragMode", default)]
    drag_mode: WidgetDragMode,
}

pub fn install(store: &Store, template_id: &str, target_id: &str) -> Result<WidgetMeta> {
    validate_id(template_id)?;
    validate_id(target_id)?;

    let existing = store.widget_ids()?;
    if existing.iter().any(|id| id == target_id) {
        return Err(Error::InvalidParams(format!(
            "widget `{target_id}` already exists"
        )));
    }
    if existing.len() >= MAX_WIDGETS {
        return Err(Error::InvalidParams(format!(
            "widget limit reached ({MAX_WIDGETS})"
        )));
    }

    let dir = BUILTIN
        .get_dir(template_id)
        .ok_or_else(|| Error::NotFound(template_id.to_string()))?;
    let manifest = dir
        .get_file(dir.path().join("template.json"))
        .ok_or_else(|| Error::NotFound(template_id.to_string()))?;
    let manifest: TemplateManifest = serde_json::from_slice(manifest.contents())?;

    let target = store.widget_dir(target_id)?;
    fs::create_dir_all(target.join("assets"))?;

    let mut meta = WidgetMeta::new(target_id, &manifest.title, manifest.allow_scripts);
    meta.drag_mode = manifest.drag_mode;
    meta.version = manifest.version.clone();
    meta.source = Some(WidgetSource {
        kind: "builtin".to_string(),
        id: template_id.to_string(),
        version: manifest.version,
    });

    copy_tree(dir, &target)?;

    atomic_write(
        &target.join(CONFIG_FILE),
        &serde_json::to_vec_pretty(&meta)?,
    )?;
    store.ensure_frame(target_id, existing.len())?;
    Ok(meta)
}

fn copy_tree(dir: &Dir<'_>, target: &Path) -> Result<()> {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(child) => {
                let relative = child
                    .path()
                    .strip_prefix(dir.path())
                    .map_err(|error| Error::Other(error.to_string()))?;
                let destination = target.join(relative);
                fs::create_dir_all(&destination)?;
                copy_tree(child, target)?;
            }
            include_dir::DirEntry::File(file) => {
                let relative = file
                    .path()
                    .strip_prefix(dir.path())
                    .map_err(|error| Error::Other(error.to_string()))?;
                if relative == Path::new("template.json") || relative == Path::new(CONFIG_FILE) {
                    continue;
                }
                let destination = target.join(relative);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                atomic_write(&destination, file.contents())?;
            }
        }
    }
    Ok(())
}

pub fn agents(template_id: &str) -> Option<String> {
    BUILTIN
        .get_file(Path::new(template_id).join("agents.md"))
        .and_then(|file| String::from_utf8(file.contents().to_vec()).ok())
}

pub fn bootstrap_welcome(store: &Store) -> Result<()> {
    let marker = store.root().join(BOOTSTRAP_MARKER);
    if marker.exists() {
        return Ok(());
    }
    if store.widget_ids()?.is_empty() {
        install(store, "welcome", "welcome")?;
    }
    atomic_write(&marker, b"")?;
    Ok(())
}
