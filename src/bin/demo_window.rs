use std::{io, thread, time::Duration};
use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Widget, Block, Borders, Paragraph, Wrap, Gauge, Sparkline, Chart, Axis, Dataset, GraphType, LineGauge, ListItem, List, Table, Row, Cell},
    symbols,
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    Terminal,
    buffer::Buffer,
    terminal::Frame,
};
use tui::text::{Text, Span, Spans};
use tui::style::{Style, Color, Modifier};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::borrow::Cow;
use filez::util::{wrap_text, RectExt, Split2D};
use filez::widgets::Slider;

pub struct LogList {
    items: Vec<LogItem>,
    offset: usize,
}
pub struct StatusBar {
    message: Vec<(String, Style)>,
}

impl StatusBar {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        f.render_widget(Paragraph::new(Text::from(Spans(self.message.iter().map(|(m, s)| Span::styled(m, *s)).collect()))), rect);
    }
}

impl LogList {
    fn push(&mut self, log_item: LogItem) {
        if self.offset == self.items.len() {
            self.offset += 1;
        }
        self.items.push(log_item);
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let (rect, rect_slider) = rect.hsplit(rect.width.saturating_sub(1));

        let mut items: Vec<Vec<Spans>> = vec![];
        let mut height = 0;
        for LogItem { message , timestamp, level, target } in self.items[..self.offset].iter().rev() {
            if height > rect.height { break; }

            let level = Span::styled(level.to_string(), Style::default().fg(Color::Red));
            let target = Span::styled(target, Style::default().fg(Color::White));
            let timestamp = Span::styled(timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), Style::default().fg(Color::White));
            let mut content = vec![vec![
                "[".into(), timestamp, " ".into(), level, " ".into(), target, "] ".into(),
            ]];
            for (idx, message_line) in message.lines().enumerate() {
                let line = Span::styled(message_line, Style::default().fg(Color::Green));
                if idx == 0 {
                    content[0].push(line);
                } else {
                    content.push(vec![line]);
                }
            }

            let wrapped = wrap_text(Text {
                lines: content.into_iter().map(|line| Spans(line)).collect()
            }, rect.width as usize, false, 0);
            height += wrapped.height() as u16;

            items.push(wrapped.lines);
        }

        let (_, rect) = rect.vsplit(rect.height.saturating_sub(height));

        f.render_widget(Paragraph::new(Text { lines: items.into_iter().rev().flatten().collect() }), rect);
        f.render_widget(Slider::new(self.offset, self.items.len()), rect_slider);
    }
}

pub struct FileTable {
    offset_x: usize,
    offset_y: usize,
    select: usize,
}

// pub fn AutoTable {
//     rows: Row,
//     offset_x: usize,
// }

impl FileTable {
    fn new() -> Self {
        Self {
            offset_x: 4,
            offset_y: 2,
            select: 3,
        }
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let (r_select, r_body) = rect.hsplit(2);
        let Split2D {
            top_left: r_body,
            top_right: r_vslider,
            bottom_left: r_hslider,
            ..
        } = r_body.split_2d(r_body.width.saturating_sub(1), r_body.height.saturating_sub(1));
        let (_, r_select) = r_select.vsplit(1); let (r_select, _) = r_select.vsplit(r_select.height.saturating_sub(1));

        self.select = (r_select.height as usize).min(self.select);

        let contents: Vec<_> = (0..50).into_iter().map(|idx|
            // 2_i32.pow(idx)
            (format!("file_{}.txt", idx), "wtf", "sdfs", "sdfs")
        ).collect();

        let header = vec![
            "name", "size", "modified", "permissions",
        ];

        let mut rows = vec![];
        let mut height = 1;
        let column_spacing = 2;
        let mut widths: Vec<_> = header.iter().map(|&line| line.len()).collect();
        for (fname, size, modified, permissions) in contents.iter().skip(self.offset_y) {
            if height >= r_body.height {
                break;
            }

            let fname = Span::raw(fname);
            let size = Span::raw(*size);
            let modified = Span::raw(*modified);
            let permissions = Span::raw(*permissions);
            let item = vec![
                fname, size, modified, permissions
            ];

            for idx in 0..4 {
                widths[idx] = widths[idx].max(item[idx].width());
            }

            rows.push(Row::new(item).height(1));
            height += 1;
        }
        // log::info!("height: {}, {}", contents.len(), height);

        let widths = Vec::from_iter(widths.into_iter().map(|w| Constraint::Length(w as u16)));

        let table = Table::new(
            rows
        ).header(
            Row::new(header)
        ).widths(
            &widths
        ).column_spacing(
            column_spacing
        );

        f.render_widget(Paragraph::new(Text::styled(" ", Style::default().bg(Color::Red))), r_select.vsplit(self.select as u16).1);
        f.render_widget(Slider::new(10, 100).slider_color(Color::Red), r_vslider);
        f.render_widget(Slider::new(100, 100).direction(Direction::Horizontal).slider_color(Color::Red), r_hslider);
        f.render_widget(table, r_body);
    }
}

