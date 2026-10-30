use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{ApkResult, App, AppAction, AppView, DeviceControlAction, NetworkAction, Screen};
use crate::config::Lang;
use crate::i18n;
use crate::scripts::{BUILTIN_SCRIPTS, ScriptFolder};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Global Ctrl+C quit (disabled on Shell; send to stdin instead)
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C'))
    {
        app.should_quit = true;
        return;
    }

    // Global quit (except when typing text)
    if !app.filter_mode
        && app.screen != Screen::InstallApk
        && app.screen != Screen::OutputPath
        && !(app.screen == Screen::Network && app.network_input_mode)
        && !(app.screen == Screen::Network && app.wifi_subnet_input_mode)
        && !(app.screen == Screen::WifiConnect && app.wifi_connect_input_mode)
        && key.code == KeyCode::Char('q')
    {
        app.should_quit = true;
        return;
    }

    match app.screen {
        Screen::DeviceSelect => handle_device_select(app, key),
        Screen::MainMenu => handle_main_menu(app, key),
        Screen::Scripts => handle_scripts(app, key),
        Screen::ScriptFolders => handle_script_folders(app, key),
        Screen::Logcat => handle_logcat(app, key),
        Screen::Processes => handle_processes(app, key),
        Screen::DeviceMenu => handle_device_menu(app, key),
        Screen::Settings => handle_settings(app, key),
        Screen::OutputPath => handle_output_path(app, key),
        Screen::Language => handle_language(app, key),
        Screen::AppsMenu => handle_apps_menu(app, key),
        Screen::AppList => handle_app_list(app, key),
        Screen::InstallApk => handle_install_apk(app, key),
        Screen::Confirm => handle_confirm(app, key),
        Screen::DeviceInfo => handle_device_info(app, key),
        Screen::DeviceControl => handle_device_control(app, key),
        Screen::Network => handle_network(app, key),
        Screen::WifiDiscover => handle_wifi_discover(app, key),
        Screen::WifiConnect => handle_wifi_connect(app, key),
    Screen::FileBrowser => handle_file_browser(app, key),
    Screen::StorageMenu => handle_storage_menu(app, key),
    Screen::StorageUsage => handle_storage_usage(app, key),
    Screen::StorageVolumes => handle_storage_volumes(app, key),
    }
}

pub fn handle_device_select(app: &mut App, key: KeyEvent) {
    let total = app.devices.len() + 2;
    match key.code {
        KeyCode::Up => {
            if !app.loading {
                App::move_list(&mut app.device_state, total, -1);
            }
        }
        KeyCode::Down => {
            if !app.loading {
                App::move_list(&mut app.device_state, total, 1);
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.loading = true;
            app.status = i18n::tr("device.select.loading");
        }
        KeyCode::Enter => {
            let selected = app.device_state.selected().unwrap_or(0);
            if selected < app.devices.len() {
                if app.devices.is_empty() {
                    app.loading = true;
                    app.status = i18n::tr("device.select.loading");
                    return;
                }
                let device = &app.devices[selected];
                if device.state != "device" {
                    app.status = format!(
                        "{} ({})",
                        i18n::tr("app.status.no_device"),
                        device.state
                    );
                    return;
                }
                let serial = device.serial.clone();
                app.select_device(serial);
            } else if selected == app.devices.len() {
                app.open_wifi_discover();
            } else {
                app.screen = Screen::WifiConnect;
                app.wifi_connect_input.clear();
                app.wifi_connect_result = None;
                app.wifi_connect_input_mode = true;
                app.status = i18n::tr("wifi.connect.hint");
            }
        }
        KeyCode::Esc => {
            app.screen = Screen::MainMenu;
        }
        _ => {}
    }
}

pub fn handle_main_menu(app: &mut App, key: KeyEvent) {
    let len = app.menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.menu_state.selected().unwrap_or(0);
            match i {
                0 => {
                    app.open_device_select();
                }
                1 => {
                    app.screen = Screen::Settings;
                    app.settings_menu_state.select(Some(0));
                }
                2 => app.should_quit = true,
                _ => {}
            }
        }
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}

pub fn handle_scripts(app: &mut App, key: KeyEvent) {
    let len = BUILTIN_SCRIPTS.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.scripts_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.scripts_menu_state, len, 1),
        KeyCode::Enter => app.open_script_folders(),
        KeyCode::Esc => app.screen = Screen::DeviceMenu,
        _ => {}
    }
}

