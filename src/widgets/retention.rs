use std::ffi::{OsStr, OsString};
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::common::error::Result;
use crate::settings;

/// Remove the oldest managed PNGs until the screenshot directory fits `limit`.
pub fn prune_screenshots(dir: &Path, limit: NonZeroUsize) -> Result<()> {
    for (_, _, path) in excess_entries(entries(dir, RetentionKind::Screenshot)?, limit) {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Remove the oldest widget directories until `trash/` fits `limit`.
pub fn prune_trash(dir: &Path, limit: NonZeroUsize) -> Result<()> {
    for (_, _, path) in excess_entries(entries(dir, RetentionKind::Trash)?, limit) {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Write a screenshot and then apply its directory's configured retention.
pub fn write_screenshot(
    root: &Path,
    screenshots_dir: &Path,
    path: &Path,
    bytes: &[u8],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    let settings = settings::load(root)?;
    prune_screenshots(screenshots_dir, settings.screenshot_retention)
}

enum RetentionKind {
    Screenshot,
    Trash,
}

type RetentionEntry = (SystemTime, OsString, PathBuf);

fn entries(dir: &Path, kind: RetentionKind) -> Result<Vec<RetentionEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        // User-created files and symlinks in managed directories are not ours to remove.
        let managed = match kind {
            RetentionKind::Screenshot => {
                file_type.is_file() && entry.path().extension() == Some(OsStr::new("png"))
            }
            RetentionKind::Trash => file_type.is_dir(),
        };
        if managed {
            entries.push((
                entry.metadata()?.modified()?,
                entry.file_name(),
                entry.path(),
            ));
        }
    }
    // Retain the oldest first so callers can remove the excess prefix.
    entries.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    Ok(entries)
}

fn excess_entries(entries: Vec<RetentionEntry>, limit: NonZeroUsize) -> Vec<RetentionEntry> {
    let remove = entries.len().saturating_sub(limit.get());
    entries.into_iter().take(remove).collect()
}
