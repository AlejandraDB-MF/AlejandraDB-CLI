use anyhow::{Context, Result, bail};
use std::process::Command;

pub fn list(adb: &str, serial: &str, opts: &crate::cli::ListOpts) -> Result<Vec<String>> {
    crate::validate::serial(serial)?;
    let mut args: Vec<String> = vec![
        "-s".into(),
        serial.into(),
        "shell".into(),
        "pm".into(),
        "list".into(),
        "packages".into(),
    ];
    if opts.third_party {
        args.push("-3".into());
    }
    if opts.system {
        args.push("-s".into());
    }
    if opts.disabled {
        args.push("-d".into());
    }
    if opts.enabled {
        args.push("-e".into());
    }
    args.push("--user".into());
    args.push(opts.user.to_string());
    let out = Command::new(adb)
        .args(&args)
        .output()
        .with_context(|| rust_i18n::t!("package.error.list").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut pkgs = parse_list(&text);
    if let Some(f) = &opts.filter {
        let f = f.to_lowercase();
        pkgs.retain(|p| p.to_lowercase().contains(&f));
    }
    pkgs.sort();
    Ok(pkgs)
}

pub fn parse_list(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            l.strip_prefix("package:").map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn uninstall(
    adb: &str,
    serial: &str,
    pkg: &str,
    keep_data: bool,
    system: bool,
) -> Result<String> {
    crate::validate::serial(serial)?;
    crate::validate::package(pkg)?;

    // Intento 1: adb uninstall (apps de usuario)
    let mut args = vec![
        "-s".to_string(),
        serial.to_string(),
        "uninstall".to_string(),
    ];
    if keep_data {
        args.push("-k".to_string());
    }
    args.push(pkg.to_string());
    let out = Command::new(adb).args(&args).output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stdout.contains("Success") {
        return Ok(rust_i18n::t!("package.result.uninstalled").to_string());
    }

    // Intento 2: pm uninstall --user 0 (apps de sistema / por usuario)
    let mut pargs = vec![
        "-s".to_string(),
        serial.to_string(),
        "shell".to_string(),
        "pm".to_string(),
        "uninstall".to_string(),
    ];
    if keep_data {
        pargs.push("-k".to_string());
    }
    pargs.push("--user".to_string());
    pargs.push("0".to_string());
    pargs.push(pkg.to_string());
    let out2 = Command::new(adb).args(&pargs).output()?;
    let s2 = String::from_utf8_lossy(&out2.stdout);
    if s2.contains("Success") {
        return Ok(rust_i18n::t!("package.result.uninstalled_user0").to_string());
    }

    // Intento 3 (solo si system): deshabilitar en lugar de borrar
    if system {
        let dargs = vec![
            "-s".to_string(),
            serial.to_string(),
            "shell".to_string(),
            "pm".to_string(),
            "disable-user".to_string(),
            "--user".to_string(),
            "0".to_string(),
            pkg.to_string(),
        ];
        let out3 = Command::new(adb).args(&dargs).output()?;
        let s3 = String::from_utf8_lossy(&out3.stdout);
        if s3.contains("disabled") || out3.status.success() {
            return Ok(rust_i18n::t!("package.result.disabled").to_string());
        }
    }

    bail!(
        "{}",
        rust_i18n::t!(
            "package.error.uninstall",
            stdout = stdout.trim(),
            stderr = stderr.trim()
        )
    )
}

pub fn available_system(adb: &str, serial: &str) -> Result<Vec<String>> {
    crate::validate::serial(serial)?;
    // `pm list packages -u` includes uninstalled (system) packages marked with a trailing indicator.
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "pm", "list", "packages", "-u"])
        .output()
        .with_context(|| rust_i18n::t!("package.error.available").to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let all = parse_list(&text);

    // installed set
    let out2 = Command::new(adb)
        .args(["-s", serial, "shell", "pm", "list", "packages"])
        .output()
        .with_context(|| rust_i18n::t!("package.error.list").to_string())?;
    let text2 = String::from_utf8_lossy(&out2.stdout);
    let installed: std::collections::HashSet<String> = parse_list(&text2).into_iter().collect();

    let mut avail: Vec<String> = all.into_iter().filter(|p| !installed.contains(p)).collect();
    avail.sort();
    avail.dedup();
    Ok(avail)
}

pub fn install_existing(adb: &str, serial: &str, pkg: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    crate::validate::package(pkg)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "pm", "install-existing", pkg])
        .output()
        .with_context(|| rust_i18n::t!("package.error.install_existing_cmd").to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stdout.to_lowercase().contains("success") || out.status.success() {
        return Ok(rust_i18n::t!("package.result.installed").to_string());
    }
    // fallback
    let out2 = Command::new(adb)
        .args([
            "-s",
            serial,
            "shell",
            "cmd",
            "package",
            "install-existing",
            pkg,
        ])
        .output()?;
    let s2 = String::from_utf8_lossy(&out2.stdout);
    if s2.to_lowercase().contains("success") || out2.status.success() {
        return Ok(rust_i18n::t!("package.result.installed").to_string());
    }
    bail!(
        "{}",
        rust_i18n::t!(
            "package.error.install_existing",
            stdout = stdout.trim(),
            stderr = stderr.trim()
        )
    )
}

