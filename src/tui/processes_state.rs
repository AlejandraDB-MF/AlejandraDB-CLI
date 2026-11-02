use crate::processes::ProcessInfo;
use std::sync::mpsc::Receiver;

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ProcessesState {
    pub processes: Vec<ProcessInfo>,
    pub process_state: ListState,
    pub process_pending: Option<ProcessInfo>,
    pub process_filter: String,
    pub process_filter_mode: bool,
    pub processes_rx: Option<Receiver<std::result::Result<Vec<ProcessInfo>, String>>>,
    pub processes_loading: bool,
}

impl Default for ProcessesState {
    fn default() -> Self {
        let mut process_state = ListState::default();
        process_state.select(Some(0));
        Self {
            processes: Vec::new(),
            process_state,
            process_pending: None,
            process_filter: String::new(),
            process_filter_mode: false,
            processes_rx: None,
            processes_loading: false,
        }
    }
}
