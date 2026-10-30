use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub serial: String,
    pub state: String,
    pub manufacturer: String,
    pub model: String,
}

pub fn devices(adb: &str) -> Result<Vec<Device>> {
    let out = Command::new(adb)
        .arg("devices")
        .output()
        .with_context(|| rust_i18n::t!("adb.error.devices").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    parse_devices(&text)
}

pub fn parse_devices(text: &str) -> Result<Vec<Device>> {
    let mut result = Vec::new();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let serial = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        let state = parts.next().unwrap_or("unknown");
        result.push(Device {
            serial: serial.to_string(),
            state: state.to_string(),
            manufacturer: String::new(),
            model: String::new(),
        });
    }
    Ok(result)
}

pub fn device_brand_model(adb: &str, serial: &str) -> (String, String) {
    // Try direct property queries first.
    let mut manufacturer = prop_value(adb, serial, "ro.product.manufacturer");
    let mut model = prop_value(adb, serial, "ro.product.model");

    // Fall back to full getprop output for environments that only support it.
    if (manufacturer.is_empty() || model.is_empty())
        && let Ok(text) = getprop(adb, serial)
    {
        let props = parse_getprop(&text);
        if manufacturer.is_empty() {
            manufacturer = props
                .get("ro.product.manufacturer")
                .cloned()
                .unwrap_or_default();
        }
        if model.is_empty() {
            model = props.get("ro.product.model").cloned().unwrap_or_default();
        }
    }

    (manufacturer, model)
}

fn prop_value(adb: &str, serial: &str, key: &str) -> String {
    if crate::validate::serial(serial).is_err() {
        return String::new();
    }
    let adb = adb.to_string();
    let serial = serial.to_string();
    let key = key.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = Command::new(&adb)
            .args(["-s", &serial, "shell", "getprop", &key])
            .output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(std::time::Duration::from_millis(2000)) {
        Ok(Ok(out)) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => String::new(),
    }
}

pub fn shell(adb: &str, serial: &str, args: &[String]) -> Result<()> {
    crate::validate::serial(serial)?;
    let status = Command::new(adb)
        .arg("-s")
        .arg(serial)
        .arg("shell")
        .args(args)
        .status()
        .with_context(|| rust_i18n::t!("adb.error.shell").to_string())?;
    if !status.success() {
        bail!(
            "{}",
            rust_i18n::t!(
                "adb.error.shell_status",
                code = format!("{:?}", status.code())
            )
        );
    }
    Ok(())
}

