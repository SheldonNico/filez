use std::io;
use tui::{
    backend::{CrosstermBackend, Backend},
    layout::{Direction, Alignment, Rect, Corner},
    Terminal,
    buffer::Buffer,
    terminal::Frame,
    widgets::{Widget, Block, Borders, Paragraph, Wrap, Gauge, Sparkline, Chart, Axis, Dataset, GraphType, LineGauge, ListItem, List, Table, Row, Cell, BorderType},
};
use tui::text::{Text, Span, Spans};
use tui::style::{Style, Color, Modifier};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{EnableMouseCapture, DisableMouseCapture, KeyEvent, KeyCode, MouseEvent, MouseEventKind, MouseButton, KeyModifiers},
};
use filez::util::{RectExt, Split2D, TuiLog, LogItem};
use filez::widgets::{Slider, ELLIPSES, Table2, List2, get_unicode_bar, get_unicode_block};
use std::time::{Instant, Duration};

pub trait Layout<L> {
    fn locate(&self, x: u16, y: u16) -> Option<L>;
}

impl<L: Copy> Layout<L> for Vec<(Rect, L)> {
    fn locate(&self, x: u16, y: u16) -> Option<L> {
        for &(r, label) in self.iter() {
            if r.contain(x, y) {
                return Some(label)
            }
        }
        None
    }
}

pub trait Ui {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, _state: &State) {
        let block = Block::default().title("PlaceHolder").borders(Borders::ALL);
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let (_, inner) = inner.vsplit(inner.height / 2);
        f.render_widget(Paragraph::new(Text::styled(format!("{:?}", rect), Style::default().fg(Color::Red)))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            inner
        );
    }

    fn on_event(&mut self, _event: Event, _state: &mut State) { }
}

pub struct PlaceHolder {
    last_event: Option<Event>
}

impl PlaceHolder {
    fn new() -> Self {
        Self {
            last_event: None
        }
    }
}

impl Ui for PlaceHolder {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, _state: &State) {
        let block = Block::default().title("PlaceHolder").borders(Borders::ALL);
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let (_, inner) = inner.vsplit(inner.height / 2);
        f.render_widget(Paragraph::new(Text::styled(
                    format!("rect: {:?}\nevent: {:?}", rect, self.last_event),
                    Style::default().fg(Color::Red))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            inner
        );
    }

    fn on_event(&mut self, event: Event, _state: &mut State) {
        self.last_event = Some(event);
    }
}

pub struct Theme {
    pub border_color_focused : Color,
    pub log_level_trace      : Color,
    pub log_level_debug      : Color,
    pub log_level_info       : Color,
    pub log_level_warn       : Color,
    pub log_level_error      : Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border_color_focused : Color::Green,

            log_level_trace      : Color::Cyan,
            log_level_debug      : Color::Blue,
            log_level_info       : Color::Green,
            log_level_warn       : Color::Yellow,
            log_level_error      : Color::Red,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Wait {
    WaitClick(ClickEvent),      // change focus
    WaitG,
}

#[derive(Debug, Clone)]
pub struct TimeOut<T> {
    pub inner: T,
    pub ins: Instant,
    pub dur: Duration,
}

impl<T> TimeOut<T> {
    pub fn timeout(&self) -> bool {
        return self.ins.elapsed() > self.dur
    }
}

pub struct State {
    pub theme: Theme,
    pub status: Status,
}

impl State {
    pub fn new() -> Self {
        Self { theme: Theme::default(), status: Status::Normal, }
    }
}

pub struct LogList {
    items: Vec<LogItem>,
    offset: usize,
}

impl Ui for LogList {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, state: &State) {
        let theme = &state.theme;
        let (rect, rect_slider) = rect.hsplit(rect.width.saturating_sub(1));

        // ref: https://docs.rs/env_logger/latest/src/env_logger/fmt/writer/termcolor/extern_impl.rs.html
        let s_level = |l| match l {
            log::Level::Trace => Style::default().fg(theme.log_level_trace).add_modifier(Modifier::DIM),
            log::Level::Debug => Style::default().fg(theme.log_level_debug),
            log::Level::Info  => Style::default().fg(theme.log_level_info),
            log::Level::Warn  => Style::default().fg(theme.log_level_warn),
            log::Level::Error => Style::default().fg(theme.log_level_error).add_modifier(Modifier::BOLD),
        };