pub fn handle_script_folders(app: &mut App, key: KeyEvent) {
    let len = ScriptFolder::ALL.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.script_folder_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.script_folder_state, len, 1),
        KeyCode::Char(' ') => app.toggle_script_folder(),
        KeyCode::Enter => app.start_el_negro_samuel(),
        KeyCode::Esc => app.open_scripts(),
        _ => {}
    }
}

pub fn handle_device_menu(app: &mut App, key: KeyEvent) {
    let len = app.device_menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.device_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.device_menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.device_menu_state.selected().unwrap_or(0);
            match i {
                0 => app.screen = Screen::AppsMenu,
                1 => app.open_scripts(),
                2 => app.open_logcat(),
                3 => app.open_processes(),
                4 => {
                    app.screen = Screen::DeviceInfo;
                    app.load_device_info();
                }
                5 => {
                    app.request_direct_shell();
                }
                6 => {
                    app.open_device_control();
                }
                7 => {
                    app.open_network();
                }
                8 => {
                    app.open_storage_menu();
                }
                9 => app.screen = Screen::MainMenu,
                _ => {}
            }
        }
        KeyCode::Esc => app.screen = Screen::MainMenu,
        _ => {}
    }
}

pub fn handle_logcat(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => app.logcat_move(-1),
        KeyCode::Down => app.logcat_move(1),
        KeyCode::PageUp => app.logcat_move(-10),
        KeyCode::PageDown => app.logcat_move(10),
        KeyCode::Char('p') | KeyCode::Char('P') => app.toggle_logcat_pause(),
        KeyCode::Char('c') | KeyCode::Char('C') => app.clear_logcat(),
        KeyCode::Char('e') | KeyCode::Char('E') => app.export_logcat(),
        KeyCode::Esc => {
            app.stop_logcat();
            app.screen = Screen::DeviceMenu;
            app.status = i18n::tr("logcat.status.stopped");
        }
        _ => {}
    }
}

pub fn handle_processes(app: &mut App, key: KeyEvent) {
    if app.process_pending.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                app.execute_process_kill();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.process_pending = None;
                app.status = i18n::tr("processes.hint");
            }
            _ => {}
        }
        return;
    }

    if app.process_filter_mode {
        match key.code {
            KeyCode::Esc => {
                if !app.process_filter.is_empty() {
                    app.process_filter.clear();
                } else {
                    app.process_filter_mode = false;
                }
            }
            KeyCode::Enter => {
                app.process_filter_mode = false;
            }
            KeyCode::Backspace => {
                app.process_filter.pop();
            }
            KeyCode::Char(c) => {
                app.process_filter.push(c);
            }
            _ => {}
        }
        return;
    }

    let len = app.filtered_process_indices().len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.process_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.process_state, len, 1),
        KeyCode::Char('/') => {
            app.process_filter_mode = true;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => app.load_processes(),
        KeyCode::Enter | KeyCode::Char('k') | KeyCode::Char('K') => app.request_process_kill(),
        KeyCode::Esc => {
            if !app.process_filter.is_empty() {
                app.process_filter.clear();
            } else {
                app.process_pending = None;
                app.screen = Screen::DeviceMenu;
            }
        }
        _ => {}
    }
}

pub fn handle_settings(app: &mut App, key: KeyEvent) {
    let len = app.settings_menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.settings_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.settings_menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.settings_menu_state.selected().unwrap_or(0);
            match i {
                0 => {
                    app.screen = Screen::Language;
                    let selected = match app.lang {
                        Lang::English => 0,
                        Lang::Spanish => 1,
                    };
                    app.language_menu_state.select(Some(selected));
                    app.status.clear();
                }
                1 => app.open_output_path(),
                2 => app.screen = Screen::MainMenu,
                _ => {}
            }
        }
        KeyCode::Esc => app.screen = Screen::MainMenu,
        _ => {}
    }
}

pub fn handle_language(app: &mut App, key: KeyEvent) {
    let len = app.language_menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.language_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.language_menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.language_menu_state.selected().unwrap_or(0);
            match i {
                0 => {
                    app.apply_language(Lang::English);
                    app.screen = Screen::Settings;
                }
                1 => {
                    app.apply_language(Lang::Spanish);
                    app.screen = Screen::Settings;
                }
                2 => app.screen = Screen::Settings,
                _ => {}
            }
        }
        KeyCode::Esc => app.screen = Screen::Settings,
        _ => {}
    }
}