pub fn remote_env(adb: &str, serial: &str, key: &str) -> Option<String> {
    if crate::validate::serial(serial).is_err() {
        return None;
    }
    let command = format!("printf %s \"${key}\"");
    let output = Command::new(adb)
        .args(["-s", serial, "shell", "sh", "-c", command.as_str()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

pub fn remote_dir_exists(adb: &str, serial: &str, path: &str) -> bool {
    if path.is_empty() || crate::validate::serial(serial).is_err() {
        return false;
    }
    Command::new(adb)
        .args(["-s", serial, "shell", "test", "-d", path])
        .output()
        .is_ok_and(|output| output.status.success())
}

pub fn pull(adb: &str, serial: &str, remote: &str, local: &Path) -> Result<()> {
    crate::validate::serial(serial)?;
    let output = Command::new(adb)
        .arg("-s")
        .arg(serial)
        .arg("pull")
        .arg(remote)
        .arg(local)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| rust_i18n::t!("adb.error.pull", remote = remote).to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            bail!(
                "{}",
                rust_i18n::t!(
                    "adb.error.pull_status",
                    code = format!("{:?}", output.status.code())
                )
            );
        }
        bail!("{stderr}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_devices_basic() {
        let text = "List of devices attached\nemulator-5554\tdevice\nABC123\toffline\n";
        let devs = parse_devices(text).unwrap();
        assert_eq!(devs.len(), 2);
        assert_eq!(devs[0].serial, "emulator-5554");
        assert_eq!(devs[1].state, "offline");
    }

    #[test]
    fn parse_devices_empty() {
        let text = "List of devices attached\n\n";
        let devs = parse_devices(text).unwrap();
        assert!(devs.is_empty());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebootTarget {
    Normal,
    Recovery,
    Bootloader,
}

pub fn reboot(adb: &str, serial: &str, target: RebootTarget) -> Result<()> {
    crate::validate::serial(serial)?;
    let mut cmd = Command::new(adb);
    cmd.arg("-s").arg(serial).arg("reboot");
    match target {
        RebootTarget::Normal => {}
        RebootTarget::Recovery => {
            cmd.arg("recovery");
        }
        RebootTarget::Bootloader => {
            cmd.arg("bootloader");
        }
    }
    let status = cmd
        .status()
        .with_context(|| "failed 'adb reboot'".to_string())?;
    if !status.success() {
        bail!(
            "{}",
            rust_i18n::t!(
                "adb.error.shell_status",
                code = format!("{:?}", status.code())
            )
        );
    }
    Ok(())
}

pub fn power_off(adb: &str, serial: &str) -> Result<()> {
    crate::validate::serial(serial)?;
    let status = Command::new(adb)
        .args(["-s", serial, "shell", "reboot", "-p"])
        .status()
        .with_context(|| "failed 'adb shell reboot -p'".to_string())?;
    if !status.success() {
        bail!(
            "{}",
            rust_i18n::t!(
                "adb.error.shell_status",
                code = format!("{:?}", status.code())
            )
        );
    }
    Ok(())
}

pub fn getprop(adb: &str, serial: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let adb = adb.to_string();
    let serial = serial.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = Command::new(&adb)
            .args(["-s", &serial, "shell", "getprop"])
            .output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(std::time::Duration::from_millis(4000)) {
        Ok(Ok(out)) => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(String::new()),
    }
}

pub fn dumpsys_battery(adb: &str, serial: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "dumpsys", "battery"])
        .output()
        .with_context(|| rust_i18n::t!("adb.error.battery").to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn df(adb: &str, serial: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "df"])
        .output()
        .with_context(|| rust_i18n::t!("adb.error.df").to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn sm_list_volumes(adb: &str, serial: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "sm", "list-volumes", "all"])
        .output()
        .with_context(|| rust_i18n::t!("adb.error.volumes").to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn df_path(adb: &str, serial: &str, path: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "df", path])
        .output()
        .with_context(|| rust_i18n::t!("adb.error.df_path", path = path).to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmVolume {
    pub id: String,
    pub state: String,
    pub uuid: Option<String>,
}

pub fn parse_sm_volumes(text: &str) -> Vec<SmVolume> {
    let mut result = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let id = parts[0].to_string();
        let state = parts[1].to_string();
        let uuid = parts.get(2).map(|s| s.to_string());
        result.push(SmVolume { id, state, uuid });
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BatteryInfo {
    pub level: Option<String>,
    pub status: Option<String>,
    pub usb_powered: Option<String>,
    pub voltage: Option<String>,
    pub temperature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DfEntry {
    pub filesystem: String,
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub mount: String,
}

pub fn parse_getprop(text: &str) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }
        let Some(close_key) = line.find(']') else {
            continue;
        };
        let key = &line[1..close_key];
        let rest = line[close_key + 1..].trim();
        let Some(rest) = rest.strip_prefix(':') else {
            continue;
        };
        let rest = rest.trim();
        if !rest.starts_with('[') || !rest.ends_with(']') || rest.len() < 2 {
            continue;
        }
        let value = &rest[1..rest.len() - 1];
        map.insert(key.to_string(), value.to_string());
    }
    map
}

pub fn parse_battery(text: &str) -> BatteryInfo {
    let mut info = BatteryInfo::default();
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("level:") {
            info.level = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("status:") {
            info.status = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("USB powered:") {
            info.usb_powered = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("voltage:") {
            info.voltage = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("temperature:") {
            info.temperature = Some(v.trim().to_string());
        }
    }
    info
}

pub fn parse_df(text: &str) -> Vec<DfEntry> {
    let mut entries = Vec::new();
    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }
        let Ok(total) = parts[1].parse::<u64>() else {
            continue;
        };
        let Ok(used) = parts[2].parse::<u64>() else {
            continue;
        };
        let Ok(avail) = parts[3].parse::<u64>() else {
            continue;
        };
        let mount = parts[parts.len() - 1].to_string();
        entries.push(DfEntry {
            filesystem: parts[0].to_string(),
            total_kb: total,
            used_kb: used,
            available_kb: avail,
            mount,
        });
    }
    entries
}

#[cfg(test)]
mod device_info_tests {
    use super::*;

    #[test]
    fn parse_getprop_basic() {
        let text = "[ro.product.model]: [Pixel]\n[ro.build.version.release]: [11]\n";
        let m = parse_getprop(text);
        assert_eq!(m.get("ro.product.model"), Some(&"Pixel".to_string()));
        assert_eq!(m.get("ro.build.version.release"), Some(&"11".to_string()));
    }

    #[test]
    fn parse_battery_basic() {
        let text =
            "  level: 85\n  status: 2\n  USB powered: true\n  voltage: 4123\n  temperature: 295\n";
        let b = parse_battery(text);
        assert_eq!(b.level.as_deref(), Some("85"));
        assert_eq!(b.status.as_deref(), Some("2"));
        assert_eq!(b.usb_powered.as_deref(), Some("true"));
        assert_eq!(b.voltage.as_deref(), Some("4123"));
        assert_eq!(b.temperature.as_deref(), Some("295"));
    }

    #[test]
    fn parse_df_basic() {
        let text =
            "Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/x 1000 400 600 40% /data\n";
        let e = parse_df(text);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].mount, "/data");
        assert_eq!(e[0].total_kb, 1000);
    }

    #[test]
    fn parse_sm_volumes_basic() {
        let text = "private mounted null\npublic:179:65 mounted 0000-0000\nemulated mounted null\n";
        let v = parse_sm_volumes(text);
        assert_eq!(v.len(), 3);
        assert_eq!(v[1].id, "public:179:65");
        assert_eq!(v[1].state, "mounted");
        assert_eq!(v[1].uuid.as_deref(), Some("0000-0000"));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInfo {
    pub ip: Option<String>,
    pub ssid: String,
    pub wifi_state: String,
}

pub fn network_info(adb: &str, serial: &str) -> Result<NetworkInfo> {
    crate::validate::serial(serial)?;
    let mut ip: Option<String> = None;

    if let Ok(out) = Command::new(adb)
        .args(["-s", serial, "shell", "ip", "route", "get", "1.1.1.1"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let words: Vec<&str> = text.split_whitespace().collect();
        for i in 0..words.len() {
            if words[i] == "src" && i + 1 < words.len() {
                let candidate = words[i + 1];
                if !candidate.is_empty() {
                    ip = Some(candidate.to_string());
                    break;
                }
            }
        }
    }

    if ip.is_none()
        && let Ok(out) = Command::new(adb)
            .args(["-s", serial, "shell", "ip", "-4", "addr", "show"])
            .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let line = line.trim();
            if let Some(pos) = line.find("inet ") {
                let rest = &line[pos + 5..];
                let addr = rest.split_whitespace().next().unwrap_or("");
                let addr = addr.split('/').next().unwrap_or(addr);
                if !addr.is_empty() && addr != "127.0.0.1" {
                    ip = Some(addr.to_string());
                    break;
                }
            }
        }
    }

    if ip.is_none()
        && let Ok(out) = Command::new(adb)
            .args(["-s", serial, "shell", "ifconfig"])
            .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Some(pos) = line.find("inet ") {
                let rest = &line[pos + 5..];
                let addr = if let Some(stripped) = rest.strip_prefix("addr:") {
                    stripped
                } else {
                    rest
                };
                let addr = addr.split_whitespace().next().unwrap_or("");
                let addr = addr.split(':').next_back().unwrap_or(addr);
                let addr = addr.split('/').next().unwrap_or(addr);
                if !addr.is_empty() && addr != "127.0.0.1" {
                    ip = Some(addr.to_string());
                    break;
                }
            }
        }
    }

    let wifi_out = Command::new(adb)
        .args(["-s", serial, "shell", "dumpsys", "wifi"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let lower = wifi_out.to_lowercase();
    let mut wifi_state = "unknown".to_string();
    if lower.contains("wi-fi is enabled")
        || lower.contains("wifi is enabled")
        || lower.contains("wi-fi state: enabled")
    {
        wifi_state = "enabled".to_string();
    } else if lower.contains("wi-fi is disabled") || lower.contains("wifi is disabled") {
        wifi_state = "disabled".to_string();
    } else if lower.contains("enabled") {
        wifi_state = "enabled".to_string();
    } else if lower.contains("disabled") {
        wifi_state = "disabled".to_string();
    }

    let mut ssid = String::new();
    for line in wifi_out.lines() {
        if let Some(pos) = line.find("SSID") {
            let rest = &line[pos..];
            if let Some(colon) = rest.find(':') {
                let val = rest[colon + 1..].trim();
                let val = val.trim_matches('"').trim();
                let val = val
                    .split(',')
                    .next()
                    .unwrap_or(val)
                    .trim()
                    .trim_matches('"')
                    .trim();
                if !val.is_empty() {
                    if val == "<unknown ssid>" || val.to_lowercase().contains("unknown ssid") {
                        ssid = crate::i18n::tr("network.status.permissions");
                    } else {
                        ssid = val.to_string();
                    }
                    break;
                }
            }
        }
    }
    if ssid.is_empty() {
        ssid = crate::i18n::tr("network.status.not_available");
    }

    Ok(NetworkInfo {
        ip,
        ssid,
        wifi_state,
    })
}

pub fn wifi_adb_disable(adb: &str, serial: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "usb"])
        .output()
        .with_context(|| "failed 'adb usb'".to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}{stderr}").trim().to_string();
    if !out.status.success() {
        bail!(
            "{}",
            if combined.is_empty() {
                "adb usb failed".to_string()
            } else {
                combined
            }
        );
    }
    Ok(if combined.is_empty() {
        "ok".to_string()
    } else {
        combined
    })
}

pub fn wifi_adb_enable(adb: &str, serial: &str, port: u16) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "tcpip", &port.to_string()])
        .output()
        .with_context(|| "failed 'adb tcpip'".to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}{stderr}").trim().to_string();
    if !out.status.success() {
        bail!(
            "{}",
            if combined.is_empty() {
                "adb tcpip failed".to_string()
            } else {
                combined
            }
        );
    }
    Ok(if combined.is_empty() {
        "ok".to_string()
    } else {
        combined
    })
}

