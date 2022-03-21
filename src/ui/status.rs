use super::*;

pub struct StatusPanel { }

impl StatusPanel {
    pub fn new() -> Self {
        Self {  }
    }
}

impl Ui for StatusPanel {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, _theme: &Theme) {
        f.render_widget(
            Paragraph::new("HELP: (?)help (q)exit")
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left)
            , rect
        );
    }
}

