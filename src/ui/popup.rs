use super::*;

pub enum PopupPanel {
    Exit(YesOrNo),
    Help(HelpPanel),
    Input(InputPanel),
}

// NOTE: Fix not mean accurate size, you still render in smaller size if terminal is too small
pub trait Popup: Ui {
    fn suit_in(&self, rect: Rect) -> Rect;
    fn emit(self) -> Option<Event>; // return signal to outside
    fn exit(&self) -> bool; // return signal to outside
}

impl PopupPanel {
    pub fn new_exit() -> Self {
        Self::Exit(YesOrNo::new(
            None,
            "exit filez?".to_owned(),
            [Some(Event::ExitConfirm), None, None],
        ))
    }

    pub fn new_help() -> Self {
        Self::Help(HelpPanel::new())
    }
}

pub struct YesOrNo {
    title: Option<String>,
    msg: String,

    exit: bool,
    state: usize,
    rtn: [Option<Event>; 3],
    r_ok: Rect,
    r_no: Rect,
    r_wd: Rect,
}

impl YesOrNo {
    fn new(
        title: Option<String>,
        msg: String,
        rtn: [Option<Event>; 3],
    ) -> Self {
        Self { title, msg, exit: false, state: 0, rtn, r_ok: Rect::default(), r_no: Rect::default(), r_wd: Default::default() }
    }
}

pub struct InputPanel {

}

pub struct HelpPanel {
    exit: bool,
    rect: Rect,
}

impl HelpPanel {
    fn new() -> Self {
        Self {
            exit: false,
            rect: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
macro_rules! auto_popup {
    ($slf:ident, $inner:ident, $code:stmt ) => {
        match $slf {
            PopupPanel::Exit($inner)  => { $code },
            PopupPanel::Help($inner)  => { $code },
            PopupPanel::Input($inner) => { $code },
        }
    };
}

impl Ui for PopupPanel {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) {
        auto_popup!(self, inner, inner.draw(f, rect, theme))
    }

    fn on_event(&mut self, event: Event) {
        auto_popup!(self, inner, inner.on_event(event))
    }

}

impl Popup for PopupPanel {
    fn suit_in(&self, rect: Rect) -> Rect {
        auto_popup!(self, inner, inner.suit_in(rect))
    }

    fn emit(self) -> Option<Event> {
        auto_popup!(self, inner, inner.emit())
    }

    fn exit(&self) -> bool {
        auto_popup!(self, inner, inner.exit())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Popup for YesOrNo {
    fn suit_in(&self, rect: Rect) -> Rect {
        rect.inner(&Margin {
            vertical: rect.height.saturating_sub(10) / 2,
            horizontal: rect.width.saturating_sub(60) / 2,
        })
    }

    fn emit(self) -> Option<Event> { self.rtn[self.state].clone() }
    fn exit(&self) -> bool { self.exit }
}

impl Ui for YesOrNo {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) {
        self.r_wd = rect;
        let mut block = Block::default().borders(Borders::ALL).border_style(theme.border_active());
        if let Some(title) = &self.title { block = block.title(&**title); }
        let r_inner = block.inner(rect);
        f.render_widget(block, rect);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(r_inner);

        let r_msg = chunks[0];
        let r_btn = chunks[1];

        if r_msg.area() == 0 || r_btn.area() == 0 { return; }

        let text = Text::from(&*self.msg);
        // estimate the lines
        let lines = (text.width() as f32 / r_msg.width as f32).min(1.0).ceil() as u16;
        let r_msg = r_msg.inner(&Margin {
            vertical: r_msg.height.saturating_sub(lines) / 2,
            horizontal: 0,
        });
        f.render_widget(Paragraph::new(&*self.msg).alignment(Alignment::Center).wrap(Wrap { trim: true }), r_msg);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(r_btn);
        self.r_ok = chunks[0]; self.r_no = chunks[1];

        let mut b_ok = Paragraph::new("Ok").alignment(Alignment::Center); if self.state == 0 { b_ok = b_ok.style(theme.btn()); }
        let mut b_no = Paragraph::new("No").alignment(Alignment::Center); if self.state == 1 { b_no = b_no.style(theme.btn()); }
        f.render_widget(b_ok, self.r_ok);
        f.render_widget(b_no, self.r_no);
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                self.state = 2;
                self.exit = true;
            },
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                self.exit = true;
            },
            Event::Key(KeyEvent { code: KeyCode::Right | KeyCode::Char('l'), .. }) => {
                self.state += 1;
                self.state = self.state.min(1);
            },
            Event::Key(KeyEvent { code: KeyCode::Left | KeyCode::Char('h'), .. }) => {
                self.state = self.state.saturating_sub(1);
            },
            Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
                self.state += 1;
                if self.state > 1 { self.state = 0; }
            },
            Event::Click(column, row) => {
                if self.r_ok.contain(column, row) {
                    self.state = 0;
                    self.exit = true;
                } else if self.r_no.contain(column, row) {
                    self.state = 1;
                    self.exit = true;
                } else if !self.r_wd.contain(column, row) {
                    self.state = 2;
                    self.exit = true;
                }
            },
            _ => {  }
        }
    }
}

impl Popup for InputPanel {
    fn suit_in(&self, rect: Rect) -> Rect { rect }

    fn emit(self) -> Option<Event> { todo!() }
    fn exit(&self) -> bool { todo!() }
}

impl Ui for InputPanel {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) { todo!() }
}

impl Popup for HelpPanel {
    fn suit_in(&self, rect: Rect) -> Rect {
        let width = rect.width * 1 / 3;
        let height = rect.height * 5 / 6;
        rect.inner(&Margin {
            vertical: (rect.height.saturating_sub(height) / 2).max(3),
            horizontal: (rect.width.saturating_sub(width) / 2).max(3),
        })
    }

    fn emit(self) -> Option<Event> {
        None
    }

    fn exit(&self) -> bool {
        self.exit
    }
}

impl Ui for HelpPanel {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) {
        self.rect = rect;
        let block = Block::default().title("Help").borders(Borders::ALL).border_style(theme.border_active());
        let r_inner = block.inner(rect);
        f.render_widget(block, rect);

        f.render_widget(
            Table::new(vec![
                // Row can be created from simple strings.
                Row::new(vec!["Row11", "Row12", "Row13"]),
                // You can style the entire row.
                Row::new(vec!["Row21", "Row22", "Row23"]).style(Style::default().fg(Color::Blue))]
            ).widths(&[Constraint::Length(5), Constraint::Length(5), Constraint::Length(10)]),
            r_inner
        );
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                self.exit = true;
            },
            // Event::Key(KeyEvent { code: KeyCode::Right | KeyCode::Char('l'), .. }) => {
            //     self.state += 1;
            //     self.state = self.state.min(1);
            // },
            // Event::Key(KeyEvent { code: KeyCode::Left | KeyCode::Char('h'), .. }) => {
            //     self.state = self.state.saturating_sub(1);
            // },
            Event::Click(column, row) => {
                if !self.rect.contain(column, row) {
                    self.exit = true;
                }
            },
            _ => {  }
        }
    }
}


