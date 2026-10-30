use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::config::Lang;
use crate::i18n;
use crate::scripts::{BUILTIN_SCRIPTS, EL_NEGRO_SAMUEL, ScriptFolder};
use crate::theme::Theme;
use crate::tui::app::{App, AppAction, AppView, NetworkAction, Screen};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if app.status.is_empty() {
            vec![Constraint::Min(3)]
        } else {
            vec![Constraint::Min(3), Constraint::Length(3)]
        })
        .split(area);

    match app.screen {
        Screen::DeviceSelect => render_device_select(f, app, chunks[0]),
        Screen::MainMenu => render_main_menu(f, app, chunks[0]),
        Screen::Scripts => render_scripts(f, app, chunks[0]),
        Screen::ScriptFolders => render_script_folders(f, app, chunks[0]),
        Screen::Logcat => render_logcat(f, app, chunks[0]),
        Screen::Processes => render_processes(f, app, chunks[0]),
        Screen::DeviceMenu => render_device_menu(f, app, chunks[0]),
        Screen::Settings => render_settings(f, app, chunks[0]),
        Screen::OutputPath => render_output_path(f, app, chunks[0]),
        Screen::Language => render_language(f, app, chunks[0]),
        Screen::AppsMenu => render_apps_menu(f, app, chunks[0]),
        Screen::AppList => render_app_list(f, app, chunks[0]),
        Screen::InstallApk => render_install_apk(f, app, chunks[0]),
        Screen::Confirm => render_confirm(f, app, chunks[0]),
        Screen::DeviceInfo => render_device_info(f, app, chunks[0]),
        Screen::DeviceControl => render_device_control(f, app, chunks[0]),
        Screen::Network => render_network(f, app, chunks[0]),
        Screen::WifiDiscover => render_wifi_discover(f, app, chunks[0]),
        Screen::WifiConnect => render_wifi_connect(f, app, chunks[0]),
        Screen::FileBrowser => render_file_browser(f, app, chunks[0]),
        Screen::StorageMenu => render_storage_menu(f, app, chunks[0]),
        Screen::StorageUsage => render_storage_usage(f, app, chunks[0]),
        Screen::StorageVolumes => render_storage_volumes(f, app, chunks[0]),
    }

    if !app.status.is_empty() {
        let status = Paragraph::new(Line::from(app.status.clone()))
            .style(Style::default().fg(ratatui::style::Color::Gray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(status, chunks[1]);
    }
}

fn list_items(keys: &[&str]) -> Vec<ListItem<'static>> {
    keys.iter()
        .map(|key| ListItem::new(i18n::tr(key)))
        .collect()
}

fn menu_block(title: String) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Theme::border())
        .title_style(Theme::title())
}

