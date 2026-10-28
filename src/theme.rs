use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn selected() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }
    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
    pub fn section() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
}
