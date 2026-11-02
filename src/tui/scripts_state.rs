use crate::scripts::{ScriptEvent, ScriptFolder};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ScriptsState {
    pub scripts_menu_state: ListState,
    pub script_folder_state: ListState,
    pub script_selected: HashSet<ScriptFolder>,
    pub script_running: bool,
    pub script_rx: Option<Receiver<ScriptEvent>>,
    pub script_last_destination: Option<PathBuf>,
}

impl Default for ScriptsState {
    fn default() -> Self {
        let mut scripts_menu_state = ListState::default();
        scripts_menu_state.select(Some(0));
        let mut script_folder_state = ListState::default();
        script_folder_state.select(Some(0));
        Self {
            scripts_menu_state,
            script_folder_state,
            script_selected: HashSet::new(),
            script_running: false,
            script_rx: None,
            script_last_destination: None,
        }
    }
}