pub fn handle_apps_menu(app: &mut App, key: KeyEvent) {
    let len = app.apps_menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.apps_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.apps_menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.apps_menu_state.selected().unwrap_or(0);
            match i {
                0 => {
                    if let Err(e) = app.ensure_safety() {
                        app.status = rust_i18n::t!("app.error", error = e).to_string();
                    }
                    app.app_action = AppAction::Uninstall;
                    app.app_view = AppView::Installed;
                    if !app.list_cache_matches(AppAction::Uninstall, AppView::Installed) {
                        app.load_packages(false);
                    }
                    app.status = i18n::tr("apps.status.help");
                    app.screen = Screen::AppList;
                }
                1 => {
                    app.apk_input.clear();
                    app.apk_result = None;
                    app.status = i18n::tr("apps.install.hint");
                    app.screen = Screen::InstallApk;
                }
                2 => {
                    if let Err(e) = app.ensure_safety() {
                        app.status = rust_i18n::t!("app.error", error = e).to_string();
                    }
                    app.app_action = AppAction::Reinstall;
                    app.app_view = AppView::Available;
                    if !app.list_cache_matches(AppAction::Reinstall, AppView::Available) {
                        app.load_available();
                    }
                    app.status = i18n::tr("apps.status.help");
                    app.screen = Screen::AppList;
                }
                3 => {
                    if let Err(e) = app.ensure_safety() {
                        app.status = rust_i18n::t!("app.error", error = e).to_string();
                    }
                    app.app_action = AppAction::Enable;
                    app.app_view = AppView::Installed;
                    if !app.list_cache_matches(AppAction::Enable, AppView::Installed) {
                        app.load_disabled_packages();
                    }
                    app.status = i18n::tr("apps.status.help");
                    app.screen = Screen::AppList;
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.screen = Screen::DeviceMenu;
        }
        _ => {}
    }
}

pub fn handle_app_list(app: &mut App, key: KeyEvent) {
    if let Err(e) = app.ensure_safety() {
        app.status = rust_i18n::t!("app.error", error = e).to_string();
    }

    if app.filter_mode {
        match key.code {
            KeyCode::Esc => {
                app.filter_mode = false;
            }
            KeyCode::Enter => {
                app.filter_mode = false;
            }
            KeyCode::Backspace => {
                app.filter.pop();
                app.invalidate_list_cache();
            }
            KeyCode::Char(c) => {
                app.filter.push(c);
                app.invalidate_list_cache();
            }
            _ => {}
        }
        return;
    }

    let filtered_len = app.current_list().len();
    match key.code {
        KeyCode::Tab => {
            use crate::tui::app::SortMode;
            app.sort_mode = match app.sort_mode {
                SortMode::NameAsc => SortMode::NameDesc,
                SortMode::NameDesc => SortMode::CriticalFirst,
                SortMode::CriticalFirst => SortMode::CriticalLast,
                SortMode::CriticalLast => SortMode::NameAsc,
            };
            app.invalidate_list_cache();
            app.status =
                rust_i18n::t!("apps.status.sort", mode = app.sort_mode.label()).to_string();
        }
        KeyCode::Right => {
            let new_view = match app.app_view {
                crate::tui::app::AppView::Installed => {
                    app.load_available();
                    crate::tui::app::AppView::Available
                }
                crate::tui::app::AppView::Available => {
                    if app.app_action == crate::tui::app::AppAction::Enable {
                        app.load_disabled_packages();
                    } else if app.app_action == crate::tui::app::AppAction::Uninstall {
                        app.load_packages(false);
                    } else {
                        app.load_packages(true);
                    }
                    crate::tui::app::AppView::Installed
                }
            };
            app.app_view = new_view;
            app.invalidate_list_cache();
        }
        KeyCode::Left => {
            let new_view = match app.app_view {
                crate::tui::app::AppView::Installed => {
                    app.load_available();
                    crate::tui::app::AppView::Available
                }
                crate::tui::app::AppView::Available => {
                    if app.app_action == crate::tui::app::AppAction::Enable {
                        app.load_disabled_packages();
                    } else if app.app_action == crate::tui::app::AppAction::Uninstall {
                        app.load_packages(false);
                    } else {
                        app.load_packages(true);
                    }
                    crate::tui::app::AppView::Installed
                }
            };
            app.app_view = new_view;
            app.invalidate_list_cache();
        }
        KeyCode::Up => App::move_list(&mut app.app_state, filtered_len, -1),
        KeyCode::Down => App::move_list(&mut app.app_state, filtered_len, 1),
        KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('/') => {
            app.filter_mode = true;
        }
        KeyCode::Char('r') => match app.app_view {
            crate::tui::app::AppView::Installed => {
                if app.app_action == crate::tui::app::AppAction::Enable {
                    app.load_disabled_packages();
                } else if app.app_action == crate::tui::app::AppAction::Uninstall {
                    app.load_packages(false);
                } else {
                    app.load_packages(true);
                }
            }
            crate::tui::app::AppView::Available => app.load_available(),
        },
        KeyCode::Char('F') => {
            app.force_mode = !app.force_mode;
            if app.force_mode {
                app.status = i18n::tr("apps.status.force_on");
            } else {
                app.status = i18n::tr("apps.status.force_off");
            }
        }
        KeyCode::Char('I') => {
            app.apk_input.clear();
            app.apk_result = None;
            app.status = i18n::tr("apps.install.hint");
            app.screen = Screen::InstallApk;
        }
        KeyCode::Enter => {
            if app.selected.is_empty() {
                app.status = i18n::tr("apps.status.select_with_space");
            } else {
                match app.app_action {
                    crate::tui::app::AppAction::Uninstall => {
                        app.screen = Screen::Confirm;
                    }
                    crate::tui::app::AppAction::Reinstall => {
                        app.install_selected();
                    }
                    crate::tui::app::AppAction::Enable => {
                        app.enable_selected();
                    }
                    crate::tui::app::AppAction::ClearCache => {
                        app.clear_cache_selected();
                    }
                }
            }
        }
        KeyCode::Esc => {
            if app.app_action == crate::tui::app::AppAction::ClearCache {
                app.screen = Screen::StorageMenu;
            } else {
                app.screen = Screen::AppsMenu;
            }
        }
        _ => {}
    }
}

