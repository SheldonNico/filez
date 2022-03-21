pub(crate) mod explorer;
pub(crate) mod info;
pub(crate) mod status;
pub(crate) mod popup;
pub(crate) mod app;

pub use info::LogRecord;
pub use app::run;
pub use panel::Styles as PanelStyles;
pub use file_explorer::Styles as FeStyles;

use crate::theme::Theme;

use std::io;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use tui::{
    Terminal, backend::CrosstermBackend,
    backend::Backend,
    Frame
};
use crossterm::{
    ExecutableCommand,
    terminal::{enable_raw_mode, disable_raw_mode},
    event::{self, EnableMouseCapture, DisableMouseCapture, KeyEvent, KeyCode, MouseEvent, MouseEventKind, MouseButton, KeyModifiers},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::widgets::{Block, Borders, Paragraph, Wrap, List, ListItem, Tabs, Table, Row, Clear};
use tui::text::{Span, Spans, Text};
use tui::style::{Style, Color, Modifier};
use tui::layout::{Alignment, Layout, Constraint, Direction, Rect, Margin};

/// NOTE: no copy here, we will send heay structs in events,
/// and keep our status Copyable
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Event {
    // Input behavior
    Key(KeyEvent),
    Mouse(MouseEvent),

    // Activity
    Log(LogRecord),
    Tick,

    // Other common commands
    Click(u16, u16),
    ScrollUp,
    ScrollDown,
    ExitConfirm,
    Keys_G,
    Keys_gg,
    Search(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Wait {
    WaitLeader([char; 5], usize), // Leader key for more keys
    WaitSearch([char; 20], usize),

    WaitClick(u16, u16),      // change focus
    WaitClickEdge(u16, u16),  // change size start

    WaitG,
}

pub trait Ui {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme);
    fn on_event(&mut self, _event: Event) {  }
}

pub trait Contain {
    fn contain(&self, column: u16, row: u16) -> bool;
}

impl Contain for Rect {
    fn contain(&self, column: u16, row: u16) -> bool {
        self.area() > 0 && column >= self.left() && column <= self.right() && row >= self.top() && row <= self.bottom()
    }
}

pub mod panel {
    use tui::style::Style;

    pub trait Styles {
        fn border_active(&self) -> Style;
        fn border_inactive(&self) -> Style;
        fn btn(&self) -> Style;
    }
}

pub mod file_explorer {
    use tui::style::Style;

    pub trait Styles {
        fn mark(&self) -> Style;
        fn cursor_select(&self) -> Style;

        fn highlight(&self) -> Style;

        // size
        fn size(&self) -> Style;

        // datetime
        fn datetime(&self) -> Style;

        // file_type
        fn directory(&self) -> Style;
        fn dot_dot(&self) -> Style;
        fn file(&self) -> Style;
        fn sym_link(&self) -> Style;
        fn other(&self) -> Style;

        // Permissions
        fn dashed(&self) -> Style;
        fn user_read(&self) -> Style;
        fn user_write(&self) -> Style;
        fn user_execute(&self) -> Style;
        fn group_read(&self) -> Style;
        fn group_write(&self) -> Style;
        fn group_execute(&self) -> Style;
        fn other_read(&self) -> Style;
        fn other_write(&self) -> Style;
        fn other_execute(&self) -> Style;
    }
}