fn render_device_select(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let title = i18n::tr("device.select.title");
    if app.loading {
        let para = Paragraph::new(i18n::tr("device.select.loading")).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    let mut items: Vec<ListItem> = app
        .devices
        .iter()
        .map(|d| {
            let conn = if d.serial.contains(':') {
                " (Wi-Fi)"
            } else {
                " (USB)"
            };
            ListItem::new(format!(
                "{} {} ({}) [{}]{}",
                d.manufacturer, d.model, d.serial, d.state, conn
            ))
        })
        .collect();
    items.push(ListItem::new(i18n::tr("device.select.discover")));
    items.push(ListItem::new(i18n::tr("device.select.connect_ip")));
    let mut state = app.device_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.device_state = state;
}

fn render_main_menu(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items = list_items(&app.menu_keys);
    let mut state = app.menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("menu.main.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.menu_state = state;
}

fn scripts_device_label(app: &App) -> String {
    app.serial
        .clone()
        .unwrap_or_else(|| i18n::tr("scripts.device.none"))
}

fn render_scripts(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let title = format!(
        "{} - {}: {}",
        i18n::tr("scripts.title"),
        i18n::tr("scripts.device"),
        scripts_device_label(app)
    );
    let items: Vec<ListItem> = BUILTIN_SCRIPTS
        .iter()
        .map(|script| {
            let label = if app.script_running && *script == EL_NEGRO_SAMUEL {
                format!("{script} ({})", i18n::tr("scripts.running"))
            } else {
                (*script).to_string()
            };
            ListItem::new(label)
        })
        .collect();
    let mut state = app.scripts_menu_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.scripts_menu_state = state;
}

fn render_script_folders(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let title = format!(
        "{} - {}: {}",
        EL_NEGRO_SAMUEL,
        i18n::tr("scripts.device"),
        scripts_device_label(app)
    );
    let items: Vec<ListItem> = ScriptFolder::ALL
        .iter()
        .map(|folder| {
            let mark = if app.script_selected.contains(folder) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{mark} {}", folder.label()))
        })
        .collect();
    let mut state = app.script_folder_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.script_folder_state = state;
}

fn render_logcat(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let serial = app.serial.as_deref().unwrap_or("-");
    let title = rust_i18n::t!("logcat.title", serial = serial).to_string();
    let items: Vec<ListItem> = app
        .logcat_lines
        .iter()
        .map(|line| ListItem::new(line.clone()))
        .collect();
    let mut state = app.logcat_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.logcat_state = state;
}

fn render_processes(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let serial = app.serial.as_deref().unwrap_or("-");
    let title = rust_i18n::t!("processes.title", serial = serial).to_string();
    let block = menu_block(title);
    let inner = block.inner(area);
    f.render_widget(block, area);
    if let Some(process) = &app.process_pending {
        let text = rust_i18n::t!(
            "processes.confirm",
            pid = process.pid,
            name = process.name,
            user = process.user
        )
        .to_string();
        let para = Paragraph::new(text).style(Theme::normal());
        f.render_widget(para, inner);
        return;
    }
    let filtered_indices = app.filtered_process_indices();
    let filtered_len = filtered_indices.len();
    if filtered_len == 0 {
        app.process_state.select(None);
    } else if let Some(selected) = app.process_state.selected() {
        if selected >= filtered_len {
            app.process_state.select(Some(filtered_len - 1));
        }
    } else {
        app.process_state.select(Some(0));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);
    let filter_line = if app.process_filter.is_empty() {
        rust_i18n::t!("processes.status.loaded", count = filtered_len).to_string()
    } else {
        format!("{} ({})", app.process_filter, filtered_len)
    };
    f.render_widget(
        Paragraph::new(filter_line).style(Style::default().fg(Color::Gray)),
        chunks[0],
    );
    let header = format!(
        "{:<8} {:<16} {}",
        i18n::tr("processes.header.pid"),
        i18n::tr("processes.header.user"),
        i18n::tr("processes.header.name")
    );
    f.render_widget(
        Paragraph::new(header).style(Style::default().fg(Color::Gray)),
        chunks[1],
    );
    let items: Vec<ListItem> = filtered_indices
        .iter()
        .filter_map(|index| app.processes.get(*index))
        .map(|process| {
            ListItem::new(format!(
                "{:<8} {:<16} {}",
                process.pid, process.user, process.name
            ))
        })
        .collect();
    let mut state = app.process_state;
    let list = List::new(items)
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[2], &mut state);
    app.process_state = state;
}

fn render_device_menu(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items = list_items(&app.device_menu_keys);
    let mut state = app.device_menu_state;
    let serial = app.serial.as_deref().unwrap_or("-");
    let title = rust_i18n::t!("device.menu.title", serial = serial).to_string();
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.device_menu_state = state;
}

fn render_output_path(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = menu_block(i18n::tr("settings.output_path"));
    let inner_width = block.inner(area).width as usize;
    let max_path_width = inner_width.saturating_sub(3);
    let visible_path = tail_chars(&app.output_path_input, max_path_width);

    let input = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(visible_path, Theme::normal()),
        Span::styled("_", Style::default().fg(Color::Cyan)),
    ]);

    let lines = vec![
        Line::from(i18n::tr("settings.output_path")),
        input,
        Line::from(""),
        Line::from(Span::styled(
            i18n::tr("settings.output_path.hint"),
            Style::default().fg(Color::Gray),
        )),
    ];
    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

