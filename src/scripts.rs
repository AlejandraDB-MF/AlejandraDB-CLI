use crate::adb;
use crate::i18n;
use anyhow::{Context, Result, bail};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use time::OffsetDateTime;
use time::macros::format_description;

pub const EL_NEGRO_SAMUEL: &str = "ELNegroSamuel";
pub const BUILTIN_SCRIPTS: [&str; 1] = [EL_NEGRO_SAMUEL];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptFolder {
    Dcim,
    Downloads,
    Documents,
    Movies,
    Pictures,
}

impl ScriptFolder {
    pub const ALL: [ScriptFolder; 5] = [
        ScriptFolder::Dcim,
        ScriptFolder::Downloads,
        ScriptFolder::Documents,
        ScriptFolder::Movies,
        ScriptFolder::Pictures,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ScriptFolder::Dcim => "DCIM",
            ScriptFolder::Downloads => "Downloads",
            ScriptFolder::Documents => "Documents",
            ScriptFolder::Movies => "Movies",
            ScriptFolder::Pictures => "Pictures",
        }
    }

    pub fn remote_candidates(self) -> &'static [&'static str] {
        match self {
            ScriptFolder::Downloads => &["Download", "Downloads"],
            ScriptFolder::Dcim => &["DCIM"],
            ScriptFolder::Documents => &["Documents"],
            ScriptFolder::Movies => &["Movies"],
            ScriptFolder::Pictures => &["Pictures"],
        }
    }
}

impl fmt::Display for ScriptFolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptSummary {
    pub copied: usize,
    pub skipped: usize,
    pub failed: usize,
    pub destination: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptEvent {
    Status(String),
    Finished(ScriptSummary),
    Failed(String),
}

pub fn spawn_el_negro_samuel(
    adb: String,
    serial: String,
    output_path: PathBuf,
    folders: Vec<ScriptFolder>,
) -> Receiver<ScriptEvent> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        match run_el_negro_samuel(&adb, &serial, &output_path, &folders, &sender) {
            Ok(summary) => {
                let _ = sender.send(ScriptEvent::Finished(summary));
            }
            Err(error) => {
                let _ = sender.send(ScriptEvent::Failed(error.to_string()));
            }
        }
    });
    receiver
}

pub fn summary_message(summary: &ScriptSummary) -> String {
    rust_i18n::t!(
        "scripts.status.finished",
        path = summary.destination.display().to_string(),
        copied = summary.copied,
        skipped = summary.skipped,
        failed = summary.failed
    )
    .to_string()
}

fn run_el_negro_samuel(
    adb: &str,
    serial: &str,
    output_path: &Path,
    folders: &[ScriptFolder],
    sender: &Sender<ScriptEvent>,
) -> Result<ScriptSummary> {
    let _ = sender.send(ScriptEvent::Status(i18n::tr("scripts.status.resolving")));
    let root = resolve_storage(adb, serial, folders)?;
    fs::create_dir_all(output_path).with_context(|| {
        rust_i18n::t!(
            "scripts.error.output_dir",
            path = output_path.display().to_string()
        )
        .to_string()
    })?;
    let timestamp = timestamp_now();
    let destination = create_timestamp_dir(output_path, &timestamp)?;
    let total = folders.len();
    let mut summary = ScriptSummary {
        copied: 0,
        skipped: 0,
        failed: 0,
        destination: destination.clone(),
    };

    for (index, folder) in folders.iter().copied().enumerate() {
        let _ = sender.send(ScriptEvent::Status(
            rust_i18n::t!(
                "scripts.status.copying",
                folder = folder.label(),
                current = index + 1,
                total = total
            )
            .to_string(),
        ));
        let mut copied = false;
        let mut last_error: Option<String> = None;
        for candidate in folder.remote_candidates() {
            let remote = join_remote(&root, candidate);
            if !adb::remote_dir_exists(adb, serial, &remote) {
                continue;
            }
            let local = destination.join(folder.label());
            match adb::pull(adb, serial, &remote, &local) {
                Ok(()) => {
                    copied = true;
                    break;
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                    break;
                }
            }
        }
        if copied {
            summary.copied += 1;
        } else if let Some(error) = last_error {
            summary.failed += 1;
            let _ = sender.send(ScriptEvent::Status(
                rust_i18n::t!(
                    "scripts.status.folder_failed",
                    folder = folder.label(),
                    error = error
                )
                .to_string(),
            ));
        } else {
            summary.skipped += 1;
            let _ = sender.send(ScriptEvent::Status(
                rust_i18n::t!("scripts.status.skipped", folder = folder.label()).to_string(),
            ));
        }
    }

    Ok(summary)
}

