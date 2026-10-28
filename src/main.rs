mod adb;
mod cli;
mod config;
mod device;
mod i18n;
mod logcat;
mod package;
mod processes;
mod safety;
mod scripts;
mod theme;
mod tui;
mod validate;

rust_i18n::i18n!("locales", fallback = "en");

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use config::Config;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}: {err:#}", i18n::tr("error.prefix"));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config = Config::load();
    i18n::init(config.language);

    let cli = Cli::parse();
    let adb = cli
        .adb
        .clone()
        .unwrap_or_else(|| std::env::var("ADB_PATH").unwrap_or_else(|_| "adb".to_string()));

    match cli.command {
        None | Some(Command::Tui) => tui::run(&adb, cli.serial.as_deref(), config.language),
        Some(Command::Devices) => {
            let devs = device::list(&adb)?;
            for d in devs {
                println!("{}\t{}", d.serial, d.state);
            }
            Ok(())
        }
        Some(Command::Apps { action }) => {
            let action = action.unwrap_or(cli::AppsAction::Interactive);
            match action {
                cli::AppsAction::List(opts) => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    let pkgs = package::list(&adb, &dev.serial, &opts)?;
                    for p in pkgs {
                        println!("{p}");
                    }
                    Ok(())
                }
                cli::AppsAction::Uninstall {
                    packages,
                    keep_data,
                    system,
                    force,
                } => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    let system_set = safety::system_packages(&adb, &dev.serial).unwrap_or_default();
                    let admin_set = safety::device_admins(&adb, &dev.serial).unwrap_or_default();
                    let package_paths =
                        safety::package_paths(&adb, &dev.serial).unwrap_or_default();
                    for pkg in packages {
                        validate::package(&pkg)?;
                        let path = package_paths.get(&pkg);
                        let kind = safety::classify(
                            &pkg,
                            &system_set,
                            &admin_set,
                            path.map(|s| s.as_str()),
                        );
                        match kind {
                            safety::AppKind::Critical if !force => {
                                eprintln!("{}", rust_i18n::t!("cli.error.critical", pkg = pkg));
                                continue;
                            }
                            safety::AppKind::DeviceAdmin if !force => {
                                eprintln!("{}", rust_i18n::t!("cli.error.admin", pkg = pkg));
                                continue;
                            }
                            safety::AppKind::System => {
                                eprintln!("{}", rust_i18n::t!("cli.warning.system", pkg = pkg));
                            }
                            _ => {}
                        }
                        let sys = system
                            || matches!(kind, safety::AppKind::System | safety::AppKind::Critical);
                        let res = package::uninstall(&adb, &dev.serial, &pkg, keep_data, sys)?;
                        println!("{pkg}: {res}");
                    }
                    Ok(())
                }
                cli::AppsAction::Install { path } => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    let res = package::install_apk(&adb, &dev.serial, &path)?;
                    println!("{path}: {res}");
                    Ok(())
                }
                cli::AppsAction::InstallExisting { packages } => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    for pkg in packages {
                        validate::package(&pkg)?;
                        let res = package::install_existing(&adb, &dev.serial, &pkg)?;
                        println!("{pkg}: {res}");
                    }
                    Ok(())
                }
                cli::AppsAction::Enable { packages } => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    for pkg in packages {
                        validate::package(&pkg)?;
                        let res = package::enable(&adb, &dev.serial, &pkg)?;
                        println!("{pkg}: {res}");
                    }
                    Ok(())
                }
                cli::AppsAction::Disable { packages } => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    for pkg in packages {
                        validate::package(&pkg)?;
                        let res = package::disable(&adb, &dev.serial, &pkg)?;
                        println!("{pkg}: {res}");
                    }
                    Ok(())
                }
                cli::AppsAction::Interactive => {
                    tui::run(&adb, cli.serial.as_deref(), config.language)
                }
            }
        }
        Some(Command::Scripts { action }) => {
            use crate::scripts::{
                ScriptFolder, create_timestamp_dir, join_remote, resolve_storage,
            };
            use std::fs;
            match action {
                cli::ScriptsAction::Run { folders, all } => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    let selected: Vec<ScriptFolder> = if all {
                        ScriptFolder::ALL.to_vec()
                    } else {
                        folders
                            .iter()
                            .filter_map(|value| {
                                let needle = value.to_lowercase();
                                ScriptFolder::ALL
                                    .iter()
                                    .copied()
                                    .find(|folder| folder.label().to_lowercase() == needle)
                            })
                            .collect()
                    };
                    if selected.is_empty() {
                        anyhow::bail!("no valid folders selected");
                    }
                    let output_path = config.output_dir();
                    fs::create_dir_all(&output_path)?;
                    let root = resolve_storage(&adb, &dev.serial, &selected)?;
                    let timestamp = crate::scripts::timestamp_now();
                    let destination = create_timestamp_dir(&output_path, &timestamp)?;
                    let mut copied = 0usize;
                    let mut skipped = 0usize;
                    let mut failed = 0usize;
                    for folder in selected {
                        let mut did_copy = false;
                        let mut last_error: Option<String> = None;
                        for candidate in folder.remote_candidates() {
                            let remote = join_remote(&root, candidate);
                            if !crate::adb::remote_dir_exists(&adb, &dev.serial, &remote) {
                                continue;
                            }
                            let local = destination.join(folder.label());
                            match crate::adb::pull(&adb, &dev.serial, &remote, &local) {
                                Ok(()) => {
                                    did_copy = true;
                                    break;
                                }
                                Err(error) => {
                                    last_error = Some(error.to_string());
                                    break;
                                }
                            }
                        }
                        if did_copy {
                            copied += 1;
                        } else if last_error.is_some() {
                            failed += 1;
                        } else {
                            skipped += 1;
                        }
                    }
                    println!(
                        "copied: {}, skipped: {}, failed: {} -> {}",
                        copied,
                        skipped,
                        failed,
                        destination.display()
                    );
                    Ok(())
                }
            }
        }
        Some(Command::Processes { action }) => {
            use crate::processes;
            match action {
                cli::ProcessesAction::List => {
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    let list = processes::list(&adb, &dev.serial)?;
                    for process in list {
                        println!("{}\t{}\t{}", process.pid, process.user, process.name);
                    }
                    Ok(())
                }
                cli::ProcessesAction::Kill { pid, force: _ } => {
                    if pid == 1 {
                        anyhow::bail!("{}", rust_i18n::t!("processes.status.protected"));
                    }
                    let dev = device::select(&adb, cli.serial.as_deref())?;
                    processes::kill(&adb, &dev.serial, pid)?;
                    println!("PID {} killed", pid);
                    Ok(())
                }
            }
        }
        Some(Command::Logcat {
            lines,
            duration,
            output,
        }) => {
            use std::io::{BufRead, Write};
            use std::process::{Command, Stdio};
            use std::sync::mpsc;
            use std::thread;
            use std::time::{Duration, Instant};

            let dev = device::select(&adb, cli.serial.as_deref())?;
            use std::fs;
            let mut cmd = Command::new(&adb);
            cmd.args(["-s", &dev.serial, "logcat"]);
            if let Some(count) = lines {
                cmd.args(["-T", &count.to_string()]);
            }
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
            let mut child = cmd.spawn()?;
            let stdout = child.stdout.take().unwrap();
            let (sender, receiver) = mpsc::channel::<String>();
            thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            if sender.send(line).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            let deadline = if lines.is_none() && duration.is_none() {
                Some(Instant::now() + Duration::from_secs(5))
            } else {
                duration.map(|seconds| Instant::now() + Duration::from_secs(seconds))
            };
            let target_lines = lines;
            let mut collected: Vec<String> = Vec::new();
            loop {
                if let Some(target) = target_lines
                    && collected.len() >= target
                {
                    break;
                }
                if let Some(dl) = deadline
                    && Instant::now() >= dl
                {
                    break;
                }
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(line) => collected.push(line),
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            let _ = child.kill();
            let _ = child.wait();

            if let Some(path) = output {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::File::create(&path)?;
                for line in &collected {
                    writeln!(file, "{}", line)?;
                }
                eprintln!("wrote {} lines to {}", collected.len(), path.display());
            } else {
                for line in &collected {
                    println!("{}", line);
                }
            }
            Ok(())
        }
        Some(Command::Shell { args }) => {
            let dev = device::select(&adb, cli.serial.as_deref())?;
            adb::shell(&adb, &dev.serial, &args)
        }
    }
}
