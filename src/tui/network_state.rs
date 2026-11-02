use crate::adb::NetworkInfo;
use crate::tui::app::{ApkResult, NetworkAction};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use ratatui::widgets::ListState;

#[derive(Debug)]
#[allow(clippy::type_complexity)]
pub struct NetworkState {
    pub network_menu_keys: Vec<&'static str>,
    pub network_menu_state: ListState,
    pub network_ip: Option<String>,
    pub network_ssid: String,
    pub network_wifi_state: String,
    pub wifi_enabled: bool,
    pub mobile_data_enabled: bool,
    pub airplane_mode: bool,
    pub network_info_rx: Option<Receiver<(Option<NetworkInfo>, Vec<String>, String)>>,
    pub network_info_loading: bool,
    pub network_info_cached_at: Option<std::time::Instant>,
    pub network_action_rx: Option<Receiver<Result<String, String>>>,
    pub network_action_loading: bool,
    pub network_action_kind: Option<String>,
    pub network_loading: bool,
    pub network_result: Option<ApkResult>,
    pub network_input: String,
    pub network_input_mode: bool,
    pub network_pending: Option<NetworkAction>,
    pub network_pending_endpoint: Option<String>,
    pub network_endpoint: Option<String>,
    pub wifi_discovered: Vec<String>,
    pub wifi_discover_state: ListState,
    pub wifi_connect_input: String,
    pub wifi_connect_input_mode: bool,
    pub wifi_connect_result: Option<ApkResult>,
    pub wifi_scanning: bool,
    pub wifi_scan_rx: Option<Receiver<crate::adb::WifiScanEntry>>,
    pub wifi_scan_results: Vec<String>,
    pub wifi_rejected: Vec<String>,
    pub wifi_show_rejected: bool,
    pub wifi_subnet_input: String,
    pub wifi_subnet_input_mode: bool,
    pub wifi_scan_start: Option<Instant>,
    pub wifi_scan_last_at: Option<Instant>,
    pub wifi_scan_progress: usize,
}

impl Default for NetworkState {
    fn default() -> Self {
        let mut network_menu_state = ListState::default();
        network_menu_state.select(Some(0));
        let mut wifi_discover_state = ListState::default();
        wifi_discover_state.select(Some(0));
        Self {
            network_menu_keys: vec![
                "network.menu.info",
                "network.menu.enable",
                "network.menu.connect",
                "network.menu.disconnect",
                "network.menu.disable",
                "network.menu.scan",
                "network.menu.back",
            ],
            network_menu_state,
            network_ip: None,
            network_ssid: String::new(),
            network_wifi_state: String::new(),
            wifi_enabled: false,
            mobile_data_enabled: false,
            airplane_mode: false,
            network_info_rx: None,
            network_info_loading: false,
            network_info_cached_at: None,
            network_action_rx: None,
            network_action_loading: false,
            network_action_kind: None,
            network_loading: false,
            network_result: None,
            network_input: String::new(),
            network_input_mode: false,
            network_pending: None,
            network_pending_endpoint: None,
            network_endpoint: None,
            wifi_discovered: Vec::new(),
            wifi_discover_state,
            wifi_connect_input: String::new(),
            wifi_connect_input_mode: false,
            wifi_connect_result: None,
            wifi_scanning: false,
            wifi_scan_rx: None,
            wifi_scan_results: Vec::new(),
            wifi_rejected: Vec::new(),
            wifi_show_rejected: false,
            wifi_subnet_input: String::new(),
            wifi_subnet_input_mode: false,
            wifi_scan_start: None,
            wifi_scan_last_at: None,
            wifi_scan_progress: 0,
        }
    }
}
