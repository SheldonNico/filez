use super::*;

const TITLE_LOG: &'static str   = "LOG";
const TITLE_QUEUE: &'static str = "Queue";
const TITLE_OK: &'static str    = "Ok";
const TITLE_ERR: &'static str   = "Err";
const TITLES: [&'static str; 4] = [ TITLE_LOG, TITLE_QUEUE, TITLE_OK, TITLE_ERR ];

#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: log::Level,
    pub target: String,
    pub timestamp: String,
    pub msg: String,
}

pub struct LogPanel {
    logs: Vec<LogRecord>,
    offset: usize,
    rect: Rect,
}

pub struct InfoPanel {
    p_logs: LogPanel,

    tab: usize,
    rect: Rect,
}

impl InfoPanel {
    pub fn new() -> Self {
        Self {
            p_logs: LogPanel {
                logs: vec![],
                offset: 0,
                rect: Rect::default(),
            },

            tab: 0,
            rect: Rect::default(),
        }
    }
}

impl Ui for LogPanel {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, _theme: &Theme) {
        let mut items = vec![];

        // ref: https://docs.rs/env_logger/latest/src/env_logger/fmt/writer/termcolor/extern_impl.rs.html
        let s_level = |l| match l {
            log::Level::Trace => Style::default().fg(Color::Cyan),
            log::Level::Debug => Style::default().fg(Color::Blue),
            log::Level::Info  => Style::default().fg(Color::Green),
            log::Level::Warn  => Style::default().fg(Color::Yellow),
            log::Level::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        };

        self.offset = self.offset.min(self.logs.len().saturating_sub(1));
        let mut height = 0;
        for record in self.logs.iter().skip(self.offset) {
            let li = ListItem::new(Spans::from(vec![
                Span::styled("[", Style::default().add_modifier(Modifier::DIM)),
                Span::from(&*record.timestamp),
                Span::from(" "),
                Span::styled(format!("{: <5}", record.level), s_level(record.level)),
                Span::from(" "),
                Span::from(&*record.target),
                Span::styled("]", Style::default().add_modifier(Modifier::DIM)),
                Span::from(" "),
                Span::styled(&record.msg, Style::default().fg(Color::White)),
            ]));
            height += li.height();
            items.push(li);
            if height > rect.height as usize { break; }
        }

        f.render_widget(List::new(items), rect);

        self.rect = rect;
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Log(record) => {
                // if it meets the last-1 column, we auto scroll
                if self.rect.height > 3 && self.offset + self.rect.height as usize - 1 == self.logs.len() {
                    self.offset += 1;
                }
                self.logs.push(record);
            },
            Event::ScrollDown => {
                self.offset = (self.offset + 1).min(self.logs.len().saturating_sub(1));
            },
            Event::ScrollUp => {
                self.offset = self.offset.saturating_sub(1);
            },
            Event::Key(KeyEvent { code: KeyCode::Up | KeyCode::Char('k'), modifiers: KeyModifiers::NONE }) => {
                self.on_event(Event::ScrollUp);
            },
            Event::Key(KeyEvent { code: KeyCode::Down | KeyCode::Char('j'), modifiers: KeyModifiers::NONE }) => {
                self.on_event(Event::ScrollDown);
            },
            Event::Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL }) => {
                self.offset = self.logs.len().saturating_sub(1);
            },
            _ => { }
        }
    }
}

impl Ui for InfoPanel {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) {
        if rect.height < 1 { return; }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(rect.height - 1),
            ])
            .split(rect);

        let titles: Vec<_> = TITLES.into_iter().map(Spans::from).collect();

        f.render_widget(
            Tabs::new(titles)
                .style(Style::default().fg(Color::White))
                .select(self.tab)
                .highlight_style(Style::default().fg(Color::Yellow)),
            chunks[0]
        );

        if self.tab >= TITLES.len() || TITLES[self.tab] == TITLE_LOG {
            self.p_logs.draw(f, chunks[1], theme);
        } else if TITLES[self.tab] == TITLE_QUEUE {
            f.render_widget(Block::default().borders(Borders::ALL).title("Q"), chunks[1]);
        } else if TITLES[self.tab] == TITLE_OK {
            f.render_widget(Block::default().borders(Borders::ALL).title("O"), chunks[1]);
        } else {
            f.render_widget(Block::default().borders(Borders::ALL).title("E"), chunks[1]);
        }

        self.rect = rect;
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Log(_)     => { self.p_logs.on_event(event); },
            Event::Click(column, row)  => {
                if row == self.rect.y {
                    let mut left = self.rect.x;
                    for (idx, title) in TITLES.into_iter().enumerate() {
                        let width = Span::from(title).width() as u16 + 2;
                        let right = left + width;
                        if left <= column && column <= right {
                            self.tab = idx;
                            break;
                        }
                        left = left + width + 1;
                    }
                }
            },
            Event::Key(KeyEvent { code: KeyCode::Right | KeyCode::Char('l'), modifiers: KeyModifiers::NONE }) => {
                self.tab = self.tab.saturating_add(1).min(TITLES.len().saturating_sub(1));
            },
            Event::Key(KeyEvent { code: KeyCode::Left | KeyCode::Char('h'), modifiers: KeyModifiers::NONE }) => {
                self.tab = self.tab.saturating_sub(1).min(TITLES.len().saturating_sub(1));
            },
            _                 => {
                if self.tab >= TITLES.len() || TITLES[self.tab] == TITLE_LOG {
                    self.p_logs.on_event(event);
                }
            },
        }
    }
}
