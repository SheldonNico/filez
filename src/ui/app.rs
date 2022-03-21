use super::*;

use crate::theme::Theme;
use super::info::InfoPanel;
use super::explorer::ExplorerPanel;
use super::status::StatusPanel;
use super::popup::{PopupPanel, Popup};
use crate::fs::LocalHost;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Uid {
    Local,
    Remote,
    Info,
    Status,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Status {
    Exit,

    Popup,                        // work like another screen
    Normal,                       // interact globally

    // Focus(Uid),
}

pub struct App {
    p_local: ExplorerPanel<LocalHost>,
    p_remote: ExplorerPanel<LocalHost>,
    p_info: InfoPanel,
    p_status: StatusPanel,
    p_popup: Option<PopupPanel>,

    status: Status,

    wait: Option<Wait>,
    wait_key: Instant,

    // layout
    layout: [u16; 5],
    focus: Option<Uid>,
}

impl Uid {
    fn title(&self) -> &'static str {
        match self {
            Self::Local  => "Local",
            Self::Remote => "Remote",
            Self::Info   => "Info",
            Self::Status => "Status",
        }
    }
}

pub fn run(rx_logs: Receiver<LogRecord>, left: &str, right: &str) -> io::Result<()> {
    let mut app = App::new(left, right)?;

    // setup
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.backend_mut().execute(EnterAlternateScreen)?;
    terminal.backend_mut().execute(EnableMouseCapture)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    log::info!("hello");
    let mut now = std::time::Instant::now();
    const TO_WAIT: std::time::Duration = std::time::Duration::from_millis(15);

    let theme = Theme::default();

    loop {
        if app.status == Status::Exit { break; }

        // timer
        app.on_event(Event::Tick);

        // poll term events
        while let Ok(true) = event::poll(std::time::Duration::from_secs(0)) {
            let eterm = event::read().unwrap();
            match eterm {
                event::Event::Key(key) => app.on_event(Event::Key(key)),
                event::Event::Mouse(mouse) => app.on_event(Event::Mouse(mouse)),
                _ => {}
            }
        }

        // poll log events
        while let Ok(log_record) = rx_logs.try_recv() {
            app.on_event(Event::Log(log_record));
        }

        // check fps: 60 fps
        let elapsed = now.elapsed();
        if TO_WAIT > elapsed {
            std::thread::sleep(TO_WAIT-elapsed);
            now = now + TO_WAIT-elapsed;
        }
        terminal.draw(|f| app.draw(f, f.size(), &theme))?;
    }

    // restore
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.backend_mut().execute(DisableMouseCapture)?;
    terminal.show_cursor()?;
    disable_raw_mode()?;

    Ok(())
}

impl Ui for App {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) {
        if rect.area() == 0 { return }
        if rect.width < 2 || rect.height < 2*3 { return }
        let mut height = rect.height;
        let h_status = if height > 1 { 1 } else { 0 }; height -= h_status;
        let h_info = if height > 20 { 20 } else { 0 }; height -= h_info;
        let h_transfer = if height > 0 { height } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(h_transfer),
                Constraint::Length(h_info),
                Constraint::Length(h_status),
            ])
            .split(rect);

        let y0 = chunks[0].bottom(); let y1 = chunks[1].bottom(); let y2 = chunks[2].bottom();

        let r_info = self.draw_block(f, theme, Uid::Info, chunks[1]);
        let r_status = chunks[2];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(chunks[0].width / 2),
                Constraint::Length(chunks[0].width - chunks[0].width / 2),
            ])
            .split(chunks[0]);
        let x0 = chunks[0].right(); let x1 = chunks[1].right();
        let r_local = self.draw_block(f, theme, Uid::Local, chunks[0]);
        let r_remote = self.draw_block(f, theme, Uid::Remote, chunks[1]);

        self.layout = [y0, y1, y2, x0, x1];

        self.p_local.draw(f, r_local, theme);
        self.p_remote.draw(f, r_remote, theme);
        self.p_info.draw(f, r_info, theme);
        self.p_status.draw(f, r_status, theme);

        if self.status == Status::Popup {
            if let Some(popup) = &mut self.p_popup {
                let r_pop = popup.suit_in(rect).intersection(rect); // bound check, hahaha
                f.render_widget(Clear, r_pop);
                popup.draw(f, r_pop, theme);
            } else {
                log::error!("Got pop up status with None p_popup");
                self.status = Status::Normal;
            }
        }
    }

    fn on_event(&mut self, event: Event) {
        match &event {
            Event::Tick => { },
            Event::Log(_) => { },
            Event::Mouse(MouseEvent { kind: MouseEventKind::Moved , .. }) => { }, // too annoying.
            Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown , .. }) => { }, // too annoying.
            Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp , .. }) => { }, // too annoying.
            _ => { log::debug!("Got event in info panel ({:?}): {:?}", self.status, event); }
        }

        if self.wait.is_some() && matches!(event, Event::Key(..) | Event::Mouse(..)) {
            let wait = self.wait.take().unwrap();
            self.process_wait(wait, event);
        } else {
            self.process_status(self.status, event);
        }
    }
}

