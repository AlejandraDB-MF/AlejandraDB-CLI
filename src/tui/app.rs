use crate::config::Lang;
use crate::i18n;
use crate::logcat::{self, LogcatEvent, LogcatSession};
use crate::processes;
#[cfg(test)]
use crate::processes::ProcessInfo;
use crate::safety::{self, AppKind};
use crate::scripts::{self, ScriptEvent, ScriptFolder};
use crate::theme::Theme;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::time::Instant;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::ListState;

use crate::adb::Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Uninstall,
    Reinstall,
    Enable,
    ClearCache,
    ForceStop,
    ClearData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceControlAction {
    Reboot,
    Recovery,
    Bootloader,
    PowerOff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkAction {
    Enable,
    Connect,
    Disconnect,
    Disable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Screen {
    DeviceSelect,
    DeviceDiscover,
    DiscoverDevices,
    DeviceDiscovery,
    ConnectedDevices,
    DeviceList,
    DevicesConnected,
    MainMenu,
    Scripts,
    ScriptFolders,
    Logcat,
    Processes,
    DeviceMenu,
    Settings,
    OutputPath,
    Language,
    AppsMenu,
    AppList,
    InstallApk,
    Confirm,
    DeviceInfo,
    DeviceControl,
    Network,
    WifiDiscover,
    WifiConnect,
    FileBrowser,
    StorageMenu,
    StorageUsage,
    StorageVolumes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkResult {
    pub text: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Installed,
    Available,
}

#[derive(Debug)]
pub struct AppCachePayload {
    pub serial: String,
    pub system: HashSet<String>,
    pub admins: HashSet<String>,
    pub paths: HashMap<String, String>,
    pub installed_all: Vec<String>,
    pub installed_third: Vec<String>,
    pub available: Vec<String>,
    pub disabled: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    NameAsc,
    NameDesc,
    CriticalFirst,
    CriticalLast,
}

impl SortMode {
    pub fn label(&self) -> String {
        match self {
            SortMode::NameAsc => i18n::tr("sort.name_asc"),
            SortMode::NameDesc => i18n::tr("sort.name_desc"),
            SortMode::CriticalFirst => i18n::tr("sort.critical_first"),
            SortMode::CriticalLast => i18n::tr("sort.critical_last"),
        }
    }
}

impl std::fmt::Display for SortMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label())
    }
}

pub struct App {
    pub adb: String,
    pub serial_override: Option<String>,
    pub serial: Option<String>,
    pub devices: Vec<Device>,
    pub device_state: ListState,
    pub device_discover_state: ListState,
    pub connected_devices_state: ListState,
    pub screen: Screen,
    pub last_screen: Option<Screen>,
    pub lang: Lang,
    pub menu_keys: Vec<&'static str>,
    pub menu_state: ListState,
    pub scripts: crate::tui::scripts_state::ScriptsState,
    pub logcat: crate::tui::logcat_state::LogcatState,
    pub processes: crate::tui::processes_state::ProcessesState,
    pub device_menu_keys: Vec<&'static str>,
    pub device_menu_state: ListState,
    pub device_control_keys: Vec<&'static str>,
    pub device_control_state: ListState,
    pub device_control_pending: Option<DeviceControlAction>,
    pub settings_menu_keys: Vec<&'static str>,
    pub settings_menu_state: ListState,
    pub language_menu_keys: Vec<&'static str>,
    pub language_menu_state: ListState,
    pub apps: crate::tui::apps_state::AppsState,
    pub status: String,
    pub should_quit: bool,
    pub loading: bool,
    pub output_path_input: String,
    pub output_path_input_mode: bool,
    pub device_info: crate::tui::device_info_state::DeviceInfoState,
    pub direct_shell_requested: bool,
    pub network: crate::tui::network_state::NetworkState,
    pub storage: crate::tui::storage_state::StorageState,
}

impl App {
    pub fn new(adb: String, serial_override: Option<String>, lang: Lang) -> Self {
        let mut device_state = ListState::default();
        device_state.select(Some(0));
        let mut device_discover_state = ListState::default();
        device_discover_state.select(Some(0));
        let mut connected_devices_state = ListState::default();
        connected_devices_state.select(Some(0));
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        let mut scripts_menu_state = ListState::default();
        scripts_menu_state.select(Some(0));
        let mut script_folder_state = ListState::default();
        script_folder_state.select(Some(0));
        let mut logcat_state = ListState::default();
        logcat_state.select(Some(0));
        let mut process_state = ListState::default();
        process_state.select(Some(0));
        let mut device_menu_state = ListState::default();
        device_menu_state.select(Some(0));
        let mut device_control_state = ListState::default();
        device_control_state.select(Some(0));
        let mut settings_menu_state = ListState::default();
        settings_menu_state.select(Some(0));
        let mut language_menu_state = ListState::default();
        language_menu_state.select(Some(0));
        let mut apps_menu_state = ListState::default();
        apps_menu_state.select(Some(0));
        let mut app_state = ListState::default();
        app_state.select(Some(0));
        let mut network_menu_state = ListState::default();
        network_menu_state.select(Some(0));
        let mut wifi_discover_state = ListState::default();
        wifi_discover_state.select(Some(0));
        let mut app = Self {
            adb,
            serial_override,
            serial: None,
            devices: Vec::new(),
            device_state,
            device_discover_state,
            connected_devices_state,
            screen: Screen::MainMenu,
            last_screen: None,
            lang,
            menu_keys: vec!["menu.main.devices", "menu.main.settings", "menu.main.exit"],
            menu_state,
            scripts: crate::tui::scripts_state::ScriptsState::default(),
            logcat: crate::tui::logcat_state::LogcatState::default(),
            processes: crate::tui::processes_state::ProcessesState::default(),
            device_menu_keys: vec![
                "device.menu.apps",
                "device.menu.scripts",
                "device.menu.logcat",
                "device.menu.processes",
                "device.menu.info",
                "device.menu.shell",
                "device.menu.control",
                "device.menu.network",
                "device.menu.storage",
                "device.menu.back",
            ],
            device_menu_state,
            device_control_keys: vec![
                "device.control.reboot",
                "device.control.recovery",
                "device.control.bootloader",
                "device.control.power_off",
                "device.control.back",
            ],
            device_control_state,
            device_control_pending: None,
            settings_menu_keys: vec!["settings.language", "settings.output_path", "settings.back"],
            settings_menu_state,
            language_menu_keys: vec!["language.english", "language.spanish", "language.back"],
            language_menu_state,
            apps: crate::tui::apps_state::AppsState::default(),
            output_path_input: String::new(),
            output_path_input_mode: false,
            status: i18n::tr("device.select.loading"),
            should_quit: false,
            loading: false,
            device_info: crate::tui::device_info_state::DeviceInfoState::default(),
            direct_shell_requested: false,
            network: crate::tui::network_state::NetworkState::default(),
            storage: crate::tui::storage_state::StorageState::default(),
        };
        app.refresh_network_menu();
        app
    }

    pub fn apply_language(&mut self, lang: Lang) {
        self.lang = lang;
        i18n::set_language(lang);
        let mut config = crate::config::Config::load();
        config.language = lang;
        match config.save() {
            Ok(()) => {
                self.status =
                    rust_i18n::t!("settings.language.saved", language = lang.native_name())
                        .to_string();
            }
            Err(error) => {
                self.status =
                    rust_i18n::t!("settings.language.save_error", error = error.to_string())
                        .to_string();
            }
        }
    }

    pub fn reset_device_state(&mut self) {
        self.apps.packages.clear();
        self.apps.available_packages.clear();
        self.apps.package_kinds.clear();
        self.apps.package_paths.clear();
        self.apps.system_packages.clear();
        self.apps.device_admins.clear();
        self.apps.selected.clear();
        self.apps.cached_list.clear();
        self.apps.filter.clear();
        self.apps.filter_mode = false;
        self.apps.apk_input.clear();
        self.apps.apk_result = None;
        self.apps.safety_loaded = false;
        self.apps.safety_serial = None;
        self.apps.app_cache_loaded = false;
        self.apps.app_cache_serial = None;
        self.apps.app_cache_rx = None;
        self.apps.app_cache_loading = false;
        self.apps.app_cache_just_loaded = false;
        self.apps.list_cache_valid = false;
        self.apps.list_cache_key = None;
        self.apps.app_state.select(Some(0));
        self.apps.apps_menu_state.select(Some(0));
        self.apps.app_action = AppAction::Uninstall;
        self.apps.app_view = AppView::Installed;
        self.device_info.device_info_lines.clear();
        self.device_info.device_info_scroll = 0;
        self.device_info.device_info_loading = false;
        self.direct_shell_requested = false;
        self.device_control_state.select(Some(0));
        self.device_control_pending = None;
        self.network.network_menu_state.select(Some(0));
        self.network.network_ip = None;
        self.network.network_ssid.clear();
        self.network.network_wifi_state.clear();
        self.network.network_loading = false;
        self.network.network_result = None;
        self.network.network_input.clear();
        self.network.network_input_mode = false;
        self.network.network_pending = None;
        self.network.network_pending_endpoint = None;
        self.network.network_endpoint = None;
        self.network.network_info_cached_at = None;
        self.network.network_info_rx = None;
        self.network.network_info_loading = false;
        self.stop_logcat();
        self.logcat.logcat_lines.clear();
        self.logcat.logcat_state.select(None);
        self.logcat.logcat_paused = false;
        self.processes.processes.clear();
        self.processes.process_state.select(Some(0));
        self.processes.process_pending = None;
        self.processes.process_filter.clear();
        self.processes.process_filter_mode = false;
        self.processes.processes_rx = None;
        self.processes.processes_loading = false;
        self.network.wifi_scanning = false;
        self.network.wifi_scan_rx = None;
        self.network.wifi_scan_results.clear();
        self.network.wifi_rejected.clear();
        self.network.wifi_subnet_input.clear();
        self.network.wifi_subnet_input_mode = false;
        self.network.wifi_scan_start = None;
        self.network.wifi_scan_progress = 0;
        self.refresh_network_menu();
    }

    pub fn select_device(&mut self, serial: String) {
        self.reset_device_state();
        self.serial = Some(serial);
        self.screen = Screen::DeviceMenu;
        self.status.clear();
    }

    pub fn open_device_select(&mut self) {
        self.screen = Screen::DeviceSelect;
        self.device_state.select(Some(0));
        self.status = i18n::tr("device.select.status").to_string();
    }