pub fn handle_install_apk(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.apk_input.clear();
            app.apk_result = None;
            app.status.clear();
            app.screen = Screen::AppsMenu;
        }
        KeyCode::Enter => {
            if app.apk_result.is_some() {
                app.apk_input.clear();
                app.apk_result = None;
                return;
            }

            let path = app.apk_input.trim().to_string();
            if let Err(e) = crate::validate::apk_path(&path) {
                app.apk_result = Some(ApkResult {
                    text: e.to_string(),
                    is_error: true,
                });
                return;
            }

            let Some(serial) = app.serial.clone() else {
                app.apk_result = Some(ApkResult {
                    text: i18n::tr("apps.install.error.no_device"),
                    is_error: true,
                });
                return;
            };

            app.apk_result = Some(
                match crate::package::install_apk(&app.adb, &serial, &path) {
                    Ok(result) => ApkResult {
                        text: rust_i18n::t!("apps.install.success", result = result).to_string(),
                        is_error: false,
                    },
                    Err(error) => ApkResult {
                        text: rust_i18n::t!("apps.install.failure", error = error).to_string(),
                        is_error: true,
                    },
                },
            );
        }
        KeyCode::Backspace => {
            if app.apk_result.is_some() {
                app.apk_result = None;
                app.apk_input.clear();
            } else {
                app.apk_input.pop();
            }
        }
        KeyCode::Char(c) => {
            if app.apk_result.is_some() {
                app.apk_result = None;
                app.apk_input.clear();
            }
            app.apk_input.push(c);
        }
        _ => {}
    }
}

pub fn handle_confirm(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.uninstall_selected();
            app.screen = Screen::AppList;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.screen = Screen::AppList;
        }
        _ => {}
    }
}

pub fn handle_device_info(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => app.scroll_device_info(-1),
        KeyCode::Down => app.scroll_device_info(1),
        KeyCode::Esc => {
            app.screen = Screen::DeviceMenu;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.load_device_info();
        }
        _ => {}
    }
}

pub fn handle_device_control(app: &mut App, key: KeyEvent) {
    if app.device_control_pending.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                let action = app.device_control_pending.unwrap();
                app.execute_device_control(action);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.device_control_pending = None;
            }
            _ => {}
        }
        return;
    }
    let len = app.device_control_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.device_control_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.device_control_state, len, 1),
        KeyCode::Enter => {
            let i = app.device_control_state.selected().unwrap_or(0);
            match i {
                0 => app.device_control_pending = Some(DeviceControlAction::Reboot),
                1 => app.device_control_pending = Some(DeviceControlAction::Recovery),
                2 => app.device_control_pending = Some(DeviceControlAction::Bootloader),
                3 => app.device_control_pending = Some(DeviceControlAction::PowerOff),
                4 => app.screen = Screen::DeviceMenu,
                _ => {}
            }
        }
        KeyCode::Esc => app.screen = Screen::DeviceMenu,
        _ => {}
    }
}

