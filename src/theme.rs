use crate::ui;
use tui::style::{Color as C, Style, Modifier as M};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Theme {
    Default,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Default
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
macro_rules! s {
    ($c:expr) => { Style::default().fg($c) };
    ($c:expr, $m:expr) => { Style::default().fg($c).add_modifier($m) };
}

impl ui::PanelStyles for Theme {
    fn border_active(&self) -> Style { s!(C::Green) }
    fn border_inactive(&self) -> Style { s!(C::White) }
    fn btn(&self) -> Style { Style::default().bg(C::Red) }
}

impl ui::FeStyles for Theme {
    fn mark(&self) -> Style  { s!(C::Red) }
    fn cursor_select(&self) -> Style { Style::default().bg(C::Yellow) }

    fn highlight(&self) -> Style { Style::default().fg(C::White).bg(C::Red) }

    // size
    fn size(&self) -> Style { s!(C::Green) }

    // datetime
    fn datetime(&self) -> Style { s!(C::Blue) }

    // file_type
    fn directory(&self) -> Style { s!(C::Blue) }
    fn dot_dot(&self) -> Style { s!(C::Blue) }
    fn file(&self) -> Style { Style::default() }
    fn sym_link(&self) -> Style { s!(C::Cyan) }
    fn other(&self) -> Style { s!(C::Yellow) }

    // Permissions
    fn dashed(&self) -> Style { Style::default().add_modifier(M::DIM) }
    fn user_read(&self) -> Style { s!(C::Yellow, M::BOLD) }
    fn user_write(&self) -> Style { s!(C::Red, M::BOLD) }
    fn user_execute(&self) -> Style {   s!(C::Green, M::BOLD) }
    fn group_read(&self) -> Style { s!(C::Yellow) }
    fn group_write(&self) -> Style { s!(C::Red) }
    fn group_execute(&self) -> Style { s!(C::Green) }
    fn other_read(&self) -> Style { s!(C::Yellow) }
    fn other_write(&self) -> Style { s!(C::Red) }
    fn other_execute(&self) -> Style { s!(C::Green) }
}