    pub fn open_connected_devices(&mut self) {
        self.screen = Screen::ConnectedDevices;
        self.connected_devices_state.select(Some(0));
        self.status = if self.devices.is_empty() {
            i18n::tr("device.select.no_usb")
        } else {
            i18n::tr("device.select.status")
        };
    }

    #[allow(dead_code)]
    pub fn open_device_list(&mut self) {
        self.open_connected_devices();
    }

    #[allow(dead_code)]
    pub fn open_devices_connected(&mut self) {
        self.open_connected_devices();
    }

    pub fn open_device_discover(&mut self) {
        self.screen = Screen::DeviceDiscover;
        self.device_discover_state.select(Some(0));
        self.status = i18n::tr("device.select.status").to_string();
    }

    #[allow(dead_code)]
    pub fn open_discover_devices(&mut self) {
        self.screen = Screen::DiscoverDevices;
        self.device_discover_state.select(Some(0));
        self.status = i18n::tr("device.select.status").to_string();
    }

    #[allow(dead_code)]
    pub fn open_device_discovery(&mut self) {
        self.screen = Screen::DeviceDiscovery;
        self.device_discover_state.select(Some(0));
        self.status = i18n::tr("device.select.status").to_string();
    }

    pub fn open_scripts(&mut self) {
        self.screen = Screen::Scripts;
        self.scripts.scripts_menu_state.select(Some(0));
        if !self.scripts.script_running {
            self.status = i18n::tr("scripts.hint");
        }
    }

    pub fn open_script_folders(&mut self) {
        self.screen = Screen::ScriptFolders;
        self.scripts.script_folder_state.select(Some(0));
        self.scripts.script_selected.clear();
        if !self.scripts.script_running {
            self.status = i18n::tr("scripts.folders.hint");
        }
    }

    pub fn toggle_script_folder(&mut self) {
        let index = self.scripts.script_folder_state.selected().unwrap_or(0);
        let Some(folder) = ScriptFolder::ALL.get(index).copied() else {
            return;
        };
        if self.scripts.script_selected.contains(&folder) {
            self.scripts.script_selected.remove(&folder);
        } else {
            self.scripts.script_selected.insert(folder);
        }
    }

    pub fn start_el_negro_samuel(&mut self) {
        if self.scripts.script_running || self.scripts.script_rx.is_some() {
            self.status = rust_i18n::t!(
                "scripts.status.already_running",
                script = scripts::EL_NEGRO_SAMUEL
            )
            .to_string();
            return;
        }
        if self.scripts.script_selected.is_empty() {
            self.status = i18n::tr("scripts.status.no_selection");
            return;
        }
        let Some(serial) = self.serial.clone() else {
            self.status = i18n::tr("scripts.status.no_device");
            return;
        };
        let output_path = crate::config::Config::load().output_dir();
        let folders = ScriptFolder::ALL
            .iter()
            .copied()
            .filter(|folder| self.scripts.script_selected.contains(folder))
            .collect();
        self.scripts.script_rx = Some(scripts::spawn_el_negro_samuel(
            self.adb.clone(),
            serial,
            output_path,
            folders,
        ));
        self.scripts.script_running = true;
        self.screen = Screen::DeviceMenu;
        self.status =
            rust_i18n::t!("scripts.status.started", script = scripts::EL_NEGRO_SAMUEL).to_string();
    }

    pub fn poll_script_events(&mut self) {
        let Some(receiver) = self.scripts.script_rx.take() else {
            return;
        };
        let mut finished = false;
        loop {
            match receiver.try_recv() {
                Ok(ScriptEvent::Status(status)) => self.status = status,
                Ok(ScriptEvent::Finished(summary)) => {
                    self.scripts.script_last_destination = Some(summary.destination.clone());
                    self.status = scripts::summary_message(&summary);
                    finished = true;
                    break;
                }
                Ok(ScriptEvent::Failed(error)) => {
                    self.status = rust_i18n::t!("scripts.status.error", error = error).to_string();
                    finished = true;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.status = i18n::tr("scripts.status.disconnected");
                    finished = true;
                    break;
                }
            }
        }
        if finished {
            self.scripts.script_running = false;
        } else {
            self.scripts.script_rx = Some(receiver);
        }
    }

