use crate::tui::app::{ApkResult, AppAction, AppCachePayload, AppView, SortMode};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct AppsState {
    pub apps_menu_keys: Vec<&'static str>,
    pub apps_menu_state: ListState,
    pub app_action: AppAction,
    pub packages: Vec<String>,
    pub available_packages: Vec<String>,
    pub package_kinds: HashMap<String, crate::safety::AppKind>,
    pub package_paths: HashMap<String, String>,
    pub app_view: AppView,
    pub sort_mode: SortMode,
    pub apk_input: String,
    pub apk_result: Option<ApkResult>,
    pub filter: String,
    pub filter_mode: bool,
    pub selected: HashSet<String>,
    pub system_packages: HashSet<String>,
    pub device_admins: HashSet<String>,
    pub safety_loaded: bool,
    pub safety_serial: Option<String>,
    pub force_mode: bool,
    pub uninstall_keep_data: bool,
    pub app_cache_serial: Option<String>,
    pub app_cache_loaded: bool,
    pub app_cache_rx: Option<Receiver<AppCachePayload>>,
    pub app_cache_loading: bool,
    pub app_cache_just_loaded: bool,
    pub app_cache_installed_all: Vec<String>,
    pub app_cache_installed_third: Vec<String>,
    pub app_cache_available: Vec<String>,
    pub app_cache_disabled: Vec<String>,
    pub app_cache_system: HashSet<String>,
    pub app_cache_admins: HashSet<String>,
    pub app_cache_paths: HashMap<String, String>,
    pub app_state: ListState,
    pub cached_list: Vec<String>,
    pub list_cache_valid: bool,
    pub list_cache_key: Option<(AppAction, AppView, Option<String>)>,
}

impl Default for AppsState {
    fn default() -> Self {
        let mut apps_menu_state = ListState::default();
        apps_menu_state.select(Some(0));
        let mut app_state = ListState::default();
        app_state.select(Some(0));
        Self {
            apps_menu_keys: vec![
                "apps.menu.uninstall",
                "apps.menu.install",
                "apps.menu.reinstall",
                "apps.menu.enable",
                "apps.menu.clear_cache",
                "apps.menu.force_stop",
                "apps.menu.clear_data",
            ],
            apps_menu_state,
            app_action: AppAction::Uninstall,
            packages: Vec::new(),
            available_packages: Vec::new(),
            package_kinds: HashMap::new(),
            package_paths: HashMap::new(),
            app_view: AppView::Installed,
            sort_mode: SortMode::NameAsc,
            apk_input: String::new(),
            apk_result: None,
            filter: String::new(),
            filter_mode: false,
            selected: HashSet::new(),
            system_packages: HashSet::new(),
            device_admins: HashSet::new(),
            safety_loaded: false,
            safety_serial: None,
            force_mode: false,
            uninstall_keep_data: false,
            app_cache_serial: None,
            app_cache_loaded: false,
            app_cache_rx: None,
            app_cache_loading: false,
            app_cache_just_loaded: false,
            app_cache_installed_all: Vec::new(),
            app_cache_installed_third: Vec::new(),
            app_cache_available: Vec::new(),
            app_cache_disabled: Vec::new(),
            app_cache_system: HashSet::new(),
            app_cache_admins: HashSet::new(),
            app_cache_paths: HashMap::new(),
            app_state,
            cached_list: Vec::new(),
            list_cache_valid: false,
            list_cache_key: None,
        }
    }
}