        let items = self.items.iter().map(|LogItem { message, timestamp, level, target }| {
            let mut content = vec![Spans(vec![
                Span::styled("[", Style::default().add_modifier(Modifier::DIM)),
                Span::raw(timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
                Span::raw(" "),
                Span::styled(format!("{: <5}", level), s_level(*level)),
                Span::raw(" "),
                Span::raw(target),
                Span::styled("]", Style::default().add_modifier(Modifier::DIM)),
                Span::from(" "),
            ])];
            for (idx, message_line) in message.lines().enumerate() {
                let line = Span::raw(message_line);
                if idx == 0 {
                    content[0].0.push(line);
                } else {
                    content.push(Spans(vec![line]));
                }
            }

            Text { lines: content }
        });

        f.render_widget(
            List2::new(
                items.take(self.offset).rev(),
                0,
                rect.height as usize,
            ).truncator(ELLIPSES).offset_x(0).wrap(rect.width as usize).start_corner(Corner::BottomLeft),
            rect
        );

        f.render_widget(
            Slider::new(self.items.len().saturating_sub(self.offset), self.items.len()).slider_color(Color::Red).rev()
            // SliderBuilder::new(self.offset, self.items.len()).make(rect_slider, Direction::Vertical)
            // .slider_color(Color::Red),
            ,
            rect_slider
        );
    }

    fn on_event(&mut self, event: Event, _state: &mut State) {
        match event {
            Event::Key(KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE }) => {
                self.offset = self.offset.saturating_add(1);
            },
            Event::Key(KeyEvent { code: KeyCode::Char('k'), modifiers: KeyModifiers::NONE }) => {
                self.offset = self.offset.saturating_sub(1);
            },
            Event::Keys_gg => {
                self.offset = 1;
            },
            Event::Key(KeyEvent { code: KeyCode::Char('G'), modifiers: KeyModifiers::SHIFT }) => {
                self.offset = self.items.len();
            },
            _ => {  }
        }
        self.offset = self.offset.min(self.items.len());
    }
}

pub struct StatusBar {
    message: Vec<(String, Style)>,
}

impl StatusBar {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, state: &State) {
        f.render_widget(Paragraph::new(Text::from(Spans(self.message.iter().map(|(m, s)| Span::styled(m, *s)).collect()))), rect);
    }
}

impl LogList {
    pub fn new() -> Self {
        Self { items: vec![], offset: 0 }
    }

    pub fn push(&mut self, log_item: LogItem) {
        if self.offset == self.items.len() {
            self.offset += 1;
        }
        self.items.push(log_item);
    }
}

pub struct FileTable {
    offset_x: usize,
    offset_y: usize,
    select: usize,
}

impl Ui for FileTable {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, state: &State) {
        let (r_select, r_body) = rect.hsplit(2);
        let Split2D {
            top_left: r_body,
            top_right: r_vslider,
            bottom_left: r_hslider,
            ..
        } = r_body.split_2d(r_body.width.saturating_sub(1), r_body.height.saturating_sub(1));
        let (_, r_select) = r_select.vsplit(1); let (r_select, _) = r_select.vsplit(r_select.height.saturating_sub(1));

        self.select = (r_select.height as usize).min(self.select);

        let contents: Vec<_> = (0..100).into_iter().map(|idx|
            // 2_i32.pow(idx)
            (format!("file_{}.txt", idx), "🧑", "sdfs\nnewline", "sdfs")
        ).collect();

        let header = vec![
            "name", "size", "modified", "permissions",
        ];

        f.render_widget(Paragraph::new(Text::styled(" ", Style::default().bg(Color::Red))), r_select.vsplit(self.select as u16).1);
        f.render_widget(Slider::new(0, 100).direction(Direction::Vertical).slider_color(Color::Red), r_vslider);
        f.render_widget(Slider::new(0, 500).direction(Direction::Horizontal).slider_color(Color::Red), r_hslider);
        f.render_widget(Table2::new(
                contents.iter().map(|(fname, size, modified, permissions)| vec![
                    Text::raw(fname),
                    Text::raw(*size),
                    Text::raw(*modified),
                    Text::raw(*permissions),
                ]),
                header.iter().map(|col| Text::raw(*col)),
                0,
                r_body.height as usize
        ).offset_x(0), r_body);
    }
}