fn render_settings(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items = list_items(&app.settings_menu_keys);
    let mut state = app.settings_menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("settings.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.settings_menu_state = state;
}

fn render_language(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .language_menu_keys
        .iter()
        .enumerate()
        .map(|(index, key)| {
            let mut label = i18n::tr(key);
            let is_current = (index == 0 && app.lang == Lang::English)
                || (index == 1 && app.lang == Lang::Spanish);
            if is_current {
                label = format!("✓ {label} ({})", i18n::tr("language.current"));
            }
            ListItem::new(label)
        })
        .collect();
    let mut state = app.language_menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("language.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.language_menu_state = state;
}

fn render_apps_menu(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items = list_items(&app.apps_menu_keys);
    let mut state = app.apps_menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("apps.menu.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.apps_menu_state = state;
}

fn render_app_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = {
        let filtered: Vec<String> = app.current_list().clone();
        filtered
            .iter()
            .map(|p| {
                let mark = if app.selected.contains(p) {
                    "[x]"
                } else {
                    "[ ]"
                };
                let kind = app.kind(p);
                let badge = match kind {
                    crate::safety::AppKind::Critical => i18n::tr("apps.badge.critical"),
                    crate::safety::AppKind::System if p.starts_with("com.google.") => {
                        i18n::tr("apps.badge.system_google")
                    }
                    crate::safety::AppKind::System => i18n::tr("apps.badge.system"),
                    crate::safety::AppKind::DeviceAdmin => i18n::tr("apps.badge.admin"),
                    crate::safety::AppKind::User => String::new(),
                };
                let style = match kind {
                    crate::safety::AppKind::Critical => Style::default()
                        .fg(Color::Red)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                    crate::safety::AppKind::System => Style::default().fg(Color::Yellow),
                    crate::safety::AppKind::DeviceAdmin => Style::default().fg(Color::Magenta),
                    crate::safety::AppKind::User => Style::default(),
                };
                let label = if badge.is_empty() {
                    format!("{mark} {p}")
                } else {
                    format!("{mark} {p} {badge}")
                };
                ListItem::new(label).style(style)
            })
            .collect()
    };

    let view_name = match app.app_view {
        AppView::Installed => i18n::tr("apps.view.installed"),
        AppView::Available => i18n::tr("apps.view.available"),
    };
    let action_name = match app.app_action {
        AppAction::Uninstall => i18n::tr("apps.action.uninstall"),
        AppAction::Reinstall => i18n::tr("apps.action.reinstall"),
        AppAction::Enable => i18n::tr("apps.action.enable"),
        AppAction::ClearCache => i18n::tr("storage.cache.title"),
    };
    // Uninstall view shows all installed packages (system apps included, marked).
    let uninstall_all =
        app.app_action == AppAction::Uninstall && app.app_view == AppView::Installed;
    let core = if uninstall_all {
        i18n::tr("apps.core.uninstall_all")
    } else {
        format!("{action_name} - {view_name}")
    };
    let mut title = if app.force_mode {
        if app.filter.is_empty() && !app.filter_mode {
            rust_i18n::t!("apps.title.force", core = core).to_string()
        } else {
            let filter = if app.filter_mode {
                format!("{}_", app.filter)
            } else {
                app.filter.clone()
            };
            rust_i18n::t!("apps.title.force_filter", core = core, filter = filter).to_string()
        }
    } else if app.filter.is_empty() && !app.filter_mode {
        rust_i18n::t!("apps.title.normal", core = core).to_string()
    } else {
        let filter = if app.filter_mode {
            format!("{}_", app.filter)
        } else {
            app.filter.clone()
        };
        rust_i18n::t!("apps.title.filter", core = core, filter = filter).to_string()
    };
    title = rust_i18n::t!(
        "apps.title.sort",
        title = title,
        mode = app.sort_mode.label()
    )
    .to_string();

    let mut state = app.app_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.app_state = state;
}

fn render_install_apk(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = menu_block(i18n::tr("apps.install.title"));
    let inner_width = block.inner(area).width as usize;
    let max_path_width = inner_width.saturating_sub(3);
    let visible_path = tail_chars(&app.apk_input, max_path_width);

    let input = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(visible_path, Theme::normal()),
        Span::styled("_", Style::default().fg(Color::Cyan)),
    ]);

    let mut lines = vec![
        Line::from(i18n::tr("apps.install.path_label")),
        input,
        Line::from(""),
        Line::from(Span::styled(
            i18n::tr("apps.install.hint"),
            Style::default().fg(Color::Gray),
        )),
    ];

    if let Some(result) = &app.apk_result {
        let result_style = if result.is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(result.text.clone(), result_style)));
        lines.push(Line::from(Span::styled(
            i18n::tr("apps.install.dismiss_hint"),
            Style::default().fg(Color::Gray),
        )));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn tail_chars(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let tail: String = value.chars().skip(count - keep).collect();
    format!("…{tail}")
}

fn render_confirm(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let has_critical = app
        .selected
        .iter()
        .any(|p| matches!(app.kind(p), crate::safety::AppKind::Critical));
    let text = if has_critical && !app.force_mode {
        rust_i18n::t!("confirm.critical", count = app.selected.len()).to_string()
    } else {
        rust_i18n::t!("confirm.uninstall", count = app.selected.len()).to_string()
    };
    let para_style = if has_critical && !app.force_mode {
        Style::default().fg(Color::Red)
    } else {
        Theme::normal()
    };
    let para = Paragraph::new(text)
        .block(menu_block(i18n::tr("confirm.title")))
        .style(para_style);
    f.render_widget(para, area);
}