    pub fn open_logcat(&mut self) {
        self.stop_logcat();
        self.screen = Screen::Logcat;
        self.logcat.logcat_lines.clear();
        self.logcat.logcat_state.select(Some(0));
        self.logcat.logcat_paused = false;
        let Some(serial) = self.serial.clone() else {
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        self.status = i18n::tr("logcat.status.loading");
        match LogcatSession::spawn(&self.adb, &serial) {
            Ok(session) => {
                self.logcat.logcat_session = Some(session);
                self.status = i18n::tr("logcat.hint");
            }
            Err(error) => {
                self.status = rust_i18n::t!("app.error", error = error).to_string();
            }
        }
    }

    pub fn poll_logcat(&mut self) {
        let Some(session) = self.logcat.logcat_session.as_ref() else {
            return;
        };
        let mut events = Vec::new();
        let mut disconnected = false;
        loop {
            match session.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }
        let mut had_error = false;
        for event in events {
            match event {
                LogcatEvent::Line(line) => self.push_logcat_line(line),
                LogcatEvent::Error(error) => {
                    had_error = true;
                    self.status = rust_i18n::t!("logcat.error.process", error = error).to_string();
                }
            }
        }
        if disconnected {
            self.logcat.logcat_session = None;
            if !had_error {
                self.status = i18n::tr("logcat.status.stopped");
            }
        }
    }

    fn push_logcat_line(&mut self, line: String) {
        if self.logcat.logcat_lines.len() >= logcat::MAX_LINES {
            self.logcat.logcat_lines.pop_front();
            if self.logcat.logcat_paused
                && let Some(selected) = self.logcat.logcat_state.selected()
            {
                self.logcat
                    .logcat_state
                    .select(Some(selected.saturating_sub(1)));
            }
        }
        self.logcat.logcat_lines.push_back(line);
        if self.logcat.logcat_paused {
            if self.logcat.logcat_state.selected().is_none() {
                self.logcat.logcat_state.select(Some(0));
            }
        } else {
            self.logcat
                .logcat_state
                .select(Some(self.logcat.logcat_lines.len().saturating_sub(1)));
        }
    }

    pub fn logcat_move(&mut self, delta: isize) {
        if self.logcat.logcat_lines.is_empty() {
            return;
        }
        let current = self.logcat.logcat_state.selected().unwrap_or(0) as isize;
        let max = self.logcat.logcat_lines.len() as isize - 1;
        let next = (current + delta).clamp(0, max);
        self.logcat.logcat_state.select(Some(next as usize));
    }

    pub fn toggle_logcat_pause(&mut self) {
        self.logcat.logcat_paused = !self.logcat.logcat_paused;
        if self.logcat.logcat_paused {
            self.status = i18n::tr("logcat.status.paused");
        } else {
            if !self.logcat.logcat_lines.is_empty() {
                let last = self.logcat.logcat_lines.len() - 1;
                self.logcat.logcat_state.select(Some(last));
            }
            self.status = i18n::tr("logcat.status.resumed");
        }
    }

    pub fn clear_logcat(&mut self) {
        self.logcat.logcat_lines.clear();
        self.logcat.logcat_state.select(None);
    }

    pub fn export_logcat(&mut self) {
        let output_dir = crate::config::Config::load().output_dir();
        if let Err(error) = std::fs::create_dir_all(&output_dir) {
            self.status =
                rust_i18n::t!("logcat.export.error", error = error.to_string()).to_string();
            return;
        }
        let now =
            time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        let format =
            time::macros::format_description!("[year]-[month]-[day]-[hour][minute][second]");
        let timestamp = now.format(&format).unwrap_or_else(|_| "export".to_string());
        let path = output_dir.join(format!("logcat-{timestamp}.txt"));
        let mut content = self
            .logcat
            .logcat_lines
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        if !content.is_empty() {
            content.push('\n');
        }
        match std::fs::write(&path, content) {
            Ok(()) => {
                self.status =
                    rust_i18n::t!("logcat.export.success", path = path.display().to_string())
                        .to_string();
            }
            Err(error) => {
                self.status =
                    rust_i18n::t!("logcat.export.error", error = error.to_string()).to_string();
            }
        }
    }

    pub fn stop_logcat(&mut self) {
        self.logcat.logcat_session = None;
    }

    pub fn open_processes(&mut self) {
        self.screen = Screen::Processes;
        self.processes.process_state.select(Some(0));
        self.processes.process_pending = None;
        self.load_processes();
    }

    pub fn open_output_path(&mut self) {
        self.output_path_input = crate::config::Config::load().output_path;
        self.output_path_input_mode = true;
        self.screen = Screen::OutputPath;
        self.status = i18n::tr("settings.output_path.hint");
    }

    pub fn apply_output_path(&mut self) {
        let value = self.output_path_input.trim().to_string();
        if value.is_empty() {
            self.status = i18n::tr("settings.output_path.empty");
            return;
        }
        let mut config = crate::config::Config::load();
        config.output_path = value.clone();
        match config.save() {
            Ok(()) => {
                self.status = rust_i18n::t!("settings.output_path.saved", path = value).to_string();
            }
            Err(error) => {
                self.status =
                    rust_i18n::t!("settings.output_path.save_error", error = error.to_string())
                        .to_string();
            }
        }
    }

    pub fn apply_wifi_subnet(&mut self) {
        let value = self.network.wifi_subnet_input.trim().to_string();
        let mut config = crate::config::Config::load();
        config.set_wifi_subnet(value.clone());
        match config.save() {
            Ok(()) => {
                if value.is_empty() {
                    self.status = i18n::tr("network.subnet.saved_auto");
                } else {
                    self.status = rust_i18n::t!("network.subnet.saved", subnet = value).to_string();
                }
            }
            Err(error) => {
                self.status = rust_i18n::t!("network.subnet.save_error", error = error.to_string())
                    .to_string();
            }
        }
        self.network.wifi_subnet_input_mode = false;
    }

    pub fn open_wifi_subnet_input(&mut self) {
        let config = crate::config::Config::load();
        self.network.wifi_subnet_input = config.wifi_subnet.unwrap_or_default();
        self.network.wifi_subnet_input_mode = true;
        self.status = i18n::tr("network.subnet.hint");
    }

    pub fn filtered_process_indices(&self) -> Vec<usize> {
        let filter = self.processes.process_filter.trim().to_lowercase();
        if filter.is_empty() {
            return (0..self.processes.processes.len()).collect();
        }
        self.processes
            .processes
            .iter()
            .enumerate()
            .filter(|(_, process)| {
                process.pid.to_string().to_lowercase().contains(&filter)
                    || process.user.to_lowercase().contains(&filter)
                    || process.name.to_lowercase().contains(&filter)
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub fn load_processes(&mut self) {
        let Some(serial) = self.serial.clone() else {
            self.processes.processes.clear();
            self.processes.process_state.select(None);
            self.processes.process_pending = None;
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        if self.processes.processes_loading {
            return;
        }
        self.processes.processes_loading = true;
        self.status = i18n::tr("processes.status.loading");
        let adb = self.adb.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.processes.processes_rx = Some(rx);
        std::thread::spawn(move || {
            let res = processes::list(&adb, &serial)
                .map_err(|e| rust_i18n::t!("processes.error.list", error = e).to_string());
            let _ = tx.send(res);
        });
    }

    pub fn poll_processes(&mut self) {
        let Some(rx) = self.processes.processes_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(list)) => {
                self.processes.processes_loading = false;
                self.processes.processes_rx = None;
                let count = list.len();
                self.processes.processes = list;
                self.processes.process_pending = None;
                if count == 0 {
                    self.processes.process_state.select(None);
                    self.status = i18n::tr("processes.status.empty");
                } else {
                    self.processes.process_state.select(Some(0));
                    self.status =
                        rust_i18n::t!("processes.status.loaded", count = count).to_string();
                }
            }
            Ok(Err(e)) => {
                self.processes.processes_loading = false;
                self.processes.processes_rx = None;
                self.processes.processes.clear();
                self.processes.process_state.select(None);
                self.processes.process_pending = None;
                self.status = e;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.processes.processes_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.processes.processes_loading = false;
                self.processes.processes_rx = None;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    pub fn request_process_kill(&mut self) {
        let indices = self.filtered_process_indices();
        let Some(position) = self.processes.process_state.selected() else {
            self.status = i18n::tr("processes.status.no_selection");
            return;
        };
        if position >= indices.len() {
            self.status = i18n::tr("processes.status.no_selection");
            return;
        }
        let Some(process) = self.processes.processes.get(indices[position]).cloned() else {
            self.status = i18n::tr("processes.status.no_selection");
            return;
        };
        if process.pid == 1 {
            self.status = i18n::tr("processes.status.protected");
            return;
        }
        self.processes.process_pending = Some(process);
    }

    pub fn execute_process_kill(&mut self) {
        let Some(process) = self.processes.process_pending.take() else {
            return;
        };
        if process.pid == 1 {
            self.status = i18n::tr("processes.status.protected");
            return;
        }
        let Some(serial) = self.serial.clone() else {
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        match processes::kill(&self.adb, &serial, process.pid) {
            Ok(()) => {
                let status =
                    rust_i18n::t!("processes.status.killed", pid = process.pid).to_string();
                self.load_processes();
                self.status = status;
            }
            Err(error) => {
                self.status = rust_i18n::t!(
                    "processes.status.kill_failed",
                    pid = process.pid,
                    error = error
                )
                .to_string();
            }
        }
    }

    pub fn load_wifi_discovered(&mut self) {
        self.loading = true;
        self.status = i18n::tr("wifi.discover.loading");
        match crate::adb::mdns_services(&self.adb) {
            Ok(endpoints) => {
                self.network.wifi_discovered = endpoints;
                if self.network.wifi_discovered.is_empty() {
                    self.network.wifi_discover_state.select(None);
                    self.status = i18n::tr("wifi.status.no_services");
                } else {
                    self.network.wifi_discover_state.select(Some(0));
                    self.status = i18n::tr("device.select.status");
                }
            }
            Err(e) => {
                self.network.wifi_discovered.clear();
                self.network.wifi_discover_state.select(None);
                self.status = rust_i18n::t!("app.error", error = e).to_string();
            }
        }
        self.loading = false;
    }

    pub fn effective_wifi_subnet(&self) -> String {
        crate::config::Config::load().wifi_subnet()
    }

    pub fn open_wifi_discover(&mut self) {
        self.screen = Screen::WifiDiscover;
        if self.network.wifi_discovered.is_empty() && self.network.wifi_rejected.is_empty() {
            self.network.wifi_discover_state.select(Some(0));
            self.network.wifi_show_rejected = false;
            self.network.wifi_scan_results.clear();
            self.network.wifi_scan_progress = 0;
            self.network.wifi_scan_start = Some(Instant::now());
            self.start_wifi_scan();
        } else {
            // Reutilizar resultados previos sin re-escanear.
            self.network.wifi_show_rejected = false;
            let len = self.network.wifi_discovered.len();
            if len == 0 {
                self.network.wifi_discover_state.select(None);
            } else {
                let sel = self
                    .network
                    .wifi_discover_state
                    .selected()
                    .unwrap_or(0)
                    .min(len - 1);
                self.network.wifi_discover_state.select(Some(sel));
            }
            if let Some(at) = self.network.wifi_scan_last_at {
                self.status = format!("Last scan: {}", at.elapsed().as_secs());
            }
        }
    }

    pub fn open_storage_menu(&mut self) {
        self.screen = Screen::StorageMenu;
        self.storage.storage_menu_state.select(Some(0));
        self.status = i18n::tr("storage.hint").to_string();
    }

    pub fn open_storage_usage(&mut self) {
        self.screen = Screen::StorageUsage;
        self.storage.storage_usage.clear();
        self.storage.storage_usage_loading = true;
        self.status = i18n::tr("storage.usage.loading").to_string();
        let serial = self.serial.clone().unwrap_or_default();
        let adb = self.adb.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.storage.storage_usage_rx = Some(rx);
        std::thread::spawn(move || {
            let folders = [
                "/sdcard/DCIM",
                "/sdcard/Download",
                "/sdcard/Documents",
                "/sdcard/Movies",
                "/sdcard/Pictures",
                "/sdcard/Android/data",
            ];
            let mut usage = Vec::new();
            for f in folders {
                let size = crate::adb::du_remote(&adb, &serial, f).unwrap_or(0);
                usage.push((f.to_string(), size));
            }
            usage.sort_by_key(|a| std::cmp::Reverse(a.1));
            let _ = tx.send(usage);
        });
    }

    pub fn poll_storage_usage(&mut self) {
        let Some(rx) = self.storage.storage_usage_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(usage) => {
                self.storage.storage_usage_rx = None;
                self.storage.storage_usage_loading = false;
                self.storage.storage_usage = usage;
                self.status = i18n::tr("storage.usage.loaded").to_string();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.storage.storage_usage_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.storage.storage_usage_rx = None;
                self.storage.storage_usage_loading = false;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    pub fn open_storage_volumes(&mut self) {
        self.screen = Screen::StorageVolumes;
        self.storage.storage_volumes.clear();
        self.storage.storage_volumes_loading = true;
        self.status = i18n::tr("storage.volumes.loading").to_string();
        let serial = self.serial.clone().unwrap_or_default();
        let adb = self.adb.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.storage.storage_volumes_rx = Some(rx);
        std::thread::spawn(move || {
            let mut vols = Vec::new();
            if let Ok(text) = crate::adb::sm_list_volumes(&adb, &serial) {
                for line in text.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        vols.push(line.to_string());
                    }
                }
            }
            let _ = tx.send(vols);
        });
    }

    pub fn poll_storage_volumes(&mut self) {
        let Some(rx) = self.storage.storage_volumes_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(vols) => {
                self.storage.storage_volumes_rx = None;
                self.storage.storage_volumes_loading = false;
                self.storage.storage_volumes = vols;
                self.status = i18n::tr("storage.volumes.loaded").to_string();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.storage.storage_volumes_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.storage.storage_volumes_rx = None;
                self.storage.storage_volumes_loading = false;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    pub fn open_file_browser(&mut self) {
        self.screen = Screen::FileBrowser;
        self.storage.file_browser_loading = true;
        self.status = "↑↓ navegar  ← subir  → entrar  d descargar  c copiar  m mover  v pegar  x borrar  r refrescar  / filtrar  Esc salir".to_string();
        self.storage.file_browser_pending_delete = None;
        self.storage.file_browser_clipboard = None;
        self.load_file_browser();
    }

    pub fn load_file_browser(&mut self) {
        let serial = self.serial.clone().unwrap_or_default();
        if serial.is_empty() {
            self.status = i18n::tr("app.status.no_device");
            return;
        }
        if self.storage.file_browser_loading {
            return;
        }
        self.storage.file_browser_loading = true;
        self.status = i18n::tr("files.loading");
        let adb = self.adb.clone();
        let path = self.storage.file_browser_path.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.storage.file_browser_rx = Some(rx);
        std::thread::spawn(move || {
            let res = crate::adb::list_remote(&adb, &serial, &path)
                .map_err(|e| rust_i18n::t!("app.error", error = e).to_string());
            let _ = tx.send(res);
        });
    }

    pub fn poll_file_browser(&mut self) {
        let Some(rx) = self.storage.file_browser_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(entries)) => {
                self.storage.file_browser_rx = None;
                self.storage.file_browser_loading = false;
                self.storage.file_browser_entries = entries;
                if self.storage.file_browser_entries.is_empty() {
                    self.storage.file_browser_state.select(None);
                    self.status = i18n::tr("files.empty").to_string();
                } else {
                    self.storage.file_browser_state.select(Some(0));
                    self.status = i18n::tr("files.loaded").to_string();
                }
            }
            Ok(Err(e)) => {
                self.storage.file_browser_rx = None;
                self.storage.file_browser_loading = false;
                self.status = e;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.storage.file_browser_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.storage.file_browser_rx = None;
                self.storage.file_browser_loading = false;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    pub fn start_wifi_scan(&mut self) {
        if self.network.wifi_scanning {
            return;
        }
        self.network.wifi_scanning = true;
        self.network.wifi_discovered.clear();
        self.network.wifi_rejected.clear();
        self.network.wifi_show_rejected = false;
        self.network.wifi_scan_results.clear();
        self.network.wifi_scan_progress = 0;
        self.network.wifi_scan_start = Some(Instant::now());
        self.status = i18n::tr("wifi.discover.scanning");

        let (sender, receiver) = std::sync::mpsc::channel();
        let adb = self.adb.clone();
        let subnet = self.effective_wifi_subnet();
        std::thread::spawn(move || {
            let _ = crate::adb::wifi_scan_network(&adb, &subnet, 800, sender);
        });
        self.network.wifi_scan_rx = Some(receiver);
    }

    pub fn poll_wifi_scan(&mut self) {
        let Some(receiver) = self.network.wifi_scan_rx.take() else {
            return;
        };
        let mut done = false;
        loop {
            match receiver.try_recv() {
                Ok(entry) => {
                    if entry.rejected {
                        self.network.wifi_rejected.push(entry.endpoint);
                    } else {
                        self.network.wifi_discovered.push(entry.endpoint);
                    }
                    self.network.wifi_scan_progress =
                        self.network.wifi_discovered.len() + self.network.wifi_rejected.len();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    done = true;
                    break;
                }
            }
        }
        if done {
            self.network.wifi_scanning = false;
            self.network.wifi_scan_rx = None;
            self.network.wifi_scan_start = None;
            self.network.wifi_scan_last_at = Some(Instant::now());
            self.network.wifi_discovered.sort();
            self.network.wifi_rejected.sort();
            if self.network.wifi_discovered.is_empty() && self.network.wifi_rejected.is_empty() {
                self.status = i18n::tr("wifi.discover.empty");
            } else {
                self.status = i18n::tr("device.select.status");
            }
        } else {
            self.network.wifi_scan_rx = Some(receiver);
        }
    }

    pub fn open_device_control(&mut self) {
        self.screen = Screen::DeviceControl;
        self.device_control_state.select(Some(0));
        self.device_control_pending = None;
        self.status.clear();
    }

    pub fn refresh_network_menu(&mut self) {
        let mut keys: Vec<&'static str> = vec!["network.menu.info"];

        // Wi-Fi toggle (only the applicable action)
        if self.network.wifi_enabled {
            keys.push("network.menu.wifi_off");
        } else {
            keys.push("network.menu.wifi_on");
        }
        // Mobile data toggle
        if self.network.mobile_data_enabled {
            keys.push("network.menu.data_off");
        } else {
            keys.push("network.menu.data_on");
        }
        // Airplane mode toggle
        if self.network.airplane_mode {
            keys.push("network.menu.airplane_off");
        } else {
            keys.push("network.menu.airplane_on");
        }

        if self.network.network_endpoint.is_some() {
            keys.push("network.menu.disconnect");
            keys.push("network.menu.disable");
        } else {
            keys.push("network.menu.enable");
            keys.push("network.menu.connect");
        }
        keys.push("network.menu.set_subnet");
        keys.push("network.menu.back");

        self.network.network_menu_keys = keys;
        self.network.network_menu_state.select(Some(0));
    }

    pub fn open_network(&mut self) {
        self.screen = Screen::Network;
        self.network.network_menu_state.select(Some(0));
        self.network.network_input.clear();
        self.network.network_input_mode = false;
        self.network.network_pending = None;
        self.network.network_pending_endpoint = None;
        self.network.network_result = None;
        self.load_network_info();
        self.refresh_network_menu();
    }

    pub fn load_network_info(&mut self) {
        let Some(serial) = self.serial.clone() else {
            self.network.network_result = Some(ApkResult {
                text: i18n::tr("app.status.no_device"),
                is_error: true,
            });
            self.status = i18n::tr("app.status.no_device");
            self.network.network_loading = false;
            return;
        };

        // Use cached data if still fresh.
        if let Some(cached_at) = self.network.network_info_cached_at
            && cached_at.elapsed() < std::time::Duration::from_secs(10)
        {
            self.refresh_network_menu();
            if let Some(ep) = &self.network.network_endpoint {
                self.status =
                    rust_i18n::t!("network.status.wifi_adb_connected", endpoint = ep).to_string();
            } else {
                self.status = i18n::tr("network.status.no_connection");
            }
            return;
        }

        if self.network.network_info_loading {
            return;
        }
        self.network.network_info_loading = true;
        self.network.network_loading = true;
        self.status = i18n::tr("info.loading");

        let adb = self.adb.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.network.network_info_rx = Some(rx);
        std::thread::spawn(move || {
            let info = crate::adb::network_info(&adb, &serial).ok();
            let endpoints = crate::adb::wifi_adb_endpoints(&adb).unwrap_or_default();
            let _ = tx.send((info, endpoints, serial));
        });
    }

    pub fn poll_network_info(&mut self) {
        let Some(rx) = self.network.network_info_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok((info, endpoints, serial)) => {
                self.network.network_info_loading = false;
                self.network.network_loading = false;
                if let Some(info) = info {
                    self.network.network_ip = info.ip;
                    self.network.network_ssid = info.ssid;
                    self.network.network_wifi_state = info.wifi_state;
                    self.network.wifi_enabled = info.wifi_enabled;
                    self.network.mobile_data_enabled = info.mobile_data_enabled;
                    self.network.airplane_mode = info.airplane_mode;
                }
                let ip = self.network.network_ip.clone();
                if let Some(existing) = self.network.network_endpoint.clone()
                    && !endpoints.contains(&existing)
                {
                    self.network.network_endpoint = None;
                }
                if self.network.network_endpoint.is_none() {
                    if let Some(ip) = ip.as_ref() {
                        let prefix = format!("{ip}:");
                        if let Some(ep) = endpoints.iter().find(|e| e.starts_with(&prefix)) {
                            self.network.network_endpoint = Some(ep.clone());
                        } else if endpoints.len() == 1 {
                            self.network.network_endpoint = endpoints.first().cloned();
                        }
                    } else if endpoints.len() == 1 {
                        self.network.network_endpoint = endpoints.first().cloned();
                    }
                }
                self.refresh_network_menu();
                self.network.network_info_cached_at = Some(std::time::Instant::now());
                if let Some(ep) = &self.network.network_endpoint {
                    self.status = rust_i18n::t!("network.status.wifi_adb_connected", endpoint = ep)
                        .to_string();
                } else {
                    self.status = i18n::tr("network.status.no_connection");
                }
                let _ = serial;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.network.network_info_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.network.network_info_loading = false;
                self.network.network_loading = false;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    pub fn start_network_action(
        &mut self,
        kind: &str,
        f: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) {
        if self.network.network_action_loading {
            return;
        }
        self.network.network_action_loading = true;
        self.network.network_action_kind = Some(kind.to_string());
        self.status = i18n::tr("files.loading");
        let (tx, rx) = std::sync::mpsc::channel();
        self.network.network_action_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });
    }

    pub fn poll_network_action(&mut self) {
        let Some(rx) = self.network.network_action_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(msg)) => {
                self.network.network_action_loading = false;
                self.network.network_action_rx = None;
                let kind = self.network.network_action_kind.take().unwrap_or_default();
                let endpoint = self.network.network_pending_endpoint.clone();
                self.network.network_pending_endpoint = None;
                self.status = msg;
                match kind.as_str() {
                    "disconnect" | "disable" => {
                        self.network.network_endpoint = None;
                        self.load_devices();
                        if kind == "disconnect" {
                            self.screen = Screen::DeviceMenu;
                        }
                    }
                    "connect" => {
                        if let Some(ep) = endpoint {
                            let normalized = crate::adb::normalize_network_endpoint(&ep);
                            if !normalized.is_empty() && normalized != ":5555" {
                                self.network.network_endpoint = Some(normalized);
                            }
                        }
                        self.load_devices();
                    }
                    _ => {}
                }
                self.refresh_network_menu();
                self.load_network_info();
            }
            Ok(Err(e)) => {
                self.network.network_action_loading = false;
                self.network.network_action_rx = None;
                self.network.network_action_kind = None;
                self.status = e;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.network.network_action_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.network.network_action_loading = false;
                self.network.network_action_rx = None;
                self.network.network_action_kind = None;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    pub fn execute_device_control(&mut self, action: DeviceControlAction) {
        let Some(serial) = self.serial.clone() else {
            self.status = i18n::tr("app.status.no_device");
            self.device_control_pending = None;
            return;
        };
        let result = match action {
            DeviceControlAction::Reboot => {
                crate::adb::reboot(&self.adb, &serial, crate::adb::RebootTarget::Normal)
            }
            DeviceControlAction::Recovery => {
                crate::adb::reboot(&self.adb, &serial, crate::adb::RebootTarget::Recovery)
            }
            DeviceControlAction::Bootloader => {
                crate::adb::reboot(&self.adb, &serial, crate::adb::RebootTarget::Bootloader)
            }
            DeviceControlAction::PowerOff => crate::adb::power_off(&self.adb, &serial),
        };
        match result {
            Ok(()) => {
                self.device_control_pending = None;
                let saved = match action {
                    DeviceControlAction::PowerOff => i18n::tr("device.control.powering_off"),
                    _ => i18n::tr("device.control.rebooting"),
                };
                self.status = saved.clone();
                self.screen = Screen::DeviceSelect;
                self.load_devices();
                self.screen = Screen::DeviceSelect;
                if self.status != saved {
                    self.status = saved;
                }
            }
            Err(_) => {
                self.device_control_pending = None;
                self.status = i18n::tr("device.control.failed");
            }
        }
    }

    pub fn scroll_device_info(&mut self, delta: isize) {
        let current = self.device_info.device_info_scroll as isize;
        let max_scroll = self.device_info.device_info_lines.len().saturating_sub(1) as isize;
        self.device_info.device_info_scroll = (current + delta).clamp(0, max_scroll) as u16;
    }

    pub fn request_direct_shell(&mut self) {
        if self.serial.is_none() {
            self.status = i18n::tr("app.status.no_device");
            return;
        }
        self.direct_shell_requested = true;
    }

    fn auto_serial(&self) -> Option<String> {
        if let Some(serial) = &self.serial_override
            && let Some(device) = self
                .devices
                .iter()
                .find(|d| d.serial == *serial && d.state == "device")
        {
            return Some(device.serial.clone());
        }
        if let Ok(env_serial) = std::env::var("ANDROID_SERIAL")
            && !env_serial.is_empty()
            && let Some(device) = self
                .devices
                .iter()
                .find(|d| d.serial == env_serial && d.state == "device")
        {
            return Some(device.serial.clone());
        }
        None
    }

    pub fn load_devices(&mut self) {
        match crate::adb::devices(&self.adb) {
            Ok(devs) => {
                // Mostrar todos los estados (device/offline/unauthorized) para diagnóstico.
                let mut usable: Vec<Device> = devs;
                for d in &mut usable {
                    let (manufacturer, model) =
                        crate::adb::device_brand_model(&self.adb, &d.serial);
                    d.manufacturer = manufacturer;
                    d.model = model;
                }
                // Deduplicar: si el mismo modelo está conectado por USB y Wi-Fi,
                // dar prioridad al USB (ocultar la entrada Wi-Fi duplicada).
                let usb_keys: std::collections::HashSet<(String, String)> = usable
                    .iter()
                    .filter(|d| !d.serial.contains(':'))
                    .map(|d| (d.manufacturer.clone(), d.model.clone()))
                    .collect();
                let mut deduped: Vec<Device> = usable
                    .into_iter()
                    .filter(|d| {
                        !(d.serial.contains(':')
                            && usb_keys.contains(&(d.manufacturer.clone(), d.model.clone())))
                    })
                    .collect();
                // USB primero, luego Wi-Fi.
                deduped.sort_by_key(|d| d.serial.contains(':'));
                self.devices = deduped;
                if let Some(serial) = self.auto_serial() {
                    let index = self
                        .devices
                        .iter()
                        .position(|device| device.serial == serial);
                    self.select_device(serial);
                    if let Some(index) = index {
                        self.device_state.select(Some(index));
                    }
                    return;
                }
                if self.devices.is_empty() {
                    self.connected_devices_state.select(None);
                    self.status = if matches!(
                        self.screen,
                        Screen::ConnectedDevices | Screen::DeviceList | Screen::DevicesConnected
                    ) {
                        i18n::tr("device.select.none")
                    } else {
                        i18n::tr("app.status.no_device")
                    };
                } else {
                    self.connected_devices_state.select(Some(0));
                    self.status = if matches!(
                        self.screen,
                        Screen::ConnectedDevices | Screen::DeviceList | Screen::DevicesConnected
                    ) {
                        i18n::tr("device.select.status")
                    } else {
                        i18n::tr("app.status.no_device_selected")
                    };
                }
            }
            Err(e) => {
                self.status = rust_i18n::t!("app.error", error = e).to_string();
            }
        }
    }

    pub fn ensure_safety(&mut self) -> Result<(), anyhow::Error> {
        if self.apps.safety_loaded && self.apps.safety_serial.as_deref() == self.serial.as_deref() {
            return Ok(());
        }
        if self.apps.app_cache_loading {
            return Ok(());
        }
        // Use cached safety data if available for this serial.
        if self.apps.app_cache_loaded
            && self.apps.app_cache_serial.as_deref() == self.serial.as_deref()
        {
            self.apps.system_packages = self.apps.app_cache_system.clone();
            self.apps.device_admins = self.apps.app_cache_admins.clone();
            self.apps.package_paths = self.apps.app_cache_paths.clone();
            self.apps.safety_loaded = true;
            self.apps.safety_serial = self.serial.clone();
            return Ok(());
        }
        let serial = self
            .serial
            .clone()
            .ok_or_else(|| anyhow::anyhow!("{}", i18n::tr("app.status.no_device_selected")))?;
        self.apps.system_packages = safety::system_packages(&self.adb, &serial)?;
        self.apps.device_admins = safety::device_admins(&self.adb, &serial)?;
        self.apps.package_paths = safety::package_paths(&self.adb, &serial)?;
        self.apps.safety_loaded = true;
        self.apps.safety_serial = Some(serial);
        Ok(())
    }

    pub fn ensure_app_cache(&mut self) {
        if self.apps.app_cache_loaded && self.apps.app_cache_serial == self.serial {
            return;
        }
        if self.apps.app_cache_loading {
            return;
        }
        let Some(serial) = self.serial.clone() else {
            return;
        };

        self.apps.app_cache_loading = true;
        self.apps.app_cache_just_loaded = false;
        let adb = self.adb.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.apps.app_cache_rx = Some(rx);

        std::thread::spawn(move || {
            let mut error_msg: Option<String> = None;
            let system = safety::system_packages(&adb, &serial).unwrap_or_else(|e| {
                if error_msg.is_none() {
                    error_msg = Some(e.to_string());
                }
                HashSet::new()
            });
            let admins = safety::device_admins(&adb, &serial).unwrap_or_else(|e| {
                if error_msg.is_none() {
                    error_msg = Some(e.to_string());
                }
                HashSet::new()
            });
            let paths = safety::package_paths(&adb, &serial).unwrap_or_else(|e| {
                if error_msg.is_none() {
                    error_msg = Some(e.to_string());
                }
                HashMap::new()
            });

            let opts_all = crate::cli::ListOpts {
                ..Default::default()
            };
            let installed_all = match crate::package::list(&adb, &serial, &opts_all) {
                Ok(v) => v,
                Err(e) => {
                    if error_msg.is_none() {
                        error_msg = Some(e.to_string());
                    }
                    Vec::new()
                }
            };
            let opts_third = crate::cli::ListOpts {
                third_party: true,
                ..Default::default()
            };
            let installed_third =
                crate::package::list(&adb, &serial, &opts_third).unwrap_or_default();
            let available = crate::package::available_system(&adb, &serial).unwrap_or_default();
            let opts_disabled = crate::cli::ListOpts {
                disabled: true,
                ..Default::default()
            };
            let disabled = crate::package::list(&adb, &serial, &opts_disabled).unwrap_or_default();

            let payload = AppCachePayload {
                serial: serial.clone(),
                system,
                admins,
                paths,
                installed_all,
                installed_third,
                available,
                disabled,
                error: error_msg,
            };
            let _ = tx.send(payload);
        });
    }

    pub fn poll_app_cache(&mut self) {
        let Some(rx) = self.apps.app_cache_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(payload) => {
                self.apps.app_cache_loading = false;
                self.apps.app_cache_serial = Some(payload.serial.clone());
                self.apps.app_cache_system = payload.system.clone();
                self.apps.app_cache_admins = payload.admins.clone();
                self.apps.app_cache_paths = payload.paths.clone();
                self.apps.app_cache_installed_all = payload.installed_all;
                self.apps.app_cache_installed_third = payload.installed_third;
                self.apps.app_cache_available = payload.available;
                self.apps.app_cache_disabled = payload.disabled;
                self.apps.app_cache_loaded = true;
                if let Some(err) = payload.error {
                    self.status = rust_i18n::t!("app.error", error = err).to_string();
                }

                self.apps.system_packages = self.apps.app_cache_system.clone();
                self.apps.device_admins = self.apps.app_cache_admins.clone();
                self.apps.package_paths = self.apps.app_cache_paths.clone();
                self.apps.safety_loaded = true;
                self.apps.safety_serial = self.apps.app_cache_serial.clone();

                self.apps.app_cache_just_loaded = true;
                if self.screen == Screen::AppList {
                    match (self.apps.app_action, self.apps.app_view) {
                        (_, AppView::Available) => self.load_available(),
                        (AppAction::Enable, AppView::Installed) => self.load_disabled_packages(),
                        _ => self.load_packages(false),
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                self.apps.app_cache_rx = Some(rx);
            }
            Err(TryRecvError::Disconnected) => {
                self.apps.app_cache_loading = false;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    fn list_cache_context(&self) -> (AppAction, AppView, Option<String>) {
        (
            self.apps.app_action,
            self.apps.app_view,
            self.serial.clone(),
        )
    }

    pub fn list_cache_matches(&self, action: AppAction, view: AppView) -> bool {
        if !self.apps.list_cache_valid {
            return false;
        }
        let key = (action, view, self.serial.clone());
        self.apps.list_cache_key.as_ref() == Some(&key)
    }

    pub fn kind(&self, pkg: &str) -> AppKind {
        if let Some(k) = self.apps.package_kinds.get(pkg) {
            return *k;
        }
        let path = self.apps.package_paths.get(pkg);
        safety::classify(
            pkg,
            &self.apps.system_packages,
            &self.apps.device_admins,
            path.map(|s| s.as_str()),
        )
    }

    #[allow(clippy::ptr_arg)]
    pub fn sort_list(&self, list: &mut Vec<String>) {
        use crate::safety::AppKind;
        match self.apps.sort_mode {
            SortMode::NameAsc => {
                list.sort();
            }
            SortMode::NameDesc => {
                list.sort_by(|a, b| b.cmp(a));
            }
            SortMode::CriticalFirst => {
                list.sort_by(|a, b| {
                    let pa = match self.kind(a) {
                        AppKind::Critical => 0,
                        AppKind::System => 1,
                        AppKind::DeviceAdmin => 2,
                        AppKind::User => 3,
                    };
                    let pb = match self.kind(b) {
                        AppKind::Critical => 0,
                        AppKind::System => 1,
                        AppKind::DeviceAdmin => 2,
                        AppKind::User => 3,
                    };
                    pa.cmp(&pb).then_with(|| a.cmp(b))
                });
            }
            SortMode::CriticalLast => {
                list.sort_by(|a, b| {
                    let pa = match self.kind(a) {
                        AppKind::User => 0,
                        AppKind::DeviceAdmin => 1,
                        AppKind::System => 2,
                        AppKind::Critical => 3,
                    };
                    let pb = match self.kind(b) {
                        AppKind::User => 0,
                        AppKind::DeviceAdmin => 1,
                        AppKind::System => 2,
                        AppKind::Critical => 3,
                    };
                    pa.cmp(&pb).then_with(|| a.cmp(b))
                });
            }
        }
    }

    pub fn load_available(&mut self) {
        let Some(_serial) = self.serial.clone() else {
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        if !self.apps.app_cache_loaded {
            self.ensure_app_cache();
            self.status = i18n::tr("files.loading");
            return;
        }
        self.ensure_app_cache();
        self.apps.package_kinds.clear();
        let _ = self.ensure_safety();
        let pkgs = self.apps.app_cache_available.clone();
        for pkg in &pkgs {
            let path = self.apps.package_paths.get(pkg);
            let k = safety::classify(
                pkg,
                &self.apps.system_packages,
                &self.apps.device_admins,
                path.map(|s| s.as_str()),
            );
            let k = if matches!(k, AppKind::User) {
                AppKind::System
            } else {
                k
            };
            self.apps.package_kinds.insert(pkg.clone(), k);
        }
        self.apps.available_packages = pkgs;
        self.apps.app_state.select(Some(0));
        self.invalidate_list_cache();
        self.status = rust_i18n::t!(
            "apps.status.available",
            count = self.apps.available_packages.len()
        )
        .to_string();
    }

    fn available_filtered(&self) -> Vec<String> {
        let mut result = if self.apps.filter.is_empty() {
            self.apps.available_packages.clone()
        } else {
            let f = self.apps.filter.to_lowercase();
            self.apps
                .available_packages
                .iter()
                .filter(|p| p.to_lowercase().contains(&f))
                .cloned()
                .collect()
        };
        self.sort_list(&mut result);
        result
    }

    fn filtered_packages(&self) -> Vec<String> {
        let mut result = if self.apps.filter.is_empty() {
            self.apps.packages.clone()
        } else {
            let f = self.apps.filter.to_lowercase();
            self.apps
                .packages
                .iter()
                .filter(|p| p.to_lowercase().contains(&f))
                .cloned()
                .collect()
        };
        self.sort_list(&mut result);
        result
    }

    pub fn load_disabled_packages(&mut self) {
        let Some(_serial) = self.serial.clone() else {
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        if !self.apps.app_cache_loaded {
            self.ensure_app_cache();
            self.status = i18n::tr("files.loading");
            return;
        }
        self.ensure_app_cache();
        self.apps.package_kinds.clear();
        let _ = self.ensure_safety();
        let pkgs = self.apps.app_cache_disabled.clone();
        for pkg in &pkgs {
            let path = self.apps.package_paths.get(pkg);
            let k = safety::classify(
                pkg,
                &self.apps.system_packages,
                &self.apps.device_admins,
                path.map(|s| s.as_str()),
            );
            self.apps.package_kinds.insert(pkg.clone(), k);
        }
        self.apps.packages = pkgs;
        self.apps.selected.clear();
        self.apps.filter.clear();
        self.apps.app_state.select(Some(0));
        self.invalidate_list_cache();
        self.status =
            rust_i18n::t!("apps.status.packages", count = self.apps.packages.len()).to_string();
    }

    pub fn load_packages(&mut self, third_party: bool) {
        let Some(_serial) = self.serial.clone() else {
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        if !self.apps.app_cache_loaded {
            self.ensure_app_cache();
            self.status = i18n::tr("files.loading");
            return;
        }
        self.ensure_app_cache();
        self.apps.package_kinds.clear();
        let _ = self.ensure_safety();
        let pkgs = if third_party {
            self.apps.app_cache_installed_third.clone()
        } else {
            self.apps.app_cache_installed_all.clone()
        };
        for pkg in &pkgs {
            let path = self.apps.package_paths.get(pkg);
            let k = safety::classify(
                pkg,
                &self.apps.system_packages,
                &self.apps.device_admins,
                path.map(|s| s.as_str()),
            );
            self.apps.package_kinds.insert(pkg.clone(), k);
        }
        self.apps.packages = pkgs;
        self.apps.selected.clear();
        self.apps.filter.clear();
        self.apps.app_state.select(Some(0));
        self.invalidate_list_cache();
        self.status =
            rust_i18n::t!("apps.status.packages", count = self.apps.packages.len()).to_string();
    }

    pub fn invalidate_list_cache(&mut self) {
        self.apps.list_cache_valid = false;
        self.apps.list_cache_key = None;
    }

    pub fn current_list(&mut self) -> &Vec<String> {
        if !self.list_cache_matches(self.apps.app_action, self.apps.app_view) {
            let computed = match self.apps.app_view {
                AppView::Installed => self.filtered_packages(),
                AppView::Available => self.available_filtered(),
            };
            self.apps.cached_list = computed;
            self.apps.list_cache_valid = true;
            self.apps.list_cache_key = Some(self.list_cache_context());
        }
        &self.apps.cached_list
    }

    pub fn toggle_selected(&mut self) {
        let idx = self.apps.app_state.selected().unwrap_or(0);
        let filtered = self.current_list();
        if filtered.is_empty() {
            return;
        }
        if idx >= filtered.len() {
            return;
        }
        let pkg = filtered[idx].clone();
        let kind = self.kind(&pkg);
        if matches!(kind, AppKind::Critical) && !self.apps.force_mode {
            self.status = rust_i18n::t!("apps.status.critical_force", pkg = pkg).to_string();
            return;
        }
        if self.apps.selected.contains(&pkg) {
            self.apps.selected.remove(&pkg);
        } else {
            self.apps.selected.insert(pkg);
        }
    }

    pub fn uninstall_selected(&mut self) {
        if self.apps.app_view != AppView::Installed {
            return;
        }
        let Some(serial) = self.serial.clone() else {
            return;
        };
        self.apps.app_cache_loaded = false;
        let to_remove: Vec<String> = self.apps.selected.drain().collect();
        if to_remove.is_empty() {
            self.status = i18n::tr("apps.status.nothing_selected");
            return;
        }
        // block critical unless force
        let has_critical = to_remove
            .iter()
            .any(|p| matches!(self.kind(p), AppKind::Critical));
        if has_critical && !self.apps.force_mode {
            self.status = i18n::tr("apps.status.critical_selected");
            // restore selection
            for p in to_remove {
                self.apps.selected.insert(p);
            }
            return;
        }
        let mut ok = 0;
        let mut fail = 0;
        for pkg in &to_remove {
            let kind = self.kind(pkg);
            let sys = matches!(kind, AppKind::System | AppKind::Critical);
            let keep = self.apps.uninstall_keep_data;
            match crate::package::uninstall(&self.adb, &serial, pkg, keep, sys) {
                Ok(_) => ok += 1,
                Err(_) => fail += 1,
            }
        }
        self.status = rust_i18n::t!("apps.status.uninstalled", ok = ok, fail = fail).to_string();
        self.load_packages(false);
    }

    pub fn install_selected(&mut self) {
        if self.apps.app_view != AppView::Available {
            return;
        }
        let Some(serial) = self.serial.clone() else {
            return;
        };
        self.apps.app_cache_loaded = false;
        let to_install: Vec<String> = self.apps.selected.drain().collect();
        if to_install.is_empty() {
            self.status = i18n::tr("apps.status.nothing_selected");
            return;
        }
        let mut ok = 0;
        let mut fail = 0;
        for pkg in &to_install {
            match crate::package::install_existing(&self.adb, &serial, pkg) {
                Ok(_) => ok += 1,
                Err(_) => fail += 1,
            }
        }
        self.status = rust_i18n::t!("apps.status.installed", ok = ok, fail = fail).to_string();
        self.load_available();
    }

    pub fn enable_selected(&mut self) {
        if let Some(serial) = self.serial.clone() {
            self.apps.app_cache_loaded = false;
            let to_enable: Vec<String> = self.apps.selected.drain().collect();
            if to_enable.is_empty() {
                self.status = i18n::tr("apps.status.nothing_selected");
                return;
            }
            let mut ok = 0;
            let mut fail = 0;
            for pkg in &to_enable {
                match crate::package::enable(&self.adb, &serial, pkg) {
                    Ok(_) => ok += 1,
                    Err(_) => fail += 1,
                }
            }
            self.status = rust_i18n::t!("apps.status.enabled", ok = ok, fail = fail).to_string();
            self.load_disabled_packages();
        }
    }

    pub fn clear_cache_selected(&mut self) {
        if self.apps.app_view != AppView::Installed {
            return;
        }
        let Some(serial) = self.serial.clone() else {
            return;
        };
        self.apps.app_cache_loaded = false;
        let to_clear: Vec<String> = self.apps.selected.drain().collect();
        if to_clear.is_empty() {
            self.status = i18n::tr("apps.status.nothing_selected");
            return;
        }
        let has_critical = to_clear
            .iter()
            .any(|p| matches!(self.kind(p), AppKind::Critical));
        if has_critical && !self.apps.force_mode {
            self.status = i18n::tr("apps.status.critical_selected");
            for p in to_clear {
                self.apps.selected.insert(p);
            }
            return;
        }
        let mut ok = 0;
        let mut fail = 0;
        for pkg in &to_clear {
            match crate::package::clear_cache(&self.adb, &serial, pkg) {
                Ok(_) => ok += 1,
                Err(_) => fail += 1,
            }
        }
        self.status = rust_i18n::t!("apps.status.cache_cleared", ok = ok, fail = fail).to_string();
        self.load_packages(false);
    }

    pub fn force_stop_selected(&mut self) {
        if self.apps.app_view != AppView::Installed {
            return;
        }
        let Some(serial) = self.serial.clone() else {
            return;
        };
        self.apps.app_cache_loaded = false;
        let to_stop: Vec<String> = self.apps.selected.drain().collect();
        if to_stop.is_empty() {
            self.status = i18n::tr("apps.status.nothing_selected");
            return;
        }
        let has_critical = to_stop
            .iter()
            .any(|p| matches!(self.kind(p), AppKind::Critical));
        if has_critical && !self.apps.force_mode {
            self.status = i18n::tr("apps.status.critical_selected");
            for p in to_stop {
                self.apps.selected.insert(p);
            }
            return;
        }
        let mut ok = 0;
        let mut fail = 0;
        for pkg in &to_stop {
            match crate::package::force_stop(&self.adb, &serial, pkg) {
                Ok(_) => ok += 1,
                Err(_) => fail += 1,
            }
        }
        self.status = rust_i18n::t!("apps.status.force_stopped", ok = ok, fail = fail).to_string();
        self.load_packages(false);
    }

    pub fn clear_data_selected(&mut self) {
        if self.apps.app_view != AppView::Installed {
            return;
        }
        let Some(serial) = self.serial.clone() else {
            return;
        };
        self.apps.app_cache_loaded = false;
        let to_clear: Vec<String> = self.apps.selected.drain().collect();
        if to_clear.is_empty() {
            self.status = i18n::tr("apps.status.nothing_selected");
            return;
        }
        let has_critical = to_clear
            .iter()
            .any(|p| matches!(self.kind(p), AppKind::Critical));
        if has_critical && !self.apps.force_mode {
            self.status = i18n::tr("apps.status.critical_selected");
            for p in to_clear {
                self.apps.selected.insert(p);
            }
            return;
        }
        let mut ok = 0;
        let mut fail = 0;
        for pkg in &to_clear {
            match crate::package::clear_data(&self.adb, &serial, pkg) {
                Ok(_) => ok += 1,
                Err(_) => fail += 1,
            }
        }
        self.status = rust_i18n::t!("apps.status.data_cleared", ok = ok, fail = fail).to_string();
        self.load_packages(false);
    }

    pub fn load_device_info(&mut self) {
        self.device_info.device_info_scroll = 0;
        let Some(serial) = self.serial.clone() else {
            self.device_info.device_info_lines = vec![Line::from(i18n::tr("info.no_device"))];
            self.status = i18n::tr("app.status.no_device");
            return;
        };
        if self.device_info.device_info_loading {
            return;
        }
        self.device_info.device_info_loading = true;
        self.status = i18n::tr("info.loading");
        let adb = self.adb.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.device_info.device_info_rx = Some(rx);
        std::thread::spawn(move || {
            let lines = Self::build_device_info_lines(&adb, &serial);
            let _ = tx.send(lines);
        });
    }

    pub fn poll_device_info(&mut self) {
        let Some(rx) = self.device_info.device_info_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(lines) => {
                self.device_info.device_info_rx = None;
                self.device_info.device_info_loading = false;
                self.device_info.device_info_lines = lines;
                self.status = i18n::tr("info.status.help");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.device_info.device_info_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.device_info.device_info_rx = None;
                self.device_info.device_info_loading = false;
                self.status = i18n::tr("app.status.no_device");
            }
        }
    }

    fn build_device_info_lines(adb: &str, serial: &str) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Device / build props
        lines.push(section_line(i18n::tr("info.section.device")));
        match crate::adb::getprop(adb, serial) {
            Ok(text) => {
                let props = crate::adb::parse_getprop(&text);
                let keys = [
                    ("ro.product.manufacturer", "info.label.manufacturer"),
                    ("ro.product.brand", "info.label.brand"),
                    ("ro.product.model", "info.label.model"),
                    ("ro.product.device", "info.label.device"),
                    ("ro.build.id", "info.label.build_id"),
                    ("ro.build.version.release", "info.label.android"),
                    ("ro.build.version.sdk", "info.label.sdk"),
                    (
                        "ro.build.version.security_patch",
                        "info.label.security_patch",
                    ),
                    ("ro.serialno", "info.label.serial"),
                ];
                for (key, label_key) in keys {
                    let val = props.get(key).cloned().unwrap_or_else(|| "-".to_string());
                    lines.push(entry_line(&i18n::tr(label_key), val));
                }
            }
            Err(e) => lines.push(Line::from(
                rust_i18n::t!("info.error.getprop", error = e).to_string(),
            )),
        }

        // Battery
        lines.push(Line::from(""));
        lines.push(section_line(i18n::tr("info.section.battery")));
        match crate::adb::dumpsys_battery(adb, serial) {
            Ok(text) => {
                let b = crate::adb::parse_battery(&text);
                let level = b.level.unwrap_or_else(|| "-".to_string());
                let scale = "100";
                lines.push(entry_line(
                    &i18n::tr("info.label.level"),
                    format!("{level}/{scale}"),
                ));
                let status_text = match b.status.as_deref() {
                    Some("1") => i18n::tr("info.battery.status.unknown"),
                    Some("2") => i18n::tr("info.battery.status.charging"),
                    Some("3") => i18n::tr("info.battery.status.discharging"),
                    Some("4") => i18n::tr("info.battery.status.not_charging"),
                    Some("5") => i18n::tr("info.battery.status.full"),
                    Some(other) => other.to_string(),
                    None => "-".to_string(),
                };
                lines.push(entry_line(&i18n::tr("info.label.status"), status_text));
                lines.push(entry_line(
                    &i18n::tr("info.label.usb_powered"),
                    b.usb_powered.unwrap_or_else(|| "-".to_string()),
                ));
                let voltage = b
                    .voltage
                    .as_deref()
                    .and_then(|v| v.parse::<i64>().ok())
                    .map(|mv| format!("{:.3} V", mv as f64 / 1000.0))
                    .unwrap_or_else(|| b.voltage.clone().unwrap_or_else(|| "-".to_string()));
                lines.push(entry_line(&i18n::tr("info.label.voltage"), voltage));
                let temp = b
                    .temperature
                    .as_deref()
                    .and_then(|t| t.parse::<i64>().ok())
                    .map(|t| format!("{:.1} °C", t as f64 / 10.0))
                    .unwrap_or_else(|| b.temperature.clone().unwrap_or_else(|| "-".to_string()));
                lines.push(entry_line(&i18n::tr("info.label.temperature"), temp));
            }
            Err(e) => lines.push(Line::from(
                rust_i18n::t!("info.error.battery", error = e).to_string(),
            )),
        }

        // Storage
        lines.push(Line::from(""));
        lines.push(section_line(i18n::tr("info.section.storage")));
        match crate::adb::df(adb, serial) {
            Ok(text) => {
                let entries = crate::adb::parse_df(&text);
                let targets = ["/data", "/system", "/vendor", "/product"];
                let mut shown = 0;
                for mount in targets {
                    if let Some(e) = entries.iter().find(|e| e.mount == mount) {
                        let total = e.total_kb as f64 / 1024.0 / 1024.0;
                        let used = e.used_kb as f64 / 1024.0 / 1024.0;
                        let avail = e.available_kb as f64 / 1024.0 / 1024.0;
                        lines.push(entry_line(mount, storage_usage(total, used, avail)));
                        shown += 1;
                    }
                }
                if shown == 0 {
                    // fallback: show /data if any
                    if let Some(e) = entries.iter().find(|e| e.mount == "/data") {
                        let total = e.total_kb as f64 / 1024.0 / 1024.0;
                        let used = e.used_kb as f64 / 1024.0 / 1024.0;
                        let avail = e.available_kb as f64 / 1024.0 / 1024.0;
                        lines.push(entry_line("/data", storage_usage(total, used, avail)));
                    } else {
                        lines.push(Line::from(i18n::tr("info.storage.parse_error")));
                    }
                }
            }
            Err(e) => lines.push(Line::from(
                rust_i18n::t!("info.error.df", error = e).to_string(),
            )),
        }

        // SD / external storage
        lines.push(Line::from(""));
        lines.push(section_line(i18n::tr("info.section.sd")));
        match crate::adb::sm_list_volumes(adb, serial) {
            Ok(text) => {
                let vols = crate::adb::parse_sm_volumes(&text);
                let mut shown = 0;
                for v in vols {
                    if !v.id.starts_with("public:") || v.state != "mounted" {
                        continue;
                    }
                    let Some(uuid) = v.uuid else {
                        continue;
                    };
                    if uuid == "null" {
                        continue;
                    }
                    let mut parsed_entry: Option<crate::adb::DfEntry> = None;
                    for path in [format!("/storage/{uuid}"), format!("/mnt/media_rw/{uuid}")] {
                        if let Ok(df_text) = crate::adb::df_path(adb, serial, &path) {
                            let entries = crate::adb::parse_df(&df_text);
                            if let Some(e) = entries.iter().find(|e| e.mount == path) {
                                parsed_entry = Some(e.clone());
                                break;
                            }
                            if parsed_entry.is_none() && !entries.is_empty() {
                                // fallback to last entry if mount path differs
                                parsed_entry = entries.last().cloned();
                            }
                        }
                    }
                    if let Some(e) = parsed_entry {
                        let total = e.total_kb as f64 / 1024.0 / 1024.0;
                        let used = e.used_kb as f64 / 1024.0 / 1024.0;
                        let avail = e.available_kb as f64 / 1024.0 / 1024.0;
                        let pct = if e.total_kb > 0 {
                            e.used_kb as f64 / e.total_kb as f64 * 100.0
                        } else {
                            0.0
                        };
                        lines.push(entry_line(
                            &uuid,
                            storage_usage_pct(total, used, avail, pct),
                        ));
                        shown += 1;
                    } else {
                        lines.push(entry_line(&uuid, i18n::tr("info.sd.parse_error")));
                    }
                }
                if shown == 0 {
                    lines.push(Line::from(i18n::tr("info.sd.none")));
                }
            }
            Err(e) => lines.push(Line::from(
                rust_i18n::t!("info.error.sm", error = e).to_string(),
            )),
        }

        lines
    }

    pub fn move_list(state: &mut ListState, len: usize, delta: isize) {
        if len == 0 {
            return;
        }
        let cur = state.selected().unwrap_or(0) as isize;
        let mut next = cur + delta;
        if next < 0 {
            next = len as isize - 1;
        }
        if next >= len as isize {
            next = 0;
        }
        state.select(Some(next as usize));
    }
}

fn section_line(title: String) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("╭─ {title} ─────────────╮"),
        Theme::section(),
    )])
}

fn storage_usage(total: f64, used: f64, avail: f64) -> String {
    rust_i18n::t!(
        "info.storage.usage",
        total = format!("{total:.2}"),
        used = format!("{used:.2}"),
        avail = format!("{avail:.2}")
    )
    .to_string()
}

fn storage_usage_pct(total: f64, used: f64, avail: f64, pct: f64) -> String {
    rust_i18n::t!(
        "info.storage.usage_pct",
        total = format!("{total:.2}"),
        used = format!("{used:.2}"),
        avail = format!("{avail:.2}"),
        pct = format!("{pct:.0}")
    )
    .to_string()
}

fn entry_line(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:<18}", format!("{label}:")),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(value, Theme::normal()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_on_main_menu() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert_eq!(app.screen, Screen::MainMenu);
    }

    #[test]
    fn device_menu_includes_shell() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert!(app.device_menu_keys.contains(&"device.menu.shell"));
    }

    #[test]
    fn device_menu_includes_control() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert!(app.device_menu_keys.contains(&"device.menu.control"));
    }

    #[test]
    fn open_device_control_sets_screen() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.open_device_control();
        assert_eq!(app.screen, Screen::DeviceControl);
        assert!(app.device_control_pending.is_none());
        assert_eq!(app.device_control_state.selected(), Some(0));
    }

    #[test]
    fn request_direct_shell_sets_flag() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.serial = Some("serial".to_string());
        app.request_direct_shell();
        assert!(app.direct_shell_requested);
    }

    #[test]
    fn device_menu_includes_network() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert!(app.device_menu_keys.contains(&"device.menu.network"));
    }

    #[test]
    fn open_network_sets_screen() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.open_network();
        assert_eq!(app.screen, Screen::Network);
    }

    #[test]
    fn wifi_state_defaults() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert!(app.network.wifi_discovered.is_empty());
        assert!(app.network.wifi_connect_input.is_empty());
        assert!(!app.network.wifi_connect_input_mode);
        assert!(app.network.wifi_connect_result.is_none());
    }

    #[test]
    fn refresh_network_menu_disconnected_keys() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.network.network_endpoint = None;
        app.refresh_network_menu();
        assert_eq!(
            app.network.network_menu_keys,
            vec![
                "network.menu.info",
                "network.menu.wifi_on",
                "network.menu.data_on",
                "network.menu.airplane_on",
                "network.menu.enable",
                "network.menu.connect",
                "network.menu.set_subnet",
                "network.menu.back",
            ]
        );
        assert_eq!(app.network.network_menu_state.selected(), Some(0));
    }

    #[test]
    fn refresh_network_menu_connected_keys() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.network.network_endpoint = Some("192.168.1.5:5555".to_string());
        app.refresh_network_menu();
        assert_eq!(
            app.network.network_menu_keys,
            vec![
                "network.menu.info",
                "network.menu.wifi_on",
                "network.menu.data_on",
                "network.menu.airplane_on",
                "network.menu.disconnect",
                "network.menu.disable",
                "network.menu.set_subnet",
                "network.menu.back",
            ]
        );
        assert_eq!(app.network.network_menu_state.selected(), Some(0));
    }

    #[test]
    fn main_menu_order_excludes_scripts() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert_eq!(
            app.menu_keys,
            vec!["menu.main.devices", "menu.main.settings", "menu.main.exit",]
        );
    }

