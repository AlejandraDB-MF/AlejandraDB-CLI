use crate::logcat::LogcatSession;
use std::collections::VecDeque;

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct LogcatState {
    pub logcat_lines: VecDeque<String>,
    pub logcat_state: ListState,
    pub logcat_paused: bool,
    pub logcat_session: Option<LogcatSession>,
}

impl Default for LogcatState {
    fn default() -> Self {
        let mut logcat_state = ListState::default();
        logcat_state.select(Some(0));
        Self {
            logcat_lines: VecDeque::new(),
            logcat_state,
            logcat_paused: false,
            logcat_session: None,
        }
    }
}