impl FileTable {
    fn new() -> Self {
        Self {
            offset_x: 4,
            offset_y: 2,
            select: 3,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Log(LogItem),

    // Input behavior
    Key(KeyEvent),
    Mouse(MouseEvent),

    Keys_gg,

    // Timer
    Tick,

    // continusous event
    Click(ClickEvent)
}

#[derive(Debug, Clone, Copy)]
pub struct ClickEvent {
    pub button: MouseButton,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Status {
    Exit,
    Normal,
}

impl Event {
    pub fn try_from_crossterm(event: crossterm::event::Event)-> Option<Self> {
        match event {
            crossterm::event::Event::Key(inner) => Some(Self::Key(inner)),
            crossterm::event::Event::Mouse(inner) => Some(Self::Mouse(inner)),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Uid {
    LogList,
    FileTable,
}

pub struct App {
    log_list: LogList,
    file_table: FileTable,
    status_bar: StatusBar,

    // internal
    focus: Option<Uid>,

    waits: Vec<TimeOut<Wait>>,

    layout: Vec<(Rect, Uid)>,
}

impl Ui for App {
    fn on_event(&mut self, event: Event, state: &mut State) {
        match &event {
            Event::Tick => { },
            Event::Log(_) => { },
            // Event::Mouse(MouseEvent { kind: MouseEventKind::Moved , .. }) => { }, // too annoying.
            // Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown , .. }) => { }, // too annoying.
            // Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp , .. }) => { }, // too annoying.
            Event::Mouse(MouseEvent { .. }) => { }, // too annoying.
            _ => { log::debug!("Got event in info panel: {:?}", event); }
        }

        for TimeOut { ins, dur, inner: wait } in std::mem::replace(&mut self.waits, Vec::new()).into_iter() {
            match (wait, &event) {
                (
                    Wait::WaitClick(ClickEvent { button, column, row, modifiers }),
                    &Event::Mouse(MouseEvent {
                        kind: MouseEventKind::Up(new_button),
                        column: new_colume,
                        row: new_row,
                        modifiers: new_modifiers
                    })
                ) if button == new_button && row == new_row && column == new_colume && modifiers == new_modifiers => {
                    self.on_event(Event::Click(ClickEvent { button, column, row, modifiers }), state);
                },
                (Wait::WaitG, Event::Key(KeyEvent { code: KeyCode::Char('g'), modifiers: KeyModifiers::NONE })) => {
                    self.on_event(Event::Keys_gg, state); return;
                },
                _ => { self.waits.push(TimeOut { ins, dur, inner: wait }); }
            }
        }

        match event {
            Event::Log(log_item) => { self.log_list.push(log_item); return ;},
            Event::Tick => {
                self.waits = std::mem::replace(&mut self.waits, Default::default())
                    .into_iter().filter(|v| !TimeOut::timeout(v)).collect();
                return;
            },
            _ => {  },
        }

        match event {
            Event::Click(ClickEvent { button: MouseButton::Left, column, row, modifiers: KeyModifiers::NONE }) => {
                if let Some(uid) = self.layout.locate(column, row) {
                    self.focus = Some(uid);
                }
            },
            Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE }) => {
                state.status = Status::Exit;
            },
            Event::Key(KeyEvent { code: KeyCode::Esc, modifiers: KeyModifiers::NONE }) => {
                self.focus = None;
            },
            Event::Key(KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::CONTROL }) => {
                if self.focus == Some(Uid::FileTable) { self.focus = Some(Uid::LogList); }
            },
            Event::Key(KeyEvent { code: KeyCode::Char('k'), modifiers: KeyModifiers::CONTROL }) => {
                if self.focus == Some(Uid::LogList) { self.focus = Some(Uid::FileTable); }
            },
            Event::Mouse(MouseEvent { kind: MouseEventKind::Down(button), column, row, modifiers }) => {
                self.start_wait(Wait::WaitClick(ClickEvent { button, column, row, modifiers }), Duration::from_secs(5));
            },
            Event::Key(KeyEvent { code: KeyCode::Char('g'), modifiers: KeyModifiers::NONE }) => {
                self.start_wait(Wait::WaitG, Duration::from_secs(5));
            },
            _ => {  }
        }

        if let Some(uid) = self.focus {
            self.dispatch(uid, state, event);
        }
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, state: &State) {
        let theme = &state.theme;
        let (r_main, r_status) = rect.vsplit(rect.height.saturating_sub(1));
        let (r_files_block, r_logs_block) = r_main.vsplit(r_main.height.saturating_sub(10));
        let (r_files_block, wtf) = r_files_block.hsplit(r_files_block.width / 2);

        PlaceHolder::new().draw(f, wtf, state);

        let b = self.make_block(Uid::LogList, theme);
        let r_logs = b.inner(r_logs_block);
        f.render_widget(b, r_logs_block);

        let b = self.make_block(Uid::FileTable, theme);
        let r_files = b.inner(r_files_block);
        f.render_widget(b, r_files_block);

        self.file_table.draw(f, r_files, state);
        self.log_list.draw(f, r_logs, state);
        self.status_bar.draw(f, r_status, state);
        self.layout = vec![(r_files, Uid::FileTable), (r_logs, Uid::LogList)];
    }
}

impl App {
    fn dispatch(&mut self, uid: Uid, state: &mut State, event: Event) {
        match uid {
            Uid::FileTable => self.file_table.on_event(event, state),
            Uid::LogList => self.log_list.on_event(event, state),
            _ => {  },
        }
    }

    fn start_wait(&mut self, wait: Wait, dur: Duration) {
        self.waits.push(TimeOut { ins: Instant::now(), dur, inner: wait });
    }

    pub fn new() -> Self {
        Self {
            log_list: LogList::new(),
            status_bar: StatusBar { message: vec![
                ("Q: exit".into(), Style::default().fg(Color::Red)),
                (" ".into(), Default::default()),
                ("move mouse to view focus animations".into(), Style::default().fg(Color::Red)),
            ] },
            file_table: FileTable::new(),
            focus: None,
            waits: Vec::with_capacity(32),
            layout: Default::default(),
        }
    }

    fn make_block(&self, uid: Uid, theme: &Theme) -> Block {
        let (bs, bt) = if self.focus == Some(uid) {
            (Style::default().fg(theme.border_color_focused), BorderType::Thick)
        } else {
            (Style::default(), BorderType::Plain)
        };
        Block::default().title(format!("{:?}", uid)).borders(Borders::ALL).border_style(bs).border_type(bt)
    }
}

fn run(rx_logs: std::sync::mpsc::Receiver<LogItem>) -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut now = std::time::Instant::now();
    const TO_WAIT: std::time::Duration = std::time::Duration::from_millis(15);

    let mut app = App::new();
    let mut state = State::new();
    loop {
        if state.status == Status::Exit { break; }

        // timer
        app.on_event(Event::Tick, &mut state);

        while let Ok(log_item) = rx_logs.try_recv() {
            app.on_event(Event::Log(log_item), &mut state);
        }

        // poll term events
        while let Ok(true) = crossterm::event::poll(std::time::Duration::from_secs(0)) {
            let eterm = crossterm::event::read().unwrap();
            if let Some(event) = Event::try_from_crossterm(eterm) {
                app.on_event(event, &mut state);
            }
        }

        let elapsed = now.elapsed();
        if TO_WAIT > elapsed {
            std::thread::sleep(TO_WAIT-elapsed);
            now = now + TO_WAIT-elapsed;
        }

        terminal.draw(|f| app.draw(f, f.size(), &state) )?;
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> io::Result<()> {
    let rx_logs = TuiLog::init(log::Level::Debug).expect("Fail to init logger");
    log::error!("🧑 hello");
    log::info!("hello again");
    log::info!("hello again\nbut with newline");
    run(rx_logs)?;

    Ok(())
}
