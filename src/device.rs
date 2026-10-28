use crate::adb::{self, Device};
use anyhow::{Result, bail};
use std::io::{self, Write};

pub fn list(adb: &str) -> Result<Vec<Device>> {
    adb::devices(adb)
}

#[allow(clippy::collapsible_if)]
pub fn select(adb: &str, serial_override: Option<&str>) -> Result<Device> {
    let all = adb::devices(adb)?;
    let usable: Vec<Device> = all.into_iter().filter(|d| d.state == "device").collect();

    if let Some(s) = serial_override {
        crate::validate::serial(s)?;
        return usable.into_iter().find(|d| d.serial == s).ok_or_else(|| {
            anyhow::anyhow!("{}", rust_i18n::t!("device.error.not_found", serial = s))
        });
    }
    if let Ok(env_serial) = std::env::var("ANDROID_SERIAL") {
        if !env_serial.is_empty() {
            if let Some(d) = usable.iter().find(|d| d.serial == env_serial) {
                return Ok(d.clone());
            }
        }
    }

    match usable.len() {
        0 => bail!("{}", rust_i18n::t!("device.error.none")),
        1 => Ok(usable.into_iter().next().unwrap()),
        _ => {
            println!("{}", rust_i18n::t!("device.select.available_title"));
            for (i, d) in usable.iter().enumerate() {
                println!("  [{}] {}  [{}]", i + 1, d.serial, d.state);
            }
            print!("{}", rust_i18n::t!("device.select.prompt"));
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let idx: usize = input
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("{}", rust_i18n::t!("device.select.invalid")))?;
            if idx == 0 || idx > usable.len() {
                bail!("{}", rust_i18n::t!("device.select.out_of_range"));
            }
            Ok(usable[idx - 1].clone())
        }
    }
}