pub fn handle_network(app: &mut App, key: KeyEvent) {
    if app.wifi_subnet_input_mode {
        match key.code {
            KeyCode::Char(c) => app.wifi_subnet_input.push(c),
            KeyCode::Backspace => {
                app.wifi_subnet_input.pop();
            }
            KeyCode::Enter => {
                app.apply_wifi_subnet();
            }
            KeyCode::Esc => {
                app.wifi_subnet_input_mode = false;
                app.wifi_subnet_input.clear();
            }
            _ => {}
        }
        return;
    }

    if app.network_input_mode {
        match key.code {
            KeyCode::Char(c) => app.network_input.push(c),
            KeyCode::Backspace => {
                app.network_input.pop();
            }
            KeyCode::Enter => {
                let input = app.network_input.trim().to_string();
                if input.is_empty() {
                    app.status = i18n::tr("network.status.enter_ip");
                } else {
                    let endpoint = crate::adb::normalize_network_endpoint(&input);
                    app.network_pending = Some(NetworkAction::Connect);
                    app.network_pending_endpoint = Some(endpoint);
                    app.network_input_mode = false;
                    app.network_input.clear();
                }
            }
            KeyCode::Esc => {
                app.network_input_mode = false;
                app.network_input.clear();
            }
            _ => {}
        }
        return;
    }

    if app.network_pending.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                let action = app.network_pending.unwrap();
                app.execute_network_action(action);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.network_pending = None;
                app.network_pending_endpoint = None;
            }
            _ => {}
        }
        return;
    }

    let len = app.network_menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.network_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.network_menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.network_menu_state.selected().unwrap_or(0);
            match i {
                0 => app.load_network_info(),
                1 => {
                    if app.network_endpoint.is_some() {
                        if let Some(ep) = app.network_endpoint.clone() {
                            app.network_pending = Some(NetworkAction::Disconnect);
                            app.network_pending_endpoint = Some(ep);
                        }
                    } else {
                        app.network_pending = Some(NetworkAction::Enable);
                    }
                }
                2 => {
                    if app.network_endpoint.is_some() {
                        // Disable Wi-Fi ADB (switch to USB mode)
                        if let Some(ep) = app.network_endpoint.clone() {
                            app.network_pending = Some(NetworkAction::Disable);
                            app.network_pending_endpoint = Some(ep);
                        }
                    } else if let Some(ip) = app.network_ip.clone() {
                        app.network_pending = Some(NetworkAction::Connect);
                        app.network_pending_endpoint =
                            Some(crate::adb::normalize_network_endpoint(&ip));
                    } else {
                        app.network_input_mode = true;
                        app.network_input.clear();
                        app.status = i18n::tr("network.status.enter_ip");
                    }
                }
                3 => {
                    // Set subnet (both states)
                    app.open_wifi_subnet_input();
                }
                4 => {
                    // Back (both states)
                    app.screen = Screen::DeviceMenu;
                }
                _ => {}
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => app.load_network_info(),
        KeyCode::Esc => app.screen = Screen::DeviceMenu,
        _ => {}
    }
}