fn render_device_info(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let title = i18n::tr("info.title");
    if app.device_info_loading {
        let para = Paragraph::new(i18n::tr("info.loading")).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    let para = Paragraph::new(app.device_info_lines.clone())
        .block(menu_block(title))
        .scroll((app.device_info_scroll, 0))
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn render_device_control(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    if app.device_control_pending.is_some() {
        let para = Paragraph::new(i18n::tr("device.control.confirm"))
            .block(menu_block(i18n::tr("device.control.title")));
        f.render_widget(para, area);
        return;
    }
    let items = list_items(&app.device_control_keys);
    let mut state = app.device_control_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("device.control.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.device_control_state = state;
}

fn render_network(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3)])
        .split(area);

    // Info area
    let mut lines: Vec<Line> = Vec::new();
    if app.network_loading {
        lines.push(Line::from(i18n::tr("info.loading")));
    } else {
        let ip = app
            .network_ip
            .clone()
            .unwrap_or_else(|| i18n::tr("network.status.no_ip"));
        lines.push(Line::from(format!(
            "{}: {}",
            i18n::tr("network.label.ip"),
            ip
        )));
        lines.push(Line::from(format!(
            "{}: {}",
            i18n::tr("network.label.ssid"),
            app.network_ssid
        )));
        lines.push(Line::from(format!(
            "{}: {}",
            i18n::tr("network.label.wifi"),
            app.network_wifi_state
        )));
        let endpoint = app
            .network_endpoint
            .clone()
            .unwrap_or_else(|| "-".to_string());
        lines.push(Line::from(format!(
            "{}: {}",
            i18n::tr("network.label.endpoint"),
            endpoint
        )));
        let subnet = crate::config::Config::load().wifi_subnet();
        lines.push(Line::from(format!(
            "{}: {}",
            i18n::tr("network.label.subnet"),
            subnet
        )));
        if let Some(result) = &app.network_result {
            let style = if result.is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };
            lines.push(Line::from(Span::styled(result.text.clone(), style)));
        }
    }
    let info = Paragraph::new(lines).block(menu_block(i18n::tr("network.title")));
    f.render_widget(info, chunks[0]);

    // Content area
    if app.wifi_subnet_input_mode {
        let input_lines = vec![
            Line::from(i18n::tr("network.subnet.label")),
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan)),
                Span::styled(app.wifi_subnet_input.clone(), Theme::normal()),
                Span::styled("_", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(Span::styled(
                i18n::tr("network.subnet.hint"),
                Style::default().fg(Color::Gray),
            )),
        ];
        let para = Paragraph::new(input_lines).block(menu_block(i18n::tr("network.title")));
        f.render_widget(para, chunks[1]);
    } else if app.network_input_mode {
        let input_lines = vec![
            Line::from(i18n::tr("network.input.label")),
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan)),
                Span::styled(app.network_input.clone(), Theme::normal()),
                Span::styled("_", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(Span::styled(
                i18n::tr("network.input.hint"),
                Style::default().fg(Color::Gray),
            )),
        ];
        let para = Paragraph::new(input_lines).block(menu_block(i18n::tr("network.title")));
        f.render_widget(para, chunks[1]);
    } else if app.network_pending.is_some() {
        let text = match app.network_pending {
            Some(NetworkAction::Enable) => i18n::tr("network.confirm.enable"),
            Some(NetworkAction::Connect) => i18n::tr("network.confirm.connect"),
            Some(NetworkAction::Disconnect) => i18n::tr("network.confirm.disconnect"),
            Some(NetworkAction::Disable) => i18n::tr("network.confirm.disable"),
            None => String::new(),
        };
        let para = Paragraph::new(text).block(menu_block(i18n::tr("network.title")));
        f.render_widget(para, chunks[1]);
    } else {
        let items = list_items(&app.network_menu_keys);
        let mut state = app.network_menu_state;
        let list = List::new(items)
            .block(menu_block(i18n::tr("network.title")))
            .highlight_style(Theme::selected())
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[1], &mut state);
        app.network_menu_state = state;
    }
}