impl App {
    fn new(left: &str, right: &str) -> io::Result<Self> {
        Ok(Self {
            p_local: ExplorerPanel::new(LocalHost, left)?,
            p_remote: ExplorerPanel::new(LocalHost, right)?,
            p_info: InfoPanel::new(),
            p_status: StatusPanel::new(),
            p_popup: None,

            status: Status::Normal,
            wait: None,
            wait_key: Instant::now(),

            layout: Default::default(),
            focus: Some(Uid::Local),
        })
    }

    fn window_left(&mut self) {
        match self.focus {
            Some(Uid::Remote) => { self.focus = Some(Uid::Local); }
            Some(Uid::Info)   => {  }
            Some(Uid::Local)  => {  }
            _                 => { self.focus = Some(Uid::Local); }
        }
    }

    fn window_right(&mut self) {
        match self.focus {
            Some(Uid::Local)  => { self.focus = Some(Uid::Remote); },
            Some(Uid::Info)   => {  },
            Some(Uid::Remote) => {  },
            _                 => { self.focus = Some(Uid::Remote); }
        }
    }

    fn window_down(&mut self) {
        match self.focus {
            Some(Uid::Remote) => { self.focus = Some(Uid::Info); }
            Some(Uid::Local)  => {  self.focus = Some(Uid::Info);  }
            Some(Uid::Info)   => {  }
            _                 => { self.focus = Some(Uid::Info); }
        }
    }

    fn window_up(&mut self) {
        match self.focus {
            Some(Uid::Remote) => {  }
            Some(Uid::Local)  => {  }
            Some(Uid::Info)   => { self.focus = Some(Uid::Local); }
            _                 => { self.focus = Some(Uid::Local); }
        }
    }

    fn draw_block<B: Backend>(&self, f: &mut Frame<B>, theme: &Theme, uid: Uid, rect: Rect) -> Rect {
        let bs = if self.focus == Some(uid) { theme.border_active() } else { theme.border_inactive() };
        let block = Block::default().title(uid.title()).borders(Borders::ALL).border_style(bs);
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        return inner;
    }

    fn interact(&mut self, uid: Uid, event: Event) {
        match uid {
            Uid::Local  => { self.p_local.on_event(event); }
            Uid::Remote => { self.p_remote.on_event(event); }
            Uid::Info   => { self.p_info.on_event(event); }
            Uid::Status => { self.p_status.on_event(event); }
        }
    }

    fn interact_focus(&mut self, event: Event) {
        if let Some(uid) = self.focus {
            self.interact(uid, event);
        }
    }

    fn interact_popup(&mut self, event: Event) {
        if self.status == Status::Popup {
            if let Some(mut popup) = self.p_popup.take() {
                popup.on_event(event);
                if popup.exit() {
                    self.status = Status::Normal; // restore back to normal
                    if let Some(res) = popup.emit() {
                        self.on_event(res);
                    }
                } else {
                    self.p_popup = Some(popup);
                }
            } else {
                log::error!("event pass to Popup, but p_popup is None");
                self.status = Status::Normal;
            }
        }
    }

    fn interact_layout(&mut self, column: u16, row: u16) -> Option<Uid> {
        let [y0, y1, y2, x0, x1] = self.layout;

        if row <= y0 {
            if column <= x0 {
                return Some(Uid::Local);
            } else if column <= x1 {
                return Some(Uid::Remote);
            }
        } else if row <= y1 {
            return Some(Uid::Info);
        } else if row <= y2 {
            return Some(Uid::Status);
        }

        None
    }

    fn start_wait(&mut self, wait: Wait) {
        self.wait = Some(wait);
        self.wait_key = Instant::now();
    }

