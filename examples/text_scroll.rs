use std::{io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Paragraph, Wrap, Gauge, Sparkline, Chart, Axis, Dataset, GraphType, LineGauge},
    symbols,
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    Terminal,
    buffer::Buffer,
};
use tui::text::{Text, Span, Spans};
use tui::style::{Style, Color, Modifier};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::borrow::Cow;
use filez::util::{wrap_text, RectExt, Split2D};
use filez::widgets::SliderBuilder;

fn main() -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (mut offset_x, mut offset_y): (usize, usize) = (8, 0);

    let mut now = std::time::Instant::now();
    const TO_WAIT: std::time::Duration = std::time::Duration::from_millis(15);

    let mut is_exit = false;

    let mut contents = vec![];
    for idx in 0..10 {
        contents.push((
            "The 😠 first line and more line ".repeat(10),
            Style::default().fg(Color::Rgb((255usize * idx / 10) as u8, 127, 127))
        ));
    }
    contents.push((
        "The is the last line".to_owned(), Style::default().fg(Color::Green)
    ));

    loop {
        if is_exit { break; }
        // poll term events
        while let Ok(true) = event::poll(std::time::Duration::from_secs(0)) {
            let eterm = event::read().unwrap();
            match eterm {
                event::Event::Key(event::KeyEvent { code: event::KeyCode::Char('q'), modifiers: event::KeyModifiers::NONE }) => {
                    is_exit = true;
                },
                event::Event::Key(event::KeyEvent { code: event::KeyCode::Down, modifiers: event::KeyModifiers::NONE }) => {
                    offset_y = offset_y.saturating_add(1);
                },
                event::Event::Key(event::KeyEvent { code: event::KeyCode::Up, modifiers: event::KeyModifiers::NONE }) => {
                    offset_y = offset_y.saturating_sub(1);
                },
                event::Event::Key(event::KeyEvent { code: event::KeyCode::Right, modifiers: event::KeyModifiers::NONE }) => {
                    offset_x = offset_x.saturating_add(1);
                },
                event::Event::Key(event::KeyEvent { code: event::KeyCode::Left, modifiers: event::KeyModifiers::NONE }) => {
                    offset_x = offset_x.saturating_sub(1);
                },
                _ => {}
            }
        }

        let elapsed = now.elapsed();
        if TO_WAIT > elapsed {
            std::thread::sleep(TO_WAIT-elapsed);
            now = now + TO_WAIT-elapsed;
        }

        terminal.draw(|f| {
            let Rect { width, height, .. } = f.size();
            let ([area_notify, area_normal], area_compare) = f.size().vsplits([3, (height-3) / 2]);

            let h_compare = area_compare.height - 1;
            let w_vslider = 1;
            let w_compare = width - w_vslider;
            let Split2D {
                top_left: area_compare,
                top_right: area_vslider,
                bottom_left: area_hslider,
                ..
            } = area_compare.split_2d(w_compare, h_compare);

            // let ([area_normal, area_compare], area_hslider) = f.size().vsplits([h_normal, h_compare]);
            // let (area_compare, area_vslider) = area_compare.hsplit(w_compare);
            let area_normal = area_normal.vmargin(3).hmargin(10);

            let text: Text = Text {
                lines: contents.iter().map(|(line, s)| Spans::from(Span::styled(line, *s))).collect()
            };

            offset_y = offset_y.min(text.height());
            offset_x = offset_x.min(text.width());

            f.render_widget(
                Paragraph::new(text.clone())
                .block(Block::default().title("Nomal").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                // .alignment(Alignment::Center)
                // .wrap(Wrap { trim: true })
                ,
                area_normal,
            );

            f.render_widget(
                Paragraph::new(wrap_text(
                        Text { lines: text.lines.clone().into_iter().skip(offset_y).collect() },
                        // you need to manage max_width after offset youself
                        area_compare.width as usize + offset_x - 2, true,
                        offset_x
                ))
                .block(Block::default().title("Compare").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                ,
                area_compare,
            );

            let slider = SliderBuilder::new(offset_y, contents.len()).make(area_vslider, Direction::Vertical).slider_color(Color::Red);
            f.render_widget(
                // Gauge::default()
                // .block(Block::default().borders(Borders::ALL).title("Progress"))
                // .gauge_style(Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::ITALIC))
                // .use_unicode(true)
                // .percent(80)

                // LineGauge::default()
                //     .block(Block::default().borders(Borders::ALL).title("Progress"))
                //     .gauge_style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD))
                //     .line_set(symbols::line::THICK)
                //     .ratio(0.4)

                slider
                ,
                area_vslider
            );

            let slider = SliderBuilder::new(offset_x, text.width()).make(area_hslider, Direction::Horizontal).slider_color(Color::Green);
            f.render_widget(
                Paragraph::new(Text::from(format!("{} {}: {:?}", offset_x, offset_y, slider.size())))
                .block(Block::default().title("notify").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Center)
                // .wrap(Wrap { trim: true })
                ,
                area_notify,
            );

            f.render_widget(
                slider,
                area_hslider,
            );

        })?;
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
