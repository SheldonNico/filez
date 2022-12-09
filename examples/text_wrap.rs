use std::{io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    Terminal,
};
use tui::text::{Text, Span, Spans};
use tui::style::{Style, Color, Modifier};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::borrow::Cow;
use filez::util::wrap_text;

fn main() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (offset_x, offset_y): (usize, usize) = (8, 9);

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                Constraint::Percentage(50),
                Constraint::Percentage(50)
                ].as_ref()
            )
            .split(f.size());


        let mut text = Text::default();
        for idx in 0..10 {
            text.extend(Text::styled(
                    "The 😠 first line and more line ".repeat(10), Style::default().fg(Color::Rgb((255usize * idx / 10) as u8, 127, 127))
            ));
        }
        text.extend(Text::styled(
                "The is the last line", Style::default().fg(Color::Green)
        ));

        f.render_widget(
            Paragraph::new(text.clone())
            .block(Block::default().title("Paragraph").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            // .alignment(Alignment::Center)
            // .wrap(Wrap { trim: true })
            ,
            chunks[0]);

        let Rect { width, .. } = chunks[1].clone();
        f.render_widget(
            Paragraph::new(wrap_text(
                    Text { lines: text.lines.into_iter().skip(offset_y).collect() },
                    // you need to manage max_width after offset youself
                    width as usize + offset_x - 2, true,
                    offset_x
            ))
            .block(Block::default().title("Paragraph").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            , chunks[1]);
    })?;

    while let Ok(eterm) = event::read() {
        match eterm {
            event::Event::Key(event::KeyEvent { code: event::KeyCode::Char('q'), modifiers: event::KeyModifiers::NONE }) => { break; },
            _ => {}
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