#[derive(Debug)]
pub enum Event {
    Log(LogItem)
}

pub struct App {
    log_list: LogList,
    file_table: FileTable,
    status: StatusBar
}

impl App {
    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::Log(log_item) => {
                self.log_list.push(log_item);
            },
        }
    }

    pub fn new() -> Self {
        Self {
            log_list: LogList { items: vec![], offset: 0 },
            status: StatusBar { message: vec![(
                "hello".into(), Style::default().fg(Color::Red)
            )] },
            file_table: FileTable::new(),
        }
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let (r_main, r_status) = rect.vsplit(rect.height.saturating_sub(1));
        let (r_logs_block, r_files_block) = r_main.hsplit(r_main.width / 2);
        let (_, r_logs_block) = r_logs_block.vsplit(r_logs_block.height / 2);
        let (r_files_block, _) = r_files_block.vsplit(r_files_block.height / 3);

        let b = Block::default().title("Logs").borders(Borders::ALL);
        let r_logs = b.inner(r_logs_block);
        f.render_widget(b, r_logs_block);

        let b = Block::default().title("Files").borders(Borders::ALL);
        let r_files = b.inner(r_files_block);
        f.render_widget(b, r_files_block);

        self.file_table.draw(f, r_files);
        self.log_list.draw(f, r_logs);
        self.status.draw(f, r_status);
    }
}

#[derive(Debug, Clone)]
pub struct LogItem {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: log::Level,
    pub target: String,
    pub message: String,
}

pub struct TuiLog {
    tx: std::sync::mpsc::SyncSender<LogItem>,
}

impl log::Log for TuiLog {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Err(_) = self.tx.send(LogItem {
                timestamp: chrono::Utc::now(),
                level: record.metadata().level(),
                target: record.metadata().target().to_string(),
                message: record.args().to_string()
            }) {
                eprintln!("The channel if full, drop message: {:?}", record);
            }
        }
    }

    fn flush(&self) {}
}

static mut LOGGER: Option<TuiLog> = None;

impl TuiLog {
    pub fn init(level: log::Level) -> Result<std::sync::mpsc::Receiver<LogItem>, log::SetLoggerError> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1024);
        log::set_max_level(level.to_level_filter());
        unsafe {
            LOGGER.replace(TuiLog { tx });
            log::set_logger(LOGGER.as_ref().unwrap())?;
        }
        Ok(rx)
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

    let mut is_exit = false;

    let mut app = App::new();
    loop {
        if is_exit { break; }

        while let Ok(log_item) = rx_logs.try_recv() {
            app.on_event(Event::Log(log_item));
        }

        // poll term events
        while let Ok(true) = event::poll(std::time::Duration::from_secs(0)) {
            let eterm = event::read().unwrap();
            if !matches!(eterm, event::Event::Mouse(_) | event::Event::Resize(..)) {
                log::debug!("got event: {:?}", eterm);
            }

            match eterm {
                event::Event::Key(event::KeyEvent { code: event::KeyCode::Char('q'), modifiers: event::KeyModifiers::NONE }) => {
                    is_exit = true;
                },
                _ => {}
            }
        }

        let elapsed = now.elapsed();
        if TO_WAIT > elapsed {
            std::thread::sleep(TO_WAIT-elapsed);
            now = now + TO_WAIT-elapsed;
        }

        terminal.draw(|f| app.draw(f, f.size()) )?;
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> io::Result<()> {
    let rx_logs = TuiLog::init(log::Level::Debug).expect("Fail to init logger");
    log::info!("hello");
    log::info!("hello again");
    log::info!("hello again\nbut with newline");
    run(rx_logs)?;

    Ok(())
}
