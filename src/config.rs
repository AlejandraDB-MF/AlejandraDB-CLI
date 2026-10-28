use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "es")]
    Spanish,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::English => "en",
            Lang::Spanish => "es",
        }
    }

    #[cfg(test)]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Lang::English),
            "es" => Some(Lang::Spanish),
            _ => None,
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Lang::English => "English",
            Lang::Spanish => "Español",
        }
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.native_name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub language: Lang,
    #[serde(default = "default_output_path")]
    pub output_path: String,
    #[serde(default)]
    pub wifi_subnet: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Lang::default(),
            output_path: default_output_path(),
            wifi_subnet: None,
        }
    }
}

fn default_output_path() -> String {
    "~/alejandradb".to_string()
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Some(legacy) = Self::legacy_path() else {
            return Self::load_from(&path);
        };
        Self::load_with_legacy(&path, &legacy)
    }

    fn load_with_legacy(path: &Path, legacy: &Path) -> Self {
        if path.exists() {
            return Self::load_from(path);
        }
        if legacy.exists() {
            let config = Self::load_json_from(legacy);
            let _ = config.save_to(path);
            return config;
        }
        Self::default()
    }

    pub fn load_from(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str::<Self>(&text).ok())
            .unwrap_or_default()
    }

    fn load_json_from(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<Self>(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()
            .ok_or_else(|| anyhow::anyhow!("{}", rust_i18n::t!("config.error.directory")))?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).with_context(|| {
                rust_i18n::t!(
                    "config.error.create_dir",
                    path = parent.display().to_string()
                )
                .to_string()
            })?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, text).with_context(|| {
            rust_i18n::t!("config.error.write", path = path.display().to_string()).to_string()
        })?;
        Ok(())
    }

    pub fn path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    pub fn legacy_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.json"))
    }

    fn config_dir() -> Option<PathBuf> {
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME")
            && !dir.is_empty()
        {
            return Some(PathBuf::from(dir).join("alejandradb"));
        }
        std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(|home| PathBuf::from(home).join(".config").join("alejandradb"))
    }

    pub fn output_dir(&self) -> PathBuf {
        let path = if self.output_path == "~" {
            "~/alejandradb"
        } else {
            self.output_path.as_str()
        };
        expand_tilde(path)
    }

    /// Devuelve la subnet efectiva para el escaneo Wi-Fi.
    /// Usa la configurada si existe, sino detecta la local, sino fallback a 192.168.1.
    pub fn wifi_subnet(&self) -> String {
        if let Some(subnet) = &self.wifi_subnet
            && !subnet.trim().is_empty()
        {
            return subnet.trim().to_string();
        }
        detect_local_subnet().unwrap_or_else(|| "192.168.1".to_string())
    }

    /// Guarda la subnet Wi-Fi en la configuración.
    pub fn set_wifi_subnet(&mut self, subnet: String) {
        let subnet = subnet.trim().to_string();
        if subnet.is_empty() {
            self.wifi_subnet = None;
        } else {
            self.wifi_subnet = Some(subnet);
        }
    }
}

/// Detecta la subnet local del equipo (Linux).
/// Intenta `ip route get 1.1.1.1` y luego `hostname -I`.
pub fn detect_local_subnet() -> Option<String> {
    // Try `ip route get 1.1.1.1`
    if let Ok(output) = std::process::Command::new("ip")
        .args(["route", "get", "1.1.1.1"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len() {
            if words[i] == "src" && i + 1 < words.len() {
                let ip = words[i + 1];
                let parts: Vec<&str> = ip.split('.').collect();
                if parts.len() == 4 {
                    return Some(format!("{}.{}.{}", parts[0], parts[1], parts[2]));
                }
            }
        }
    }
    // Fallback: try hostname -I
    if let Ok(output) = std::process::Command::new("hostname").arg("-I").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(ip) = text.split_whitespace().next() {
            let parts: Vec<&str> = ip.split('.').collect();
            if parts.len() == 4 {
                return Some(format!("{}.{}.{}", parts[0], parts[1], parts[2]));
            }
        }
    }
    None
}