pub fn wifi_adb_connect(adb: &str, endpoint: &str) -> Result<String> {
    let out = Command::new(adb)
        .args(["connect", endpoint])
        .output()
        .with_context(|| "failed 'adb connect'".to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}{stderr}").trim().to_string();
    if !out.status.success() {
        bail!(
            "{}",
            if combined.is_empty() {
                "adb connect failed".to_string()
            } else {
                combined
            }
        );
    }
    let lower = combined.to_lowercase();
    if !lower.contains("connected") || lower.contains("unable") || lower.contains("failed") {
        bail!(
            "{}",
            if combined.is_empty() {
                "adb connect failed".to_string()
            } else {
                combined
            }
        );
    }
    Ok(combined)
}

pub fn wifi_adb_disconnect(adb: &str, endpoint: &str) -> Result<String> {
    let out = Command::new(adb)
        .args(["disconnect", endpoint])
        .output()
        .with_context(|| "failed 'adb disconnect'".to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}{stderr}").trim().to_string();
    if !out.status.success() {
        bail!(
            "{}",
            if combined.is_empty() {
                "adb disconnect failed".to_string()
            } else {
                combined
            }
        );
    }
    Ok(combined)
}

pub fn parse_wifi_adb_endpoints(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let serial = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        let state = parts.next().unwrap_or("unknown");
        if serial.contains(':') && state == "device" && !result.contains(&serial.to_string()) {
            result.push(serial.to_string());
        }
    }
    result
}

