use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::config::Lang;
use crate::i18n;
use crate::scripts::{BUILTIN_SCRIPTS, EL_NEGRO_SAMUEL, ScriptFolder};
use crate::theme::Theme;
use crate::tui::app::{App, AppAction, AppView, NetworkAction, Screen};

pub fn render(f: &mut Frame, app: &mut App) {
    if app.last_screen != Some(app.screen) {
        if app.last_screen == Some(Screen::AppList) && app.status == i18n::tr("apps.status.help") {
            app.status.clear();
        }
        app.last_screen = Some(app.screen);
    }
    let area = f.area();
    // AppList gestiona su propio footer, por lo que no reservamos el área
    // inferior de estado (evita la franja en blanco del tamaño del hint).
    let status_reserved = !app.status.is_empty() && app.screen != Screen::AppList;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if status_reserved {
            vec![Constraint::Min(3), Constraint::Length(3)]
        } else {
            vec![Constraint::Min(3)]
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

    if status_reserved {
        let status = Paragraph::new(Line::from(app.status.clone()))
            .style(Theme::muted())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Theme::border()),
            );
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
            let label = if app.scripts.script_running && *script == EL_NEGRO_SAMUEL {
                format!("{script} ({})", i18n::tr("scripts.running"))
            } else {
                (*script).to_string()
            };
            ListItem::new(label)
        })
        .collect();
    let mut state = app.scripts.scripts_menu_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.scripts.scripts_menu_state = state;
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
            let mark = if app.scripts.script_selected.contains(folder) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{mark} {}", folder.label()))
        })
        .collect();
    let mut state = app.scripts.script_folder_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.scripts.script_folder_state = state;
}

fn render_logcat(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let serial = app.serial.as_deref().unwrap_or("-");
    let title = rust_i18n::t!("logcat.title", serial = serial).to_string();
    let items: Vec<ListItem> = app
        .logcat
        .logcat_lines
        .iter()
        .map(|line| ListItem::new(line.clone()))
        .collect();
    let mut state = app.logcat.logcat_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.logcat.logcat_state = state;
}