fn render_wifi_discover(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let mut title = i18n::tr("wifi.discover.title").to_string();
    if !app.wifi_scanning {
        if let Some(at) = app.wifi_scan_last_at {
            title = format!("{} (last scan {}s ago)", title, at.elapsed().as_secs());
        }
    }
    if app.wifi_scanning {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = if let Some(start) = app.wifi_scan_start {
            let elapsed = start.elapsed().as_millis();
            frames[(elapsed / 100) as usize % frames.len()]
        } else {
            frames[0]
        };
        let progress = app.wifi_scan_progress;
        let total = 254;
        let text = format!(
            "{} {} {}/{}",
            frame,
            i18n::tr("wifi.discover.scanning"),
            progress,
            total
        );
        let para = Paragraph::new(text).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    if app.wifi_discovered.is_empty() && app.wifi_rejected.is_empty() {
        let text = format!(
            "{}\n{}",
            i18n::tr("wifi.discover.empty"),
            i18n::tr("wifi.discover.empty_hint")
        );
        let para = Paragraph::new(text).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();
    for e in &app.wifi_discovered {
        items.push(ListItem::new(format!("✓ {}", e)).style(Style::default().fg(Color::Green)));
    }
    if app.wifi_show_rejected {
        for e in &app.wifi_rejected {
            items.push(ListItem::new(format!("✗ {}", e)).style(Style::default().fg(Color::Red)));
        }
    } else if !app.wifi_rejected.is_empty() {
        items.push(
            ListItem::new(format!(
                "… {} rejected (press 'a' to show)",
                app.wifi_rejected.len()
            ))
            .style(Style::default().fg(Color::DarkGray)),
        );
    }

    let mut state = app.wifi_discover_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.wifi_discover_state = state;
}

fn render_wifi_connect(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = menu_block(i18n::tr("wifi.connect.title"));
    let inner_width = block.inner(area).width as usize;
    let max_width = inner_width.saturating_sub(3);
    let visible = tail_chars(&app.wifi_connect_input, max_width);

    let input = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(visible, Theme::normal()),
        Span::styled("_", Style::default().fg(Color::Cyan)),
    ]);

    let mut lines = vec![
        Line::from(i18n::tr("wifi.connect.label")),
        input,
        Line::from(""),
        Line::from(Span::styled(
            i18n::tr("wifi.connect.hint"),
            Style::default().fg(Color::Gray),
        )),
    ];

    if let Some(result) = &app.wifi_connect_result {
        let style = if result.is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(result.text.clone(), style)));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_file_browser(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let title = format!("Files: {}", app.file_browser_path);
    if app.file_browser_loading {
        let para = Paragraph::new(i18n::tr("files.loading")).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    if app.file_browser_entries.is_empty() {
        let para = Paragraph::new(i18n::tr("files.empty")).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    let filter = app.file_browser_filter.to_lowercase();
    let mut items: Vec<ListItem> = Vec::new();
    for e in &app.file_browser_entries {
        if !filter.is_empty() && !e.name.to_lowercase().contains(&filter) {
            continue;
        }
        let icon = if e.is_dir { "📁" } else { "📄" };
        let size = if e.is_dir {
            String::new()
        } else {
            format!("  {} B", e.size)
        };
        items.push(ListItem::new(format!("{} {}{}", icon, e.name, size)));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);
    let mut state = app.file_browser_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[0], &mut state);
    app.file_browser_state = state;
    let help = "↑↓ navegar  ← subir  → entrar  d descargar  c copiar  m mover  v pegar  x borrar  r refrescar  / filtrar  Esc salir";
    let para = Paragraph::new(help).block(menu_block("Help".to_string()));
    f.render_widget(para, chunks[1]);
}

fn render_storage_menu(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items = list_items(&app.storage_menu_keys);
    let mut state = app.storage_menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("storage.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.storage_menu_state = state;
}

fn render_storage_usage(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    if app.storage_usage.is_empty() {
        let para = Paragraph::new(i18n::tr("storage.usage.empty"))
            .block(menu_block(i18n::tr("storage.usage.title")));
        f.render_widget(para, area);
        return;
    }
    let mut items: Vec<ListItem> = Vec::new();
    for (path, size) in &app.storage_usage {
        let mb = *size as f64 / 1_048_576.0;
        items.push(ListItem::new(format!("{}  {:.1} MB", path, mb)));
    }
    let list = List::new(items).block(menu_block(i18n::tr("storage.usage.title")));
    f.render_widget(list, area);
}

fn render_storage_volumes(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    if app.storage_volumes.is_empty() {
        let para = Paragraph::new(i18n::tr("storage.volumes.empty"))
            .block(menu_block(i18n::tr("storage.volumes.title")));
        f.render_widget(para, area);
        return;
    }
    let items: Vec<ListItem> = app
        .storage_volumes
        .iter()
        .map(|v| ListItem::new(v.clone()))
        .collect();
    let list = List::new(items).block(menu_block(i18n::tr("storage.volumes.title")));
    f.render_widget(list, area);
}
