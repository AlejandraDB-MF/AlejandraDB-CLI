use anyhow::{Result, bail};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};

pub const MAX_LINES: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogcatEvent {
    Line(String),
    Error(String),
}

pub struct LogcatSession {
    child: Child,
    rx: Receiver<LogcatEvent>,
    threads: Vec<JoinHandle<()>>,
}

impl LogcatSession {
    pub fn spawn(adb: &str, serial: &str) -> Result<Self> {
        crate::validate::serial(serial)?;
        let mut child = Command::new(adb)
            .args(["-s", serial, "logcat", "-v", "threadtime", "-T", "200"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                anyhow::anyhow!(
                    "{}",
                    rust_i18n::t!("logcat.error.start", error = error.to_string())
                )
            })?;

        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            bail!("{}", rust_i18n::t!("logcat.error.pipe"));
        };
        let Some(stderr) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            bail!("{}", rust_i18n::t!("logcat.error.pipe"));
        };

        let (sender, rx) = mpsc::channel();
        let stdout_sender = sender.clone();
        let stdout_thread = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if stdout_sender.send(LogcatEvent::Line(line)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = stdout_sender.send(LogcatEvent::Error(error.to_string()));
                        break;
                    }
                }
            }
        });
        let stderr_thread = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) if !line.trim().is_empty() => {
                        if sender.send(LogcatEvent::Error(line)).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(error) => {
                        let _ = sender.send(LogcatEvent::Error(error.to_string()));
                        break;
                    }
                }
            }
        });

        Ok(Self {
            child,
            rx,
            threads: vec![stdout_thread, stderr_thread],
        })
    }

    pub fn try_recv(&self) -> Result<LogcatEvent, TryRecvError> {
        self.rx.try_recv()
    }
}

impl Drop for LogcatSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
    }
}
