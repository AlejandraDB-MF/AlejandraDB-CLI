use anyhow::{Result, bail};
use regex::Regex;
use std::path::Path;

pub fn serial(s: &str) -> Result<()> {
    let re = Regex::new(r"^[A-Za-z0-9._:\-]+$").unwrap();
    if !re.is_match(s) {
        bail!("{}", rust_i18n::t!("validate.error.serial", serial = s));
    }
    Ok(())
}

pub fn package(s: &str) -> Result<()> {
    let re = Regex::new(r"^[A-Za-z0-9._-]+$").unwrap();
    if !re.is_match(s) {
        bail!("{}", rust_i18n::t!("validate.error.package", package = s));
    }
    Ok(())
}

pub fn apk_path(path: &str) -> Result<()> {
    let path = path.trim();
    if path.is_empty() {
        bail!("{}", rust_i18n::t!("apps.install.error.empty"));
    }

    let file = Path::new(path);
    if !file.is_file() {
        bail!(
            "{}",
            rust_i18n::t!("apps.install.error.not_found", path = path)
        );
    }

    let is_apk = file
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("apk"));
    if !is_apk {
        bail!(
            "{}",
            rust_i18n::t!("apps.install.error.extension", path = path)
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "alejandradb-validate-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn apk_path_rejects_empty() {
        assert!(apk_path("").is_err());
        assert!(apk_path("   ").is_err());
    }

    #[test]
    fn apk_path_rejects_missing_file() {
        let path =
            std::env::temp_dir().join(format!("alejandradb-missing-{}.apk", std::process::id()));
        let _ = fs::remove_file(&path);
        assert!(apk_path(path.to_str().unwrap()).is_err());
    }

    #[test]
    fn apk_path_rejects_wrong_extension() {
        let dir = temp_dir("wrong-extension");
        let path = dir.join("application.txt");
        fs::write(&path, b"not an apk").unwrap();
        assert!(apk_path(path.to_str().unwrap()).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apk_path_accepts_existing_apk() {
        let dir = temp_dir("valid-apk");
        let path = dir.join("valid app.APK");
        fs::write(&path, b"apk").unwrap();
        assert!(apk_path(path.to_str().unwrap()).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }
}