pub fn wifi_adb_endpoints(adb: &str) -> Result<Vec<String>> {
    let out = Command::new(adb)
        .arg("devices")
        .output()
        .with_context(|| rust_i18n::t!("adb.error.devices").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(parse_wifi_adb_endpoints(&text))
}

pub fn normalize_network_endpoint(input: &str) -> String {
    let input = input.trim();
    if input.contains(':') {
        input.to_string()
    } else {
        format!("{input}:5555")
    }
}

#[cfg(test)]
mod network_tests {
    use super::normalize_network_endpoint;

    #[test]
    fn normalize_appends_port() {
        assert_eq!(
            normalize_network_endpoint("192.168.1.5"),
            "192.168.1.5:5555"
        );
    }

    #[test]
    fn normalize_keeps_port() {
        assert_eq!(
            normalize_network_endpoint("192.168.1.5:5555"),
            "192.168.1.5:5555"
        );
    }
}

#[cfg(test)]
mod wifi_tests {
    use super::normalize_wifi_endpoint;

    #[test]
    fn normalize_appends_port() {
        assert_eq!(normalize_wifi_endpoint("192.168.1.5"), "192.168.1.5:5555");
    }

    #[test]
    fn normalize_keeps_port() {
        assert_eq!(
            normalize_wifi_endpoint("192.168.1.5:5555"),
            "192.168.1.5:5555"
        );
    }

    #[test]
    fn normalize_empty() {
        assert_eq!(normalize_wifi_endpoint(""), "");
        assert_eq!(normalize_wifi_endpoint("   "), "");
    }

    #[test]
    fn normalize_hostname() {
        assert_eq!(normalize_wifi_endpoint("device.local"), "device.local:5555");
    }
}

pub fn mdns_services(adb: &str) -> Result<Vec<String>> {
    let out = Command::new(adb)
        .args(["mdns", "services"])
        .output()
        .with_context(|| "failed 'adb mdns services'".to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");
    if !out.status.success() && combined.trim().is_empty() {
        bail!("failed 'adb mdns services'");
    }
    let mut result: Vec<String> = Vec::new();
    for line in combined.lines() {
        for token in line.split_whitespace() {
            let token = token.trim_matches(|c: char| c == ',' || c == ';' || c == ')');
            if token.contains(':') {
                let port_part = token.rsplit(':').next().unwrap_or("");
                if port_part.chars().all(|c| c.is_ascii_digit())
                    && !port_part.is_empty()
                    && !result.contains(&token.to_string())
                {
                    result.push(token.to_string());
                }
            }
        }
    }
    Ok(result)
}

pub fn wifi_connect(adb: &str, endpoint: &str) -> Result<String> {
    let out = Command::new(adb)
        .args(["connect", endpoint])
        .output()
        .with_context(|| "failed 'adb connect'".to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let combined = format!("{stdout}{stderr}").trim().to_string();
    let lower = combined.to_lowercase();
    if lower.contains("connected to") || lower.contains("already connected") {
        return Ok(if combined.is_empty() {
            "connected".to_string()
        } else {
            combined
        });
    }
    if !out.status.success() {
        bail!(
            "{}",
            if combined.is_empty() {
                "adb connect failed".to_string()
            } else {
                combined
            }
        );
    }
    bail!(
        "{}",
        if combined.is_empty() {
            "adb connect failed".to_string()
        } else {
            combined
        }
    );
}

/// Entrada de resultado de un escaneo Wi-Fi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiScanEntry {
    pub endpoint: String,
    pub rejected: bool,
}

/// Función pura: determina si la salida de `adb connect` indica conexión exitosa.
/// Se considera conectado solo si el mensaje contiene "connected" y no contiene
/// indicadores de fallo (failed, refused, unable, timed out, no route, etc.).
pub fn connect_succeeded(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("failed")
        || lower.contains("refused")
        || lower.contains("unable")
        || lower.contains("timed out")
        || lower.contains("no route")
        || lower.contains("cannot")
        || lower.contains("error")
    {
        return false;
    }
    lower.contains("connected")
}

/// Intenta `adb connect <endpoint>` con un tiempo de espera corto.
/// Devuelve `Ok(true)` si el dispositivo se conectó, `Ok(false)` si fue rechazado
/// (incluyendo timeout), y `Err` si no se pudo ejecutar adb.
fn adb_connect_with_timeout(adb: &str, endpoint: &str, timeout_ms: u64) -> Result<bool> {
    let adb = adb.to_string();
    let endpoint = endpoint.to_string();
    let (sender, receiver) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let output = std::process::Command::new(&adb)
            .args(["connect", &endpoint])
            .output();
        let _ = sender.send(output);
    });

    match receiver.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
        Ok(Ok(output)) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let _ = handle.join();
            Ok(connect_succeeded(&combined))
        }
        Ok(Err(error)) => {
            let _ = handle.join();
            Err(error.into())
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            let _ = handle.join();
            Ok(false)
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            let _ = handle.join();
            Ok(false)
        }
    }
}