pub fn install_apk(adb: &str, serial: &str, path: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    let out = Command::new(adb)
        .args(["-s", serial, "install", "-r", path])
        .output()
        .with_context(|| rust_i18n::t!("package.error.install_cmd").to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stdout.to_lowercase().contains("success") || out.status.success() {
        return Ok(rust_i18n::t!("package.result.installed").to_string());
    }
    bail!(
        "{}",
        rust_i18n::t!(
            "package.error.install",
            stdout = stdout.trim(),
            stderr = stderr.trim()
        )
    )
}

pub fn enable(adb: &str, serial: &str, pkg: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    crate::validate::package(pkg)?;
    let out = Command::new(adb)
        .args(["-s", serial, "shell", "pm", "enable", pkg])
        .output()
        .with_context(|| rust_i18n::t!("package.error.enable_cmd").to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stdout.to_lowercase().contains("enabled") || out.status.success() {
        return Ok(rust_i18n::t!("package.result.enabled").to_string());
    }
    // fallback user-scoped
    let out2 = Command::new(adb)
        .args([
            "-s",
            serial,
            "shell",
            "pm",
            "enable-user",
            "--user",
            "0",
            pkg,
        ])
        .output()?;
    let s2 = String::from_utf8_lossy(&out2.stdout);
    if s2.to_lowercase().contains("enabled") || out2.status.success() {
        return Ok(rust_i18n::t!("package.result.enabled").to_string());
    }
    bail!(
        "{}",
        rust_i18n::t!(
            "package.error.enable",
            stdout = stdout.trim(),
            stderr = stderr.trim()
        )
    )
}

#[allow(dead_code)]
pub fn disable(adb: &str, serial: &str, pkg: &str) -> Result<String> {
    crate::validate::serial(serial)?;
    crate::validate::package(pkg)?;
    let out = Command::new(adb)
        .args([
            "-s",
            serial,
            "shell",
            "pm",
            "disable-user",
            "--user",
            "0",
            pkg,
        ])
        .output()
        .with_context(|| rust_i18n::t!("package.error.disable_cmd").to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.to_lowercase().contains("disabled") || out.status.success() {
        return Ok(rust_i18n::t!("package.result.disable").to_string());
    }
    bail!(
        "{}",
        rust_i18n::t!("package.error.disable", stdout = stdout.trim())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_packages() {
        let text = "package:com.foo\npackage:com.bar.baz\nnot-a-package-line\n";
        let pkgs = parse_list(text);
        assert_eq!(pkgs, vec!["com.foo", "com.bar.baz"]);
    }
}
