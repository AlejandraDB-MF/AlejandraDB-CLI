use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn selected() -> Style {
        Style::default()
            .fg(Color::LightGreen)
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
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    }
    pub fn section() -> Style {
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    }
    pub fn danger() -> Style {
        Style::default().fg(Color::Red)
    }
    pub fn warning() -> Style {
        Style::default().fg(Color::Yellow)
    }
    pub fn admin() -> Style {
        Style::default().fg(Color::Magenta)
    }
    pub fn success() -> Style {
        Style::default().fg(Color::Green)
    }
    pub fn muted() -> Style {
        Style::default().fg(Color::Gray)
    }
    pub fn dark_muted() -> Style {
        Style::default().fg(Color::DarkGray)
    }
    pub fn accent() -> Style {
        Style::default().fg(Color::LightGreen)
    }
}