/// Escanea la red local con un pool de workers limitado (32 hilos).
/// Intenta `adb connect <subnet>.x:5555` para x en 1..=254.
/// Cada intento tiene un timeout corto. Los resultados se envían por el canal
/// `sender` a medida que se completan.
pub fn wifi_scan_network(
    adb: &str,
    subnet: &str,
    timeout_ms: u64,
    sender: std::sync::mpsc::Sender<WifiScanEntry>,
) -> Result<()> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(1));
    let num_workers = 32;
    let mut handles = vec![];

    for _ in 0..num_workers {
        let adb = adb.to_string();
        let subnet = subnet.to_string();
        let counter = Arc::clone(&counter);
        let sender = sender.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                let i = counter.fetch_add(1, Ordering::SeqCst);
                if i > 254 {
                    break;
                }
                let endpoint = format!("{subnet}.{i}:5555");
                let connected =
                    adb_connect_with_timeout(&adb, &endpoint, timeout_ms).unwrap_or(false);
                let entry = WifiScanEntry {
                    endpoint,
                    rejected: !connected,
                };
                let _ = sender.send(entry);
            }
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

/// Normaliza un endpoint Wi-Fi añadiendo el puerto 5555 si no tiene uno.
/// Devuelve string vacío si la entrada está vacía o solo tiene espacios.
#[allow(dead_code)]
pub fn normalize_wifi_endpoint(input: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        return String::new();
    }
    if input.contains(':') {
        input.to_string()
    } else {
        format!("{input}:5555")
    }
}

