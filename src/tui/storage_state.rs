use crate::adb::RemoteEntry;
use std::sync::mpsc::Receiver;

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct StorageState {
    pub file_browser_path: String,
    pub file_browser_entries: Vec<RemoteEntry>,
    pub file_browser_state: ListState,
    pub file_browser_loading: bool,
    pub file_browser_rx: Option<Receiver<Result<Vec<RemoteEntry>, String>>>,
    pub file_browser_filter: String,
    pub file_browser_input_mode: bool,
    pub file_browser_input: String,
    pub file_browser_pending_delete: Option<String>,
    pub file_browser_clipboard: Option<(String, bool)>,
    pub storage_usage: Vec<(String, u64)>,
    pub storage_usage_rx: Option<Receiver<Vec<(String, u64)>>>,
    pub storage_usage_loading: bool,
    pub storage_volumes: Vec<String>,
    pub storage_volumes_rx: Option<Receiver<Vec<String>>>,
    pub storage_volumes_loading: bool,
    pub storage_menu_keys: Vec<&'static str>,
    pub storage_menu_state: ListState,
}

impl Default for StorageState {
    fn default() -> Self {
        let mut file_browser_state = ListState::default();
        file_browser_state.select(Some(0));
        let mut storage_menu_state = ListState::default();
        storage_menu_state.select(Some(0));
        Self {
            file_browser_path: "/sdcard".to_string(),
            file_browser_entries: Vec::new(),
            file_browser_state,
            file_browser_loading: false,
            file_browser_rx: None,
            file_browser_filter: String::new(),
            file_browser_input_mode: false,
            file_browser_input: String::new(),
            file_browser_pending_delete: None,
            file_browser_clipboard: None,
            storage_usage: Vec::new(),
            storage_usage_rx: None,
            storage_usage_loading: false,
            storage_volumes: Vec::new(),
            storage_volumes_rx: None,
            storage_volumes_loading: false,
            storage_menu_keys: vec![
                "storage.menu.files",
                "storage.menu.usage",
                "storage.menu.volumes",
                "storage.menu.clean",
                "storage.menu.back",
            ],
            storage_menu_state,
        }
    }
}