    #[test]
    fn device_menu_order_is_exact() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert_eq!(
            app.device_menu_keys,
            vec![
                "device.menu.apps",
                "device.menu.scripts",
                "device.menu.logcat",
                "device.menu.processes",
                "device.menu.info",
                "device.menu.shell",
                "device.menu.control",
                "device.menu.network",
                "device.menu.storage",
                "device.menu.back",
            ]
        );
    }

    #[test]
    fn logcat_ring_buffer_keeps_latest_lines() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        for index in 0..(logcat::MAX_LINES + 3) {
            app.push_logcat_line(format!("line {index}"));
        }
        assert_eq!(app.logcat.logcat_lines.len(), logcat::MAX_LINES);
        assert_eq!(
            app.logcat.logcat_lines.front().map(String::as_str),
            Some("line 3")
        );
        assert_eq!(
            app.logcat.logcat_state.selected(),
            Some(logcat::MAX_LINES.saturating_sub(1))
        );
    }

    #[test]
    fn logcat_pause_freezes_selection_and_resume_jumps_to_bottom() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.push_logcat_line("first".to_string());
        app.toggle_logcat_pause();
        app.push_logcat_line("second".to_string());
        assert_eq!(app.logcat.logcat_state.selected(), Some(0));
        app.toggle_logcat_pause();
        assert_eq!(app.logcat.logcat_state.selected(), Some(1));
    }

    #[test]
    fn open_scripts_sets_screen() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.open_scripts();
        assert_eq!(app.screen, Screen::Scripts);
        assert_eq!(app.scripts.scripts_menu_state.selected(), Some(0));
    }

    #[test]
    fn open_script_folders_starts_unchecked() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.scripts.script_selected.insert(ScriptFolder::Pictures);
        app.open_script_folders();
        assert_eq!(app.screen, Screen::ScriptFolders);
        assert_eq!(app.scripts.script_folder_state.selected(), Some(0));
        assert!(app.scripts.script_selected.is_empty());
    }

    #[test]
    fn space_toggles_script_folder() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.open_script_folders();
        app.toggle_script_folder();
        assert!(app.scripts.script_selected.contains(&ScriptFolder::Dcim));
        app.toggle_script_folder();
        assert!(!app.scripts.script_selected.contains(&ScriptFolder::Dcim));
    }

    #[test]
    fn start_without_folders_is_blocked() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.serial = Some("serial".to_string());
        app.screen = Screen::ScriptFolders;
        app.start_el_negro_samuel();
        assert!(!app.scripts.script_running);
        assert!(app.scripts.script_rx.is_none());
        assert_eq!(app.screen, Screen::ScriptFolders);
        assert!(!app.status.is_empty());
    }

    #[test]
    fn start_without_device_is_blocked() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.screen = Screen::ScriptFolders;
        app.scripts.script_selected.insert(ScriptFolder::Dcim);
        app.start_el_negro_samuel();
        assert!(!app.scripts.script_running);
        assert!(app.scripts.script_rx.is_none());
        assert_eq!(app.screen, Screen::ScriptFolders);
        assert!(!app.status.is_empty());
    }

    #[test]
    fn starting_script_returns_to_device_menu() {
        let mut app = App::new(
            "/nonexistent/alejandradb-test-adb".to_string(),
            None,
            Lang::English,
        );
        app.serial = Some("serial".to_string());
        app.scripts.script_selected.insert(ScriptFolder::Dcim);
        app.screen = Screen::ScriptFolders;
        app.start_el_negro_samuel();
        assert!(app.scripts.script_running);
        assert!(app.scripts.script_rx.is_some());
        assert_eq!(app.screen, Screen::DeviceMenu);
    }

    #[test]
    fn scripts_do_not_auto_select_a_device() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.devices = vec![Device {
            serial: "serial".to_string(),
            state: "device".to_string(),
            manufacturer: "Test".to_string(),
            model: "Phone".to_string(),
        }];
        app.screen = Screen::ScriptFolders;
        app.scripts.script_selected.insert(ScriptFolder::Dcim);
        app.start_el_negro_samuel();
        assert!(!app.scripts.script_running);
        assert!(app.scripts.script_rx.is_none());
        assert!(app.serial.is_none());
    }

    #[test]
    fn second_script_start_is_blocked_while_running() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.scripts.script_running = true;
        app.serial = Some("serial".to_string());
        app.scripts.script_selected.insert(ScriptFolder::Dcim);
        app.start_el_negro_samuel();
        assert!(app.scripts.script_running);
        assert!(app.scripts.script_rx.is_none());
        assert!(!app.status.is_empty());
    }

    #[test]
    fn polling_updates_status_and_finishes() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        let (sender, receiver) = std::sync::mpsc::channel();
        app.scripts.script_rx = Some(receiver);
        app.scripts.script_running = true;

        sender
            .send(ScriptEvent::Status("copying".to_string()))
            .unwrap();
        app.poll_script_events();
        assert_eq!(app.status, "copying");
        assert!(app.scripts.script_running);

        sender
            .send(ScriptEvent::Finished(scripts::ScriptSummary {
                copied: 1,
                skipped: 0,
                failed: 0,
                destination: PathBuf::from("/tmp/output/20260101-010203"),
            }))
            .unwrap();
        app.poll_script_events();
        assert!(!app.scripts.script_running);
        assert!(app.scripts.script_rx.is_none());
        assert!(app.status.contains('1'));
        assert_eq!(
            app.scripts.script_last_destination,
            Some(PathBuf::from("/tmp/output/20260101-010203"))
        );
    }

    #[test]
    fn polling_failure_clears_running_state() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        let (sender, receiver) = std::sync::mpsc::channel();
        app.scripts.script_rx = Some(receiver);
        app.scripts.script_running = true;

        sender
            .send(ScriptEvent::Failed("storage missing".to_string()))
            .unwrap();
        app.poll_script_events();
        assert!(!app.scripts.script_running);
        assert!(app.scripts.script_rx.is_none());
        assert!(app.status.contains("storage missing"));
    }

    #[test]
    fn filtered_process_indices_returns_all_when_filter_empty() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.processes.processes = vec![
            ProcessInfo {
                user: "root".to_string(),
                pid: 1,
                ppid: 0,
                name: "/init".to_string(),
            },
            ProcessInfo {
                user: "root".to_string(),
                pid: 8,
                ppid: 2,
                name: "kworker/0:0H".to_string(),
            },
        ];
        assert_eq!(app.filtered_process_indices(), vec![0, 1]);
    }

    #[test]
    fn filtered_process_indices_filters_by_name() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.processes.process_filter = "kworker".to_string();
        app.processes.processes = vec![
            ProcessInfo {
                user: "root".to_string(),
                pid: 1,
                ppid: 0,
                name: "/init".to_string(),
            },
            ProcessInfo {
                user: "root".to_string(),
                pid: 8,
                ppid: 2,
                name: "kworker/0:0H".to_string(),
            },
        ];
        assert_eq!(app.filtered_process_indices(), vec![1]);
    }

    #[test]
    fn filtered_process_indices_maps_selection_to_process() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.processes.process_filter = "8".to_string();
        app.processes.processes = vec![
            ProcessInfo {
                user: "root".to_string(),
                pid: 1,
                ppid: 0,
                name: "/init".to_string(),
            },
            ProcessInfo {
                user: "root".to_string(),
                pid: 8,
                ppid: 2,
                name: "kworker/0:0H".to_string(),
            },
        ];
        app.processes.process_state.select(Some(0));
        app.request_process_kill();
        assert!(app.processes.process_pending.is_some());
        let pending = app.processes.process_pending.clone().unwrap();
        assert_eq!(pending.pid, 8);
    }

    #[test]
    fn settings_menu_includes_output_path() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert_eq!(
            app.settings_menu_keys,
            vec!["settings.language", "settings.output_path", "settings.back"]
        );
    }

    #[test]
    fn open_output_path_sets_screen_and_input() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.open_output_path();
        assert_eq!(app.screen, Screen::OutputPath);
        assert_eq!(
            app.output_path_input,
            crate::config::Config::load().output_path
        );
        assert!(app.output_path_input_mode);
    }

    #[test]
    fn app_cache_not_loaded_without_serial() {
        let app = App::new("adb".to_string(), None, Lang::English);
        assert!(!app.apps.app_cache_loaded);
        assert!(app.apps.app_cache_serial.is_none());
    }

    #[test]
    fn reset_device_state_clears_app_cache() {
        let mut app = App::new("adb".to_string(), None, Lang::English);
        app.apps.app_cache_loaded = true;
        app.apps.app_cache_serial = Some("serial".to_string());
        app.reset_device_state();
        assert!(!app.apps.app_cache_loaded);
        assert!(app.apps.app_cache_serial.is_none());
    }
}