#[cfg(test)]
mod wifi_adb_parse_tests {
    use super::parse_wifi_adb_endpoints;

    #[test]
    fn filters_non_tcp_offline_and_dedups() {
        let text = "List of devices attached\nemulator-5554\tdevice\n192.168.1.10:5555\tdevice\n192.168.1.11:5555\toffline\n192.168.1.10:5555\tdevice\n";
        let result = parse_wifi_adb_endpoints(text);
        assert_eq!(result, vec!["192.168.1.10:5555".to_string()]);
    }
}

// ---------- File browser helpers ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

pub fn list_remote(adb: &str, serial: &str, path: &str) -> Result<Vec<RemoteEntry>> {
    crate::validate::serial(serial)?;
    let target = if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    };
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "ls", "-la", &target])
        .output()
        .with_context(|| format!("failed 'ls {path}'"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.trim().is_empty() || text.contains("No such file") || text.contains("Permission denied")
    {
        return Ok(Vec::new());
    }
    Ok(parse_remote_ls(&text))
}

pub fn parse_remote_ls(text: &str) -> Vec<RemoteEntry> {
    let mut result = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            if let Some(name) = parts.last() {
                if *name != "." && *name != ".." {
                    result.push(RemoteEntry {
                        name: name.to_string(),
                        is_dir: false,
                        size: 0,
                        modified: String::new(),
                    });
                }
            }
            continue;
        }
        let perms = parts[0];
        let is_dir = perms.starts_with('d');
        let size: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
        let modified = if parts.len() >= 8 {
            format!("{} {}", parts[5], parts[6])
        } else {
            parts.get(5).unwrap_or(&"").to_string()
        };
        let name_start = if parts.len() >= 8 { 7 } else { 6 };
        let name = parts[name_start..].join(" ");
        if name == "." || name == ".." || name.is_empty() {
            continue;
        }
        result.push(RemoteEntry {
            name,
            is_dir,
            size,
            modified,
        });
    }
    result
}

