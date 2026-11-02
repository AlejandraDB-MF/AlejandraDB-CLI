use ratatui::text::Line;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct DeviceInfoState {
    pub device_info_lines: Vec<Line<'static>>,
    pub device_info_scroll: u16,
    pub device_info_loading: bool,
    pub device_info_rx: Option<Receiver<Vec<Line<'static>>>>,
}

#[allow(clippy::derivable_impls)]
impl Default for DeviceInfoState {
    fn default() -> Self {
        Self {
            device_info_lines: Vec::new(),
            device_info_scroll: 0,
            device_info_loading: false,
            device_info_rx: None,
        }
    }
}
