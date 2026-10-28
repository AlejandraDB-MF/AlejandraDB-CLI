use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    User,
    System,
    Critical,
    DeviceAdmin,
}

const CRITICAL_PREFIXES: &[&str] = &[
    "com.android.systemui",
    "com.android.settings",
    "com.android.phone",
    "com.android.server.telecom",
    "com.android.providers.settings",
    "com.android.providers.telephony",
    "com.google.android.gms",
    "com.google.android.gsf",
];

const CRITICAL_EXACT: &[&str] = &[
    "com.android.systemui",
    "com.android.settings",
    "com.android.phone",
];

pub fn is_critical(pkg: &str) -> bool {
    if CRITICAL_EXACT.contains(&pkg) {
        return true;
    }
    CRITICAL_PREFIXES.iter().any(|p| pkg.starts_with(p))
}

pub fn system_packages(adb: &str, serial: &str) -> Result<HashSet<String>> {
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "pm", "list", "packages", "-s"])
        .output()
        .with_context(|| rust_i18n::t!("safety.error.system_packages").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(crate::package::parse_list(&text).into_iter().collect())
}

pub fn device_admins(adb: &str, serial: &str) -> Result<HashSet<String>> {
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "dpm", "list-active-admins"])
        .output()
        .with_context(|| rust_i18n::t!("safety.error.admins").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut set = HashSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.to_lowercase().contains("no active") {
            continue;
        }
        // format: com.pkg/.AdminReceiver  or com.pkg/...
        if let Some(pkg) = line.split('/').next() {
            let pkg = pkg.trim();
            if !pkg.is_empty() {
                set.insert(pkg.to_string());
            }
        }
    }
    Ok(set)
}

pub fn package_paths(adb: &str, serial: &str) -> Result<HashMap<String, String>> {
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "pm", "list", "packages", "-f"])
        .output()
        .with_context(|| rust_i18n::t!("safety.error.paths").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(parse_package_paths(&text))
}

pub fn parse_package_paths(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("package:") else {
            continue;
        };
        if let Some((path, pkg)) = rest.rsplit_once('=') {
            let path = path.trim();
            let pkg = pkg.trim();
            if !path.is_empty() && !pkg.is_empty() {
                map.insert(pkg.to_string(), path.to_string());
            }
        }
    }
    map
}

pub fn classify(
    pkg: &str,
    system_set: &HashSet<String>,
    admin_set: &HashSet<String>,
    path: Option<&str>,
) -> AppKind {
    if admin_set.contains(pkg) {
        return AppKind::DeviceAdmin;
    }
    let is_crit_path = path
        .map(|p| {
            p.starts_with("/system/priv-app")
                || p.starts_with("/system/framework")
                || p.starts_with("/vendor/priv-app")
        })
        .unwrap_or(false);
    if is_critical(pkg) || is_crit_path {
        return AppKind::Critical;
    }
    let is_sys_path = path
        .map(|p| {
            p.starts_with("/system/app")
                || p.starts_with("/vendor/app")
                || p.starts_with("/product/app")
                || p.starts_with("/odm/app")
        })
        .unwrap_or(false);
    if is_sys_path || system_set.contains(pkg) {
        return AppKind::System;
    }
    AppKind::User
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn critical_detection() {
        assert!(is_critical("com.android.systemui"));
        assert!(is_critical("com.google.android.gms"));
        assert!(!is_critical("com.example.app"));
    }

    #[test]
    fn classify_user() {
        let sys = HashSet::new();
        let admins = HashSet::new();
        assert!(matches!(
            classify("com.example.app", &sys, &admins, None),
            AppKind::User
        ));
    }

    #[test]
    fn classify_system() {
        let mut sys = HashSet::new();
        sys.insert("com.foo.system".to_string());
        let admins = HashSet::new();
        assert!(matches!(
            classify("com.foo.system", &sys, &admins, None),
            AppKind::System
        ));
    }

    #[test]
    fn parse_package_paths_basic() {
        let text = "package:/system/app/Foo.apk=com.foo\npackage:/data/app/bar.apk=com.bar\n";
        let m = parse_package_paths(text);
        assert_eq!(m.get("com.foo"), Some(&"/system/app/Foo.apk".to_string()));
        assert_eq!(m.get("com.bar"), Some(&"/data/app/bar.apk".to_string()));
    }
}