fn expand_tilde(value: &str) -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    expand_tilde_with_home(value, home.as_deref())
}

fn expand_tilde_with_home(value: &str, home: Option<&Path>) -> PathBuf {
    if value == "~" {
        if let Some(home) = home {
            return home.to_path_buf();
        }
    } else if let Some(rest) = value.strip_prefix("~/")
        && let Some(home) = home
    {
        return home.join(rest);
    }
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "alejandradb-config-test-{}-{nanos}-{name}",
            std::process::id()
        ))
    }

    #[test]
    fn default_language_and_output_path() {
        let config = Config::default();
        assert_eq!(config.language, Lang::English);
        assert_eq!(config.output_path, "~/alejandradb");
        assert_eq!(
            toml::from_str::<Config>("").unwrap(),
            Config {
                language: Lang::English,
                output_path: "~/alejandradb".to_string(),
                wifi_subnet: None,
            }
        );
    }

    #[test]
    fn language_code_mapping() {
        assert_eq!(Lang::English.code(), "en");
        assert_eq!(Lang::Spanish.code(), "es");
        assert_eq!(Lang::from_code("en"), Some(Lang::English));
        assert_eq!(Lang::from_code("es"), Some(Lang::Spanish));
        assert_eq!(Lang::from_code("fr"), None);
    }

    #[test]
    fn config_toml_roundtrip() {
        let path = temp_path("roundtrip").join("config.toml");
        let config = Config {
            language: Lang::Spanish,
            output_path: "/tmp/alejandradb-output".to_string(),
            wifi_subnet: None,
        };
        config.save_to(&path).unwrap();
        assert_eq!(Config::load_from(&path), config);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("language = \"es\""));
        assert!(text.contains("output_path = \"/tmp/alejandradb-output\""));
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn legacy_json_migration_keeps_backup() {
        let dir = temp_path("migration");
        fs::create_dir_all(&dir).unwrap();
        let primary = dir.join("config.toml");
        let legacy = dir.join("config.json");
        fs::write(&legacy, r#"{"language":"es"}"#).unwrap();

        let config = Config::load_with_legacy(&primary, &legacy);

        assert_eq!(config.language, Lang::Spanish);
        assert_eq!(config.output_path, "~/alejandradb");
        assert!(primary.exists());
        assert!(legacy.exists());
        assert_eq!(Config::load_from(&primary), config);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn existing_toml_is_not_overwritten_by_legacy() {
        let dir = temp_path("existing");
        fs::create_dir_all(&dir).unwrap();
        let primary = dir.join("config.toml");
        let legacy = dir.join("config.json");
        fs::write(&legacy, r#"{"language":"es"}"#).unwrap();
        fs::write(
            &primary,
            "language = \"en\"\noutput_path = \"/tmp/existing\"\n",
        )
        .unwrap();

        let config = Config::load_with_legacy(&primary, &legacy);

        assert_eq!(config.language, Lang::English);
        assert_eq!(config.output_path, "/tmp/existing");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_and_invalid_config_default_to_english() {
        let missing = temp_path("missing").join("config.toml");
        assert_eq!(Config::load_from(&missing).language, Lang::English);
        assert_eq!(Config::load_from(&missing).output_path, "~/alejandradb");

        let path = temp_path("invalid.toml");
        fs::write(&path, "language = [").unwrap();
        assert_eq!(Config::load_from(&path).language, Lang::English);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn expands_tilde_and_home_subpaths() {
        let home = Path::new("/home/tester");
        assert_eq!(
            expand_tilde_with_home("~", Some(home)),
            PathBuf::from("/home/tester")
        );
        assert_eq!(
            expand_tilde_with_home("~/Media", Some(home)),
            PathBuf::from("/home/tester/Media")
        );
        assert_eq!(
            expand_tilde_with_home("/var/tmp", Some(home)),
            PathBuf::from("/var/tmp")
        );
        assert_eq!(
            expand_tilde_with_home("~other", Some(home)),
            PathBuf::from("~other")
        );
    }
}