pub fn pull_remote(adb: &str, serial: &str, remote: &str, local: &str) -> Result<()> {
    crate::validate::serial(serial)?;
    let status = Command::new(adb)
        .args(["-s", serial, "pull", remote, local])
        .status()
        .with_context(|| format!("failed pull {remote}"))?;
    if !status.success() {
        bail!("pull failed: {remote}");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn push_remote(adb: &str, serial: &str, local: &str, remote: &str) -> Result<()> {
    crate::validate::serial(serial)?;
    let status = Command::new(adb)
        .args(["-s", serial, "push", local, remote])
        .status()
        .with_context(|| format!("failed push {local}"))?;
    if !status.success() {
        bail!("push failed: {local}");
    }
    Ok(())
}

pub fn remove_remote(adb: &str, serial: &str, path: &str) -> Result<()> {
    crate::validate::serial(serial)?;
    if path == "/" || path == "/system" || path == "/data" || path == "/proc" || path == "/dev" {
        bail!("refusing to remove critical path: {path}");
    }
    let status = Command::new(adb)
        .args(["-s", serial, "shell", "rm", "-rf", path])
        .status()
        .with_context(|| format!("failed rm {path}"))?;
    if !status.success() {
        bail!("rm failed: {path}");
    }
    Ok(())
}

pub fn du_remote(adb: &str, serial: &str, path: &str) -> Result<u64> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "du", "-sb", path])
        .output()
        .with_context(|| format!("failed du {path}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let size = text
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    Ok(size)
}

pub fn copy_remote(adb: &str, serial: &str, src: &str, dst: &str) -> Result<()> {
    crate::validate::serial(serial)?;
    if !validate_remote_path(src) || !validate_remote_path(dst) {
        bail!("invalid path");
    }
    let status = Command::new(adb)
        .args(["-s", serial, "shell", "cp", "-r", src, dst])
        .status()
        .with_context(|| format!("failed cp {src}"))?;
    if !status.success() {
        bail!("cp failed: {src}");
    }
    Ok(())
}

pub fn move_remote(adb: &str, serial: &str, src: &str, dst: &str) -> Result<()> {
    crate::validate::serial(serial)?;
    if !validate_remote_path(src) || !validate_remote_path(dst) {
        bail!("invalid path");
    }
    let status = Command::new(adb)
        .args(["-s", serial, "shell", "mv", src, dst])
        .status()
        .with_context(|| format!("failed mv {src}"))?;
    if !status.success() {
        bail!("mv failed: {src}");
    }
    Ok(())
}

pub fn validate_remote_path(path: &str) -> bool {
    let p = path.trim();
    if p.is_empty() || p == "/" {
        return false;
    }
    if p.contains("..") {
        return false;
    }
    let blocked = ["/system", "/proc", "/dev", "/vendor", "/sys"];
    for b in blocked {
        if p == b || p.starts_with(&format!("{b}/")) {
            return false;
        }
    }
    if p == "/data" || (p.starts_with("/data/") && !p.starts_with("/data/local/tmp")) {
        return false;
    }
    true
}

#[cfg(test)]
mod file_browser_tests {
    use super::*;

    #[test]
    fn parse_remote_ls_basic() {
        let text = "total 12\n\
drwxr-xr-x 2 root root 4096 2026-10-28 10:15 DCIM\n\
-rw-r--r-- 1 root root 1234 2026-10-28 10:15 report.pdf\n";
        let result = parse_remote_ls(text);
        assert_eq!(result.len(), 2);
        assert!(result[0].is_dir);
        assert_eq!(result[0].name, "DCIM");
        assert!(!result[1].is_dir);
        assert_eq!(result[1].name, "report.pdf");
        assert_eq!(result[1].size, 1234);
    }

    #[test]
    fn parse_remote_ls_empty() {
        let text = "total 0\n";
        let result = parse_remote_ls(text);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn validate_remote_path_rejects() {
        assert!(!validate_remote_path("/"));
        assert!(!validate_remote_path("/system"));
        assert!(!validate_remote_path("/data"));
        assert!(!validate_remote_path("/foo/../bar"));
        assert!(!validate_remote_path(""));
        assert!(validate_remote_path("/sdcard/Download"));
        assert!(validate_remote_path("/data/local/tmp"));
    }
}

#[cfg(test)]
mod connect_succeeded_tests {
    use super::connect_succeeded;

    #[test]
    fn connected_to_message() {
        assert!(connect_succeeded("connected to 192.168.1.85:5555"));
    }

    #[test]
    fn already_connected_message() {
        assert!(connect_succeeded("already connected to 192.168.1.85:5555"));
    }

    #[test]
    fn failed_connection_refused() {
        assert!(!connect_succeeded(
            "failed to connect to '192.168.1.5:5555': Connection refused"
        ));
    }

    #[test]
    fn failed_no_route_to_host() {
        assert!(!connect_succeeded(
            "failed to connect to '192.168.1.5:5555': No route to host"
        ));
    }

    #[test]
    fn failed_timed_out() {
        assert!(!connect_succeeded(
            "failed to connect to '192.168.1.5:5555': Connection timed out"
        ));
    }

    #[test]
    fn failed_unable() {
        assert!(!connect_succeeded("unable to connect to 192.168.1.5:5555"));
    }

    #[test]
    fn empty_string() {
        assert!(!connect_succeeded(""));
    }

    #[test]
    fn case_insensitive() {
        assert!(connect_succeeded("Connected To 192.168.1.85:5555"));
        assert!(!connect_succeeded(
            "FAILED to connect to '192.168.1.5:5555': Connection Refused"
        ));
    }
}
