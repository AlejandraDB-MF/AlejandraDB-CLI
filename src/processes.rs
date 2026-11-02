use anyhow::{Context, Result, bail};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub user: String,
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
}

pub fn list(adb: &str, serial: &str) -> Result<Vec<ProcessInfo>> {
    crate::validate::serial(serial)?;
    let attempts: [&[&str]; 3] = [
        &[
            "-s",
            serial,
            "shell",
            "ps",
            "-A",
            "-o",
            "USER,PID,PPID,NAME",
        ],
        &["-s", serial, "shell", "ps", "-A"],
        &["-s", serial, "shell", "ps"],
    ];
    let mut last_error: Option<String> = None;
    for args in attempts {
        match Command::new(adb).args(args).output() {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout);
                let list = parse_ps(&text);
                if !list.is_empty() {
                    return Ok(list);
                }
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    last_error = Some(if stderr.is_empty() {
                        rust_i18n::t!(
                            "processes.error.ps_status",
                            code = format!("{:?}", output.status.code())
                        )
                        .to_string()
                    } else {
                        stderr
                    });
                }
            }
            Err(error) => {
                last_error = Some(error.to_string());
            }
        }
    }
    if let Some(error) = last_error {
        bail!("{error}");
    }
    Ok(Vec::new())
}

#[allow(clippy::collapsible_if)]
pub fn parse_ps(text: &str) -> Vec<ProcessInfo> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let (header, data_lines): (Option<Vec<&str>>, &[&str]) =
        if lines[0].split_whitespace().any(|t| t == "PID") {
            (Some(lines[0].split_whitespace().collect()), &lines[1..])
        } else {
            (None, &lines[..])
        };

    let mut result = Vec::new();
    for line in data_lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        // Try header-based parsing first.
        let mut parsed: Option<ProcessInfo> = None;
        if let Some(h) = &header {
            let user_idx = h
                .iter()
                .position(|t| *t == "USER" || *t == "UID")
                .unwrap_or(0);
            let pid_idx = h.iter().position(|t| *t == "PID").unwrap_or(1);
            let ppid_idx = h.iter().position(|t| *t == "PPID").unwrap_or(2);
            let name_idx = h
                .iter()
                .position(|t| *t == "NAME" || *t == "CMD" || *t == "COMMAND" || *t == "ARGS")
                .unwrap_or_else(|| h.len().saturating_sub(1));

            if user_idx < parts.len()
                && pid_idx < parts.len()
                && ppid_idx < parts.len()
                && name_idx < parts.len()
            {
                if let (Ok(pid), Ok(ppid)) = (
                    parts[pid_idx].parse::<u32>(),
                    parts[ppid_idx].parse::<u32>(),
                ) {
                    // If data has more columns than header, an extra state column
                    // (e.g. "S") may precede NAME; skip it.
                    let mut ni = name_idx;
                    if parts.len() > h.len() && ni < parts.len() {
                        let tok = parts[ni];
                        if tok.len() == 1 && "SRZTDI".contains(tok) && ni + 1 < parts.len() {
                            ni += 1;
                        }
                    }
                    if ni < parts.len() {
                        let name = parts[ni..].join(" ");
                        if !name.is_empty() {
                            parsed = Some(ProcessInfo {
                                user: parts[user_idx].to_string(),
                                pid,
                                ppid,
                                name,
                            });
                        }
                    }
                }
            }
        }

        if parsed.is_none() {
            // Fallback heuristic for headerless or unexpected formats.
            let Ok(pid) = parts[1].parse::<u32>() else {
                continue;
            };
            let Ok(ppid) = parts[2].parse::<u32>() else {
                continue;
            };
            let name = if parts.len() == 4 {
                parts[3].to_string()
            } else if parts[3].parse::<u64>().is_ok() {
                parts[parts.len() - 1].to_string()
            } else {
                parts[3..].join(" ")
            };
            parsed = Some(ProcessInfo {
                user: parts[0].to_string(),
                pid,
                ppid,
                name,
            });
        }

        if let Some(p) = parsed {
            result.push(p);
        }
    }

    result.sort_by_key(|process| process.pid);
    result
}

pub fn kill(adb: &str, serial: &str, pid: u32) -> Result<()> {
    if pid == 1 {
        bail!("{}", rust_i18n::t!("processes.status.protected"));
    }
    crate::validate::serial(serial)?;
    let output = Command::new(adb)
        .args(["-s", serial, "shell", "kill", &pid.to_string()])
        .output()
        .with_context(|| rust_i18n::t!("processes.error.kill_cmd").to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}").trim().to_string();
    let lower = combined.to_lowercase();
    let failed = !output.status.success()
        || lower.contains("not permitted")
        || lower.contains("permission denied")
        || lower.contains("no such process");
    if failed {
        if combined.is_empty() {
            bail!(
                "{}",
                rust_i18n::t!(
                    "processes.error.kill_status",
                    code = format!("{:?}", output.status.code())
                )
            );
        }
        bail!("{combined}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_output() {
        let text = "USER PID PPID NAME\nroot 1 0 /init\nu0_a12 321 1 com.android.systemui\n";
        let list = parse_ps(text);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].user, "root");
        assert_eq!(list[0].pid, 1);
        assert_eq!(list[0].ppid, 0);
        assert_eq!(list[0].name, "/init");
        assert_eq!(list[1].name, "com.android.systemui");
    }

    #[test]
    fn parses_legacy_android_output() {
        let text = "USER PID PPID VSIZE RSS WCHAN PC NAME\nroot 1 0 21628 1020 SyS_epoll_ 0000000000 S /init\nu0_a12 456 1 1000 200 0 0 S com.android.phone\n";
        let list = parse_ps(text);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].pid, 1);
        assert_eq!(list[0].name, "/init");
        assert_eq!(list[1].user, "u0_a12");
        assert_eq!(list[1].ppid, 1);
        assert_eq!(list[1].name, "com.android.phone");
    }

    #[test]
    fn preserves_compact_names_with_spaces() {
        let text = "USER PID PPID NAME\nroot 9 1 kernel worker\n";
        let list = parse_ps(text);
        assert_eq!(list[0].name, "kernel worker");
    }

    #[test]
    fn skips_headers_and_non_numeric_pids() {
        let text = "USER PID PPID NAME\ninvalid row here\nroot notapid 0 /init\n";
        assert!(parse_ps(text).is_empty());
    }

    #[test]
    fn sorts_by_pid_ascending() {
        let text = "USER PID PPID NAME\nroot 10 1 ten\nroot 2 1 two\n";
        let list = parse_ps(text);
        assert_eq!(list[0].pid, 2);
        assert_eq!(list[1].pid, 10);
    }

    #[test]
    fn refuses_to_kill_pid_one() {
        assert!(kill("adb", "serial", 1).is_err());
    }
}