pub fn handle_wifi_discover(app: &mut App, key: KeyEvent) {
    if app.wifi_scanning {
        if key.code == KeyCode::Esc {
            // dejar que el escaneo termine en background; volver a device select
            app.open_device_select();
        }
        return;
    }

    let len = if app.wifi_show_rejected {
        app.wifi_discovered.len() + app.wifi_rejected.len()
    } else {
        app.wifi_discovered.len()
    };
    match key.code {
        KeyCode::Up => {
            if len > 0 {
                App::move_list(&mut app.wifi_discover_state, len, -1);
            }
        }
        KeyCode::Down => {
            if len > 0 {
                App::move_list(&mut app.wifi_discover_state, len, 1);
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.wifi_show_rejected = !app.wifi_show_rejected;
            let new_len = if app.wifi_show_rejected {
                app.wifi_discovered.len() + app.wifi_rejected.len()
            } else {
                app.wifi_discovered.len()
            };
            if new_len == 0 {
                app.wifi_discover_state.select(None);
            } else {
                let sel = app.wifi_discover_state.selected().unwrap_or(0).min(new_len - 1);
                app.wifi_discover_state.select(Some(sel));
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.start_wifi_scan();
        }
        KeyCode::Enter => {
            let Some(i) = app.wifi_discover_state.selected() else {
                return;
            };
            if i >= app.wifi_discovered.len() {
                return;
            }
            let endpoint = app.wifi_discovered[i].clone();
            if app.devices.iter().any(|d| d.serial == endpoint) {
                app.select_device(endpoint);
                return;
            }
            match crate::adb::wifi_connect(&app.adb, &endpoint) {
                Ok(_) => {
                    app.loading = true;
                    app.status = i18n::tr("device.select.loading");
                    // Reintentar para que adb devices registre el endpoint.
                    for _ in 0..3 {
                        app.load_devices();
                        if app.devices.iter().any(|d| d.serial == endpoint) {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                    if let Some(dev) = app.devices.iter().find(|d| d.serial == endpoint) {
                        let state = dev.state.clone();
                        if state == "device" {
                            app.select_device(endpoint);
                        } else {
                            if app.serial.is_none() {
                                app.open_device_select();
                            }
                            app.status = format!(
                                "{} ({})",
                                i18n::tr("wifi.status.connected"),
                                state
                            );
                        }
                    } else if app.serial.is_none() {
                        app.open_device_select();
                        app.status = format!(
                            "{} (adb devices no lo lista aún; usa 'r' para refrescar)",
                            i18n::tr("wifi.status.connected")
                        );
                    } else {
                        app.status = i18n::tr("wifi.status.connected");
                    }
                    app.loading = false;
                }
                Err(e) => {
                    app.status = rust_i18n::t!("app.error", error = e).to_string();
                }
            }
        }
        KeyCode::Esc => {
            app.loading = true;
            app.load_devices();
            app.loading = false;
            app.open_device_select();
        }
        _ => {}
    }
}

pub fn handle_wifi_connect(app: &mut App, key: KeyEvent) {
    if app.wifi_connect_input_mode {
        match key.code {
            KeyCode::Char(c) => {
                app.wifi_connect_input.push(c);
            }
            KeyCode::Backspace => {
                app.wifi_connect_input.pop();
            }
            KeyCode::Enter => {
                let input = app.wifi_connect_input.trim().to_string();
                if input.is_empty() {
                    app.wifi_connect_result = Some(ApkResult {
                        text: i18n::tr("wifi.connect.empty"),
                        is_error: true,
                    });
                    app.status = i18n::tr("wifi.connect.empty");
                    return;
                }
                let endpoint = crate::adb::normalize_network_endpoint(&input);
                if endpoint.is_empty() {
                    app.wifi_connect_result = Some(ApkResult {
                        text: i18n::tr("wifi.connect.invalid"),
                        is_error: true,
                    });
                    app.status = i18n::tr("wifi.connect.invalid");
                    return;
                }
                match crate::adb::wifi_connect(&app.adb, &endpoint) {
                    Ok(_) => {
                        app.loading = true;
                        app.status = i18n::tr("device.select.loading");
                        for _ in 0..3 {
                            app.load_devices();
                            if app.devices.iter().any(|d| d.serial == endpoint) {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        app.loading = false;
                        if let Some(dev) = app.devices.iter().find(|d| d.serial == endpoint) {
                            let state = dev.state.clone();
                            if state == "device" {
                                app.select_device(endpoint.clone());
                            } else if app.serial.is_none() {
                                app.open_device_select();
                                app.status = format!(
                                    "{} ({})",
                                    i18n::tr("wifi.status.connected"),
                                    state
                                );
                            } else {
                                app.status = format!(
                                    "{} ({})",
                                    i18n::tr("wifi.status.connected"),
                                    state
                                );
                            }
                        } else if app.serial.is_none() {
                            app.open_device_select();
                            app.status = format!(
                                "{} (adb devices no lo lista aún; usa 'r' para refrescar)",
                                i18n::tr("wifi.status.connected")
                            );
                        } else {
                            app.status = i18n::tr("wifi.status.connected");
                        }
                        app.wifi_connect_result = Some(ApkResult {
                            text: i18n::tr("wifi.status.connected"),
                            is_error: false,
                        });
                        app.status = i18n::tr("wifi.status.connected");
                    }
                    Err(e) => {
                        app.wifi_connect_result = Some(ApkResult {
                            text: rust_i18n::t!("app.error", error = e).to_string(),
                            is_error: true,
                        });
                        app.status = rust_i18n::t!("app.error", error = e).to_string();
                    }
                }
            }
            KeyCode::Esc => {
                app.wifi_connect_input.clear();
                app.wifi_connect_result = None;
                app.wifi_connect_input_mode = false;
                app.open_device_select();
            }
            _ => {}
        }
        return;
    }

    if key.code == KeyCode::Esc {
        app.wifi_connect_input.clear();
        app.wifi_connect_result = None;
        app.wifi_connect_input_mode = false;
        app.open_device_select();
    }
}

pub fn handle_output_path(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.output_path_input_mode = false;
            app.screen = Screen::Settings;
        }
        KeyCode::Enter => {
            app.apply_output_path();
        }
        KeyCode::Backspace => {
            app.output_path_input.pop();
        }
        KeyCode::Char(c) => {
            app.output_path_input.push(c);
        }
        _ => {}
    }
}

pub fn handle_file_browser(app: &mut App, key: KeyEvent) {
    if app.file_browser_input_mode {
        match key.code {
            KeyCode::Char(c) => app.file_browser_input.push(c),
            KeyCode::Backspace => {
                app.file_browser_input.pop();
            }
            KeyCode::Enter => {
                app.file_browser_input_mode = false;
            }
            KeyCode::Esc => {
                app.file_browser_input.clear();
                app.file_browser_input_mode = false;
            }
            _ => {}
        }
        return;
    }
    if app.file_browser_pending_delete.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let Some(path) = app.file_browser_pending_delete.take() {
                    if crate::adb::validate_remote_path(&path) {
                        let _ = crate::adb::remove_remote(&app.adb, app.serial.as_deref().unwrap_or(""), &path);
                    }
                    app.load_file_browser();
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.file_browser_pending_delete = None;
                app.status = i18n::tr("files.hint").to_string();
            }
            _ => {}
        }
        return;
    }
    let filtered: Vec<usize> = app
        .file_browser_entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            app.file_browser_filter.is_empty()
                || e.name
                    .to_lowercase()
                    .contains(&app.file_browser_filter.to_lowercase())
        })
        .map(|(i, _)| i)
        .collect();
    let len = filtered.len();
    match key.code {
        KeyCode::Up => {
            if len > 0 {
                App::move_list(&mut app.file_browser_state, len, -1);
            }
        }
        KeyCode::Down => {
            if len > 0 {
                App::move_list(&mut app.file_browser_state, len, 1);
            }
        }
        KeyCode::Enter => {
            let Some(sel) = app.file_browser_state.selected() else {
                return;
            };
            if sel >= filtered.len() {
                return;
            }
            let idx = filtered[sel];
            let entry = &app.file_browser_entries[idx];
            if entry.is_dir {
                let new_path = if app.file_browser_path.ends_with('/') {
                    format!("{}{}", app.file_browser_path, entry.name)
                } else {
                    format!("{}/{}", app.file_browser_path, entry.name)
                };
                app.file_browser_path = new_path;
                app.load_file_browser();
            }
        }
        KeyCode::Backspace => {
            let p = app.file_browser_path.trim_end_matches('/');
            if let Some(pos) = p.rfind('/') {
                if pos == 0 {
                    app.file_browser_path = "/".to_string();
                } else {
                    app.file_browser_path = p[..pos].to_string();
                }
                if app.file_browser_path.is_empty() {
                    app.file_browser_path = "/".to_string();
                }
                app.load_file_browser();
            } else {
                app.screen = Screen::DeviceMenu;
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            let Some(sel) = app.file_browser_state.selected() else {
                return;
            };
            if sel >= filtered.len() {
                return;
            }
            let idx = filtered[sel];
            let entry = &app.file_browser_entries[idx];
            if entry.is_dir {
                return;
            }
            let remote = format!("{}/{}", app.file_browser_path.trim_end_matches('/'), entry.name);
            let local = format!("{}/{}", std::env::var("HOME").unwrap_or_default(), entry.name);
            match crate::adb::pull_remote(&app.adb, app.serial.as_deref().unwrap_or(""), &remote, &local) {
                Ok(_) => app.status = i18n::tr("files.pull_done").to_string(),
                Err(e) => app.status = rust_i18n::t!("app.error", error = e).to_string(),
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            let Some(sel) = app.file_browser_state.selected() else {
                return;
            };
            if sel >= filtered.len() {
                return;
            }
            let idx = filtered[sel];
            let entry = &app.file_browser_entries[idx];
            let path = format!("{}/{}", app.file_browser_path.trim_end_matches('/'), entry.name);
            if crate::adb::validate_remote_path(&path) {
                app.file_browser_pending_delete = Some(path);
                app.status = i18n::tr("files.delete_confirm").to_string();
            } else {
                app.status = i18n::tr("files.protected").to_string();
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.load_file_browser();
        }
        KeyCode::Char('/') => {
            app.file_browser_input_mode = true;
            app.file_browser_input.clear();
            app.file_browser_filter.clear();
        }
        KeyCode::Esc => {
            app.screen = Screen::DeviceMenu;
        }
        _ => {}
    }
    if app.file_browser_input_mode {
        app.file_browser_filter = app.file_browser_input.clone();
    }
}

pub fn handle_storage_menu(app: &mut App, key: KeyEvent) {
    if app.storage_cache_input_mode {
        match key.code {
            KeyCode::Char(c) => app.storage_cache_input.push(c),
            KeyCode::Backspace => {
                app.storage_cache_input.pop();
            }
            KeyCode::Enter => {
                let pkg = app.storage_cache_input.trim().to_string();
                app.storage_cache_input.clear();
                app.storage_cache_input_mode = false;
                if pkg.is_empty() {
                    app.status = i18n::tr("storage.cache.empty").to_string();
                    return;
                }
                if crate::validate::package(&pkg).is_err() {
                    app.status = i18n::tr("storage.cache.invalid").to_string();
                    return;
                }
                let serial = app.serial.clone().unwrap_or_default();
                match (|| -> anyhow::Result<()> {
                    let out = std::process::Command::new(&app.adb)
                        .args(["-s", &serial, "shell", "pm", "clear", &pkg])
                        .output()?;
                    let text = String::from_utf8_lossy(&out.stdout).to_string()
                        + &String::from_utf8_lossy(&out.stderr);
                    if !text.to_lowercase().contains("success") {
                        anyhow::bail!("{}", text.trim());
                    }
                    Ok(())
                })() {
                    Ok(_) => app.status = i18n::tr("storage.cache.done").to_string(),
                    Err(e) => app.status = rust_i18n::t!("app.error", error = e).to_string(),
                }
            }
            KeyCode::Esc => {
                app.storage_cache_input.clear();
                app.storage_cache_input_mode = false;
            }
            _ => {}
        }
        return;
    }
    let len = app.storage_menu_keys.len();
    match key.code {
        KeyCode::Up => App::move_list(&mut app.storage_menu_state, len, -1),
        KeyCode::Down => App::move_list(&mut app.storage_menu_state, len, 1),
        KeyCode::Enter => {
            let i = app.storage_menu_state.selected().unwrap_or(0);
            match i {
                0 => app.open_file_browser(),
                1 => {
                    if let Err(e) = app.ensure_safety() {
                        app.status = rust_i18n::t!("app.error", error = e).to_string();
                    }
                    app.app_action = AppAction::ClearCache;
                    app.app_view = AppView::Installed;
                    if !app.list_cache_matches(AppAction::ClearCache, AppView::Installed) {
                        app.load_packages(false);
                    }
                    app.status = i18n::tr("apps.status.help").to_string();
                    app.screen = Screen::AppList;
                }
                2 => app.open_storage_usage(),
                3 => app.open_storage_volumes(),
                4 => {
                    // Quick clean: borrar thumbnails
                    let serial = app.serial.clone().unwrap_or_default();
                    let path = "/sdcard/DCIM/.thumbnails".to_string();
                    if crate::adb::validate_remote_path(&path) {
                        let _ = crate::adb::remove_remote(&app.adb, &serial, &path);
                        app.status = i18n::tr("storage.clean.done").to_string();
                    }
                }
                5 => app.screen = Screen::DeviceMenu,
                _ => {}
            }
        }
        KeyCode::Esc => app.screen = Screen::DeviceMenu,
        _ => {}
    }
}

pub fn handle_storage_usage(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('r') || key.code == KeyCode::Char('R') {
        app.open_storage_usage();
    }
    if key.code == KeyCode::Esc {
        app.screen = Screen::StorageMenu;
    }
}

pub fn handle_storage_volumes(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.screen = Screen::StorageMenu;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Lang;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn main_menu_settings_index_opens_settings() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.menu_state.select(Some(1));
        handle_key(&mut app, press(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Settings);
    }

    #[test]
    fn main_menu_exit_index_quits() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.menu_state.select(Some(2));
        handle_key(&mut app, press(KeyCode::Enter));
        assert!(app.should_quit);
    }

    #[test]
    fn scripts_escape_returns_to_device_menu() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.screen = Screen::Scripts;
        handle_key(&mut app, press(KeyCode::Esc));
        assert_eq!(app.screen, Screen::DeviceMenu);
    }

    #[test]
    fn script_folders_escape_returns_to_scripts() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.screen = Screen::ScriptFolders;
        handle_key(&mut app, press(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Scripts);
    }

    #[test]
    fn device_menu_scripts_index_opens_scripts() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.serial = Some("serial".to_string());
        app.screen = Screen::DeviceMenu;
        app.device_menu_state.select(Some(1));
        handle_key(&mut app, press(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Scripts);
    }

    #[test]
    fn device_menu_back_index_returns_to_main_menu() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.serial = Some("serial".to_string());
        app.screen = Screen::DeviceMenu;
        app.device_menu_state.select(Some(9));
        handle_key(&mut app, press(KeyCode::Enter));
        assert_eq!(app.screen, Screen::MainMenu);
    }
}