pub fn resolve_storage(adb: &str, serial: &str, folders: &[ScriptFolder]) -> Result<String> {
    let mut candidates = Vec::new();
    candidates.push("/storage/self/primary".to_string());
    candidates.push("/storage/emulated/0".to_string());
    if let Some(external) = adb::remote_env(adb, serial, "EXTERNAL_STORAGE") {
        candidates.push(external);
    }
    candidates.push("/sdcard".to_string());
    candidates.push("/storage/sdcard0".to_string());

    let mut unique = Vec::new();
    for candidate in candidates {
        let root = candidate.trim().trim_end_matches('/').to_string();
        if !root.is_empty() && !unique.contains(&root) {
            unique.push(root);
        }
    }

    let existing_roots: Vec<String> = unique
        .into_iter()
        .filter(|root| adb::remote_dir_exists(adb, serial, root))
        .collect();
    if existing_roots.is_empty() {
        bail!("{}", i18n::tr("scripts.status.storage_not_found"))
    }

    let mut best: Option<(usize, usize, String)> = None;
    for (index, root) in existing_roots.iter().enumerate() {
        let count = folders
            .iter()
            .filter(|folder| {
                folder.remote_candidates().iter().any(|candidate| {
                    adb::remote_dir_exists(adb, serial, &join_remote(root, candidate))
                })
            })
            .count();
        match &mut best {
            None => best = Some((count, index, root.clone())),
            Some((best_count, _, _)) if count > *best_count => {
                best = Some((count, index, root.clone()))
            }
            _ => {}
        }
    }

    let (_, _, root) = best.expect("existing roots are non-empty");
    Ok(root)
}

pub fn join_remote(root: &str, name: &str) -> String {
    format!(
        "{}/{}",
        root.trim_end_matches('/'),
        name.trim_start_matches('/')
    )
}

pub fn timestamp_now() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format_timestamp(now)
}

fn format_timestamp(value: OffsetDateTime) -> String {
    let format = format_description!("[year]-[month]-[day]");
    value
        .format(&format)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
}

fn unique_timestamp_path(base: &Path, timestamp: &str) -> PathBuf {
    let mut candidate = base.join(timestamp);
    let mut suffix = 1;
    while candidate.exists() {
        candidate = base.join(format!("{timestamp}-{suffix}"));
        suffix += 1;
    }
    candidate
}

pub fn create_timestamp_dir(base: &Path, timestamp: &str) -> Result<PathBuf> {
    fs::create_dir_all(base).with_context(|| {
        rust_i18n::t!(
            "scripts.error.output_dir",
            path = base.display().to_string()
        )
        .to_string()
    })?;
    loop {
        let candidate = unique_timestamp_path(base, timestamp);
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    rust_i18n::t!(
                        "scripts.error.destination",
                        path = candidate.display().to_string()
                    )
                    .to_string()
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "alejandradb-scripts-test-{}-{nanos}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn builtin_registry_contains_el_negro_samuel() {
        assert_eq!(EL_NEGRO_SAMUEL, "ELNegroSamuel");
        assert_eq!(BUILTIN_SCRIPTS, [EL_NEGRO_SAMUEL]);
    }

    #[test]
    fn folders_have_literal_labels() {
        assert_eq!(ScriptFolder::Dcim.label(), "DCIM");
        assert_eq!(ScriptFolder::Downloads.label(), "Downloads");
        assert_eq!(ScriptFolder::Documents.label(), "Documents");
        assert_eq!(ScriptFolder::Movies.label(), "Movies");
        assert_eq!(ScriptFolder::Pictures.label(), "Pictures");
    }

    #[test]
    fn downloads_tries_download_then_downloads() {
        assert_eq!(
            ScriptFolder::Downloads.remote_candidates(),
            &["Download", "Downloads"]
        );
        assert_eq!(ScriptFolder::Dcim.remote_candidates(), &["DCIM"]);
        assert_eq!(ScriptFolder::Pictures.remote_candidates(), &["Pictures"]);
    }

    #[test]
    fn timestamp_uses_local_format() {
        let value = OffsetDateTime::from_unix_timestamp(0).unwrap();
        assert_eq!(format_timestamp(value), "1970-01-01");
    }

    #[test]
    fn unique_timestamp_adds_numeric_suffix() {
        let dir = temp_dir("unique");
        fs::create_dir_all(dir.join("20260101-010203")).unwrap();
        assert_eq!(
            unique_timestamp_path(&dir, "20260101-010203"),
            dir.join("20260101-010203-1")
        );
        fs::create_dir_all(dir.join("20260101-010203-1")).unwrap();
        assert_eq!(
            unique_timestamp_path(&dir, "20260101-010203"),
            dir.join("20260101-010203-2")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_timestamp_dir_creates_unique_directory() {
        let dir = temp_dir("create-unique");
        fs::create_dir_all(dir.join("20260101-010203")).unwrap();
        let created = create_timestamp_dir(&dir, "20260101-010203").unwrap();
        assert_eq!(created, dir.join("20260101-010203-1"));
        assert!(created.is_dir());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn joins_remote_paths() {
        assert_eq!(join_remote("/sdcard/", "/DCIM"), "/sdcard/DCIM");
        assert_eq!(
            join_remote("/storage/emulated/0", "Download"),
            "/storage/emulated/0/Download"
        );
    }
}