fn render_processes(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let serial = app.serial.as_deref().unwrap_or("-");
    let title = rust_i18n::t!("processes.title", serial = serial).to_string();
    if app.processes.processes_loading {
        let para = Paragraph::new(loading_indicator(&i18n::tr("processes.status.loading"))).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    let block = menu_block(title);
    let inner = block.inner(area);
    f.render_widget(block, area);
    if let Some(process) = &app.processes.process_pending {
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
        app.processes.process_state.select(None);
    } else if let Some(selected) = app.processes.process_state.selected() {
        if selected >= filtered_len {
            app.processes.process_state.select(Some(filtered_len - 1));
        }
    } else {
        app.processes.process_state.select(Some(0));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);
    let filter_line = if app.processes.process_filter.is_empty() {
        rust_i18n::t!("processes.status.loaded", count = filtered_len).to_string()
    } else {
        format!("{} ({})", app.processes.process_filter, filtered_len)
    };
    f.render_widget(Paragraph::new(filter_line).style(Theme::muted()), chunks[0]);
    let header = format!(
        "{:<8} {:<16} {}",
        i18n::tr("processes.header.pid"),
        i18n::tr("processes.header.user"),
        i18n::tr("processes.header.name")
    );
    f.render_widget(Paragraph::new(header).style(Theme::muted()), chunks[1]);
    let items: Vec<ListItem> = filtered_indices
        .iter()
        .filter_map(|index| app.processes.processes.get(*index))
        .map(|process| {
            ListItem::new(format!(
                "{:<8} {:<16} {}",
                process.pid, process.user, process.name
            ))
        })
        .collect();
    let mut state = app.processes.process_state;
    let list = List::new(items)
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[2], &mut state);
    app.processes.process_state = state;
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
        Span::styled("> ", Theme::accent()),
        Span::styled(visible_path, Theme::normal()),
        Span::styled("_", Theme::accent()),
    ]);

    let lines = vec![
        Line::from(i18n::tr("settings.output_path")),
        input,
        Line::from(""),
        Line::from(Span::styled(
            i18n::tr("settings.output_path.hint"),
            Theme::muted(),
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
    let items = list_items(&app.apps.apps_menu_keys);
    let mut state = app.apps.apps_menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("apps.menu.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.apps.apps_menu_state = state;
}

fn render_app_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = {
        let filtered: Vec<String> = app.current_list().clone();
        filtered
            .iter()
            .map(|p| {
                let mark = if app.apps.selected.contains(p) {
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
                    crate::safety::AppKind::Critical => {
                        Theme::danger().add_modifier(ratatui::style::Modifier::BOLD)
                    }
                    crate::safety::AppKind::System => Theme::warning(),
                    crate::safety::AppKind::DeviceAdmin => Theme::admin(),
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

    let view_name = match app.apps.app_view {
        AppView::Installed => i18n::tr("apps.view.installed"),
        AppView::Available => i18n::tr("apps.view.available"),
    };
    let action_name = match app.apps.app_action {
        AppAction::Uninstall => i18n::tr("apps.action.uninstall"),
        AppAction::Reinstall => i18n::tr("apps.action.reinstall"),
        AppAction::Enable => i18n::tr("apps.action.enable"),
        AppAction::ClearCache => i18n::tr("storage.cache.title"),
        AppAction::ForceStop => i18n::tr("apps.action.force_stop"),
        AppAction::ClearData => i18n::tr("apps.action.clear_data"),
    };
    // Uninstall view shows all installed packages (system apps included, marked).
    let uninstall_all =
        app.apps.app_action == AppAction::Uninstall && app.apps.app_view == AppView::Installed;
    let core = if uninstall_all {
        i18n::tr("apps.core.uninstall_all")
    } else {
        format!("{action_name} - {view_name}")
    };
    let title = if app.apps.force_mode {
        if app.apps.filter.is_empty() && !app.apps.filter_mode {
            rust_i18n::t!("apps.title.force", core = core).to_string()
        } else {
            let filter = if app.apps.filter_mode {
                format!("{}_", app.apps.filter)
            } else {
                app.apps.filter.clone()
            };
            rust_i18n::t!("apps.title.force_filter", core = core, filter = filter).to_string()
        }
    } else if app.apps.filter.is_empty() && !app.apps.filter_mode {
        rust_i18n::t!("apps.title.normal", core = core).to_string()
    } else {
        let filter = if app.apps.filter_mode {
            format!("{}_", app.apps.filter)
        } else {
            app.apps.filter.clone()
        };
        rust_i18n::t!("apps.title.filter", core = core, filter = filter).to_string()
    };

    if !app.apps.app_cache_loaded {
        let para = Paragraph::new(loading_indicator(&i18n::tr("files.loading"))).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }

    let hints_text = if app.status.is_empty() {
        format!(
            "Sort: {} | Space select | Enter run | Tab sort | / filter | Esc back",
            app.apps.sort_mode.label()
        )
    } else {
        app.status.clone()
    };
    let inner_width = area.width.saturating_sub(2) as usize;
    let inner_width = if inner_width == 0 { 1 } else { inner_width };
    let wrapped_lines = (hints_text.len() + inner_width - 1) / inner_width;
    let wrapped_lines = wrapped_lines.max(1).min(4);
    let footer_height = (wrapped_lines + 2) as u16;

    let mut state = app.apps.app_state;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(footer_height)])
        .split(area);
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[0], &mut state);
    app.apps.app_state = state;

    let footer = Paragraph::new(hints_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Info")
                .border_style(Theme::border()),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(footer, chunks[1]);
}

fn render_install_apk(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = menu_block(i18n::tr("apps.install.title"));
    let inner_width = block.inner(area).width as usize;
    let max_path_width = inner_width.saturating_sub(3);
    let visible_path = tail_chars(&app.apps.apk_input, max_path_width);

    let input = Line::from(vec![
        Span::styled("> ", Theme::accent()),
        Span::styled(visible_path, Theme::normal()),
        Span::styled("_", Theme::accent()),
    ]);

    let mut lines = vec![
        Line::from(i18n::tr("apps.install.path_label")),
        input,
        Line::from(""),
        Line::from(Span::styled(i18n::tr("apps.install.hint"), Theme::muted())),
    ];

    if let Some(result) = &app.apps.apk_result {
        let result_style = if result.is_error {
            Theme::danger()
        } else {
            Theme::success()
        };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(result.text.clone(), result_style)));
        lines.push(Line::from(Span::styled(
            i18n::tr("apps.install.dismiss_hint"),
            Theme::muted(),
        )));
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn loading_indicator(base: &str) -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let dots = ((millis / 400) % 4) as usize;
    let base = base.trim_end_matches('.');
    format!("{}{}", base, ".".repeat(dots + 1))
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
        .apps
        .selected
        .iter()
        .any(|p| matches!(app.kind(p), crate::safety::AppKind::Critical));
    let text = if has_critical && !app.apps.force_mode {
        rust_i18n::t!("confirm.critical", count = app.apps.selected.len()).to_string()
    } else {
        rust_i18n::t!("confirm.uninstall", count = app.apps.selected.len()).to_string()
    };
    let para_style = if has_critical && !app.apps.force_mode {
        Theme::danger()
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
    if app.device_info.device_info_loading {
        let para = Paragraph::new(loading_indicator(&i18n::tr("info.loading"))).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    let para = Paragraph::new(app.device_info.device_info_lines.clone())
        .block(menu_block(title))
        .scroll((app.device_info.device_info_scroll, 0))
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
    if app.network.network_loading {
        lines.push(Line::from(i18n::tr("info.loading")));
    } else {
        let ip = app
            .network
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
            app.network.network_ssid
        )));
        lines.push(Line::from(format!(
            "{}: {}",
            i18n::tr("network.label.wifi"),
            app.network.network_wifi_state
        )));
        let endpoint = app
            .network
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
        if let Some(result) = &app.network.network_result {
            let style = if result.is_error {
                Theme::danger()
            } else {
                Theme::success()
            };
            lines.push(Line::from(Span::styled(result.text.clone(), style)));
        }
    }
    let info = Paragraph::new(lines).block(menu_block(i18n::tr("network.title")));
    f.render_widget(info, chunks[0]);

    // Content area
    if app.network.wifi_subnet_input_mode {
        let input_lines = vec![
            Line::from(i18n::tr("network.subnet.label")),
            Line::from(vec![
                Span::styled("> ", Theme::accent()),
                Span::styled(app.network.wifi_subnet_input.clone(), Theme::normal()),
                Span::styled("_", Theme::accent()),
            ]),
            Line::from(Span::styled(
                i18n::tr("network.subnet.hint"),
                Theme::muted(),
            )),
        ];
        let para = Paragraph::new(input_lines).block(menu_block(i18n::tr("network.title")));
        f.render_widget(para, chunks[1]);
    } else if app.network.network_input_mode {
        let input_lines = vec![
            Line::from(i18n::tr("network.input.label")),
            Line::from(vec![
                Span::styled("> ", Theme::accent()),
                Span::styled(app.network.network_input.clone(), Theme::normal()),
                Span::styled("_", Theme::accent()),
            ]),
            Line::from(Span::styled(i18n::tr("network.input.hint"), Theme::muted())),
        ];
        let para = Paragraph::new(input_lines).block(menu_block(i18n::tr("network.title")));
        f.render_widget(para, chunks[1]);
    } else if app.network.network_pending.is_some() {
        let text = match app.network.network_pending {
            Some(NetworkAction::Enable) => i18n::tr("network.confirm.enable"),
            Some(NetworkAction::Connect) => i18n::tr("network.confirm.connect"),
            Some(NetworkAction::Disconnect) => i18n::tr("network.confirm.disconnect"),
            Some(NetworkAction::Disable) => i18n::tr("network.confirm.disable"),
            None => String::new(),
        };
        let para = Paragraph::new(text).block(menu_block(i18n::tr("network.title")));
        f.render_widget(para, chunks[1]);
    } else {
        let items = list_items(&app.network.network_menu_keys);
        let mut state = app.network.network_menu_state;
        let list = List::new(items)
            .block(menu_block(i18n::tr("network.title")))
            .highlight_style(Theme::selected())
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, chunks[1], &mut state);
        app.network.network_menu_state = state;
    }
}

fn render_wifi_discover(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let mut title = i18n::tr("wifi.discover.title").to_string();
    if !app.network.wifi_scanning
        && let Some(at) = app.network.wifi_scan_last_at
    {
        title = format!("{} (last scan {}s ago)", title, at.elapsed().as_secs());
    }
    if app.network.wifi_scanning {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = if let Some(start) = app.network.wifi_scan_start {
            let elapsed = start.elapsed().as_millis();
            frames[(elapsed / 100) as usize % frames.len()]
        } else {
            frames[0]
        };
        let progress = app.network.wifi_scan_progress;
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
    if app.network.wifi_discovered.is_empty() && app.network.wifi_rejected.is_empty() {
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
    for e in &app.network.wifi_discovered {
        items.push(ListItem::new(format!("✓ {}", e)).style(Theme::success()));
    }
    if app.network.wifi_show_rejected {
        for e in &app.network.wifi_rejected {
            items.push(ListItem::new(format!("✗ {}", e)).style(Theme::danger()));
        }
    } else if !app.network.wifi_rejected.is_empty() {
        items.push(
            ListItem::new(format!(
                "… {} rejected (press 'a' to show)",
                app.network.wifi_rejected.len()
            ))
            .style(Theme::dark_muted()),
        );
    }

    let mut state = app.network.wifi_discover_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.network.wifi_discover_state = state;
}

fn render_wifi_connect(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let block = menu_block(i18n::tr("wifi.connect.title"));
    let inner_width = block.inner(area).width as usize;
    let max_width = inner_width.saturating_sub(3);
    let visible = tail_chars(&app.network.wifi_connect_input, max_width);

    let input = Line::from(vec![
        Span::styled("> ", Theme::accent()),
        Span::styled(visible, Theme::normal()),
        Span::styled("_", Theme::accent()),
    ]);

    let mut lines = vec![
        Line::from(i18n::tr("wifi.connect.label")),
        input,
        Line::from(""),
        Line::from(Span::styled(i18n::tr("wifi.connect.hint"), Theme::muted())),
    ];

    if let Some(result) = &app.network.wifi_connect_result {
        let style = if result.is_error {
            Theme::danger()
        } else {
            Theme::success()
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
    let title = format!("Files: {}", app.storage.file_browser_path);
    if app.storage.file_browser_loading {
        let para = Paragraph::new(loading_indicator(&i18n::tr("files.loading"))).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    if app.storage.file_browser_entries.is_empty() {
        let para = Paragraph::new(i18n::tr("files.empty")).block(menu_block(title));
        f.render_widget(para, area);
        return;
    }
    let filter = app.storage.file_browser_filter.to_lowercase();
    let mut items: Vec<ListItem> = Vec::new();
    for e in &app.storage.file_browser_entries {
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
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);
    let mut state = app.storage.file_browser_state;
    let list = List::new(items)
        .block(menu_block(title))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[0], &mut state);
    app.storage.file_browser_state = state;
    let help = "↑↓ navegar  ← subir  → entrar  d descargar  c copiar  m mover  v pegar  x borrar  r refrescar  / filtrar  Esc salir";
    let para = Paragraph::new(help).block(menu_block("Help".to_string()));
    f.render_widget(para, chunks[1]);
}

fn render_storage_menu(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items = list_items(&app.storage.storage_menu_keys);
    let mut state = app.storage.storage_menu_state;
    let list = List::new(items)
        .block(menu_block(i18n::tr("storage.title")))
        .highlight_style(Theme::selected())
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, area, &mut state);
    app.storage.storage_menu_state = state;
}

fn render_storage_usage(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    if app.storage.storage_usage_loading {
        let para = Paragraph::new(loading_indicator(&i18n::tr("storage.usage.loading")))
            .block(menu_block(i18n::tr("storage.usage.title")));
        f.render_widget(para, area);
        return;
    }
    if app.storage.storage_usage.is_empty() {
        let para = Paragraph::new(i18n::tr("storage.usage.empty"))
            .block(menu_block(i18n::tr("storage.usage.title")));
        f.render_widget(para, area);
        return;
    }
    let mut items: Vec<ListItem> = Vec::new();
    for (path, size) in &app.storage.storage_usage {
        let mb = *size as f64 / 1_048_576.0;
        items.push(ListItem::new(format!("{}  {:.1} MB", path, mb)));
    }
    let list = List::new(items).block(menu_block(i18n::tr("storage.usage.title")));
    f.render_widget(list, area);
}

fn render_storage_volumes(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    if app.storage.storage_volumes_loading {
        let para = Paragraph::new(loading_indicator(&i18n::tr("storage.volumes.loading")))
            .block(menu_block(i18n::tr("storage.volumes.title")));
        f.render_widget(para, area);
        return;
    }
    if app.storage.storage_volumes.is_empty() {
        let para = Paragraph::new(i18n::tr("storage.volumes.empty"))
            .block(menu_block(i18n::tr("storage.volumes.title")));
        f.render_widget(para, area);
        return;
    }
    let items: Vec<ListItem> = app
        .storage
        .storage_volumes
        .iter()
        .map(|v| ListItem::new(v.clone()))
        .collect();
    let list = List::new(items).block(menu_block(i18n::tr("storage.volumes.title")));
    f.render_widget(list, area);
}