    fn process_wait(&mut self, wait: Wait, event: Event) {
        match (wait, &event) {
            (Wait::WaitClick(column_old, row_old), &Event::Mouse(MouseEvent { kind: MouseEventKind::Drag(MouseButton::Left), column, row, .. })) => {
                if column_old == column && row_old == row {
                    // the mouse is too sensitive
                    self.start_wait(Wait::WaitClick(column, row));
                }
            },
            (Wait::WaitClick(..), &Event::Mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, .. })) => {
                self.on_event(Event::Click(column, row));
            },
            (Wait::WaitLeader(mut keys, mut idx), &Event::Key(KeyEvent { code: KeyCode::Char(code), modifiers: KeyModifiers::NONE })) => {
                self.wait_key = Instant::now();
                keys[idx] = code;
                match keys {
                    ['w', 'h', ..] => { self.window_left(); },
                    ['w', 'l', ..] => { self.window_right(); },
                    ['w', 'j', ..] => { self.window_down(); },
                    ['w', 'k', ..] => { self.window_up(); },
                    _ => {
                        if keys.len() > idx + 1 {
                            idx += 1;
                            self.start_wait(Wait::WaitLeader(keys, idx));
                        } else {
                            log::info!("leader keys more than 5, return to normal status: {:?}", keys);
                        }
                    }
                }
            },
            (Wait::WaitG, &Event::Key(KeyEvent { code: KeyCode::Char('g'), modifiers: KeyModifiers::NONE })) => {
                self.on_event(Event::Keys_gg);
            },
            (Wait::WaitSearch(keys, idx), &Event::Key(KeyEvent { code: KeyCode::Backspace, modifiers: KeyModifiers::NONE })) => {
                self.start_wait(Wait::WaitSearch(keys, idx.saturating_sub(1)));
            },
            (Wait::WaitSearch(mut keys, mut idx), &Event::Key(KeyEvent { code: KeyCode::Char(code), modifiers: KeyModifiers::NONE })) => {
                keys[idx] = code;

                if keys.len() > idx + 1 {
                    idx += 1;
                    self.start_wait(Wait::WaitSearch(keys, idx));
                } else {
                    log::info!("search keys more than {}, return to normal status: {:?}", keys.len(), keys);
                }
            },
            (Wait::WaitSearch(keys, len), &Event::Key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE })) => {
                let search: String = keys.into_iter().take(len).collect();
                self.on_event(Event::Search(search));
            },

            (_, Event::Key(..) | Event::Mouse(..)) => { },
            _ => unreachable!()
        }
    }

    fn process_status(&mut self, status: Status, event: Event) {
        // ref event here, since event doesn't have Copy
        match (status, &event) {
            (_, Event::Tick) => {
                if self.wait.is_some() && self.wait_key.elapsed() > std::time::Duration::from_secs(5) {
                    self.wait = None;
                    log::info!("leader keys wait too long, return to normal status");
                }
            },

            (_, Event::Log(_)) => {
                self.p_info.on_event(event);
            },

            // processing mouse events
            (_, &Event::Mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, .. })) => {
                self.start_wait(Wait::WaitClick(column, row));
            },
            (_, &Event::Mouse(MouseEvent { column, row, kind: MouseEventKind::ScrollUp, .. })) => {
                if let Some(uid) = self.interact_layout(column, row) { self.interact(uid, Event::ScrollUp); }
            },
            (_, &Event::Mouse(MouseEvent { column, row, kind: MouseEventKind::ScrollDown, .. })) => {
                if let Some(uid) = self.interact_layout(column, row) { self.interact(uid, Event::ScrollDown); }
            },

            (_, Event::ExitConfirm) => {
                self.status = Status::Exit;
            },

            // escap in most case
            (Status::Normal, Event::Key(KeyEvent { code: KeyCode::Esc, .. })) => {
                self.status = Status::Normal;
                self.focus = None;
            },
            (Status::Normal, Event::Key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE })) => {
                if self.focus != Some(Uid::Local) {
                    self.focus = Some(Uid::Local);
                } else {
                    self.interact_focus(event);
                }
            },
            (Status::Normal, Event::Key(KeyEvent { code: KeyCode::Char('q'), .. })) => {
                self.p_popup = Some(PopupPanel::new_exit());
                self.status = Status::Popup;
            },
            (Status::Normal, Event::Key(KeyEvent { code: KeyCode::Char('?'), .. })) => {
                self.p_popup = Some(PopupPanel::new_help());
                self.status = Status::Popup;
            },

            // leader keys
            (Status::Normal, &Event::Key(KeyEvent { code: KeyCode::Char(' '), modifiers: KeyModifiers::NONE })) => {
                self.start_wait(Wait::WaitLeader([0 as char; 5], 0));
            },
            // search keys
            (Status::Normal, &Event::Key(KeyEvent { code: KeyCode::Char('/'), modifiers: KeyModifiers::NONE })) => {
                self.start_wait(Wait::WaitSearch(Default::default(), 0));
            },

            (Status::Normal, &Event::Key(KeyEvent { code: KeyCode::Char('G'), modifiers: KeyModifiers::SHIFT })) => {
                self.on_event(Event::Keys_G);
            },
            (Status::Normal, &Event::Key(KeyEvent { code: KeyCode::Char('g'), modifiers: KeyModifiers::NONE })) => {
                self.start_wait(Wait::WaitG);
            },

            ////////////////////////////////////////////////////////////////////////////////////////////////////////////
            (Status::Popup, Event::Key(..)) => { self.interact_popup(event); }
            (Status::Popup, Event::Mouse(..)) => { self.interact_popup(event); }
            (Status::Popup, Event::Click(..)) => { self.interact_popup(event); }
            (Status::Popup, Event::ScrollDown) => { self.interact_popup(event); }
            (Status::Popup, Event::ScrollUp) => { self.interact_popup(event); }

            (Status::Normal, &Event::Click(column, row)) => {
                if let Some(uid) = self.interact_layout(column, row) {
                    self.focus = Some(uid);
                    self.interact_focus(event);
                }
            },
            _ => { self.interact_focus(event); },
        }
    }
}

