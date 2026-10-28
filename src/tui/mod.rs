pub mod app;
pub mod events;
pub mod ui;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::config::Lang;
use crate::i18n;
use app::{App, Screen};

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            DisableMouseCapture,
            LeaveAlternateScreen,
            cursor::Show
        );
    }
}

pub fn run(adb: &str, serial_override: Option<&str>, lang: Lang) -> Result<()> {
    let _restore = RestoreTerminal;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(
        adb.to_string(),
        serial_override.map(|s| s.to_string()),
        lang,
    );
    app.load_devices();

    loop {
        app.poll_script_events();
        app.poll_logcat();
        app.poll_wifi_scan();
        terminal.draw(|f| ui::render(f, &mut app))?;
        if app.should_quit {
            break;
        }
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    events::handle_key(&mut app, key);
                    if app.loading {
                        terminal.draw(|f| ui::render(f, &mut app))?;
                        match app.screen {
                            Screen::WifiDiscover => app.load_wifi_discovered(),
                            _ => app.load_devices(),
                        }
                        app.loading = false;
                    }
                }
                Event::Mouse(mouse) => {
                    if app.screen == Screen::DeviceInfo {
                        match mouse.kind {
                            MouseEventKind::ScrollUp => app.scroll_device_info(-1),
                            MouseEventKind::ScrollDown => app.scroll_device_info(1),
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
            if app.direct_shell_requested {
                app.direct_shell_requested = false;
                let serial = app.serial.clone().unwrap_or_default();
                match run_direct_shell(&mut terminal, app.adb.as_str(), &serial) {
                    Ok(()) => {
                        app.status = i18n::tr("shell.closed");
                    }
                    Err(error) => {
                        app.status = rust_i18n::t!("shell.error", error = error).to_string();
                    }
                }
            }
        }
    }

    app.stop_logcat();
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn run_direct_shell(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    adb: &str,
    serial: &str,
) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show
    )?;
    let status = std::process::Command::new(adb)
        .args(["-s", serial, "shell"])
        .status();
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal.clear()?;
    execute!(io::stdout(), cursor::Show)?;
    match status {
        Ok(_) => Ok(()),
        Err(error) => Err(error.into()),
    }
}
