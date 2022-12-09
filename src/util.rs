use tui::text::{Text, Span, Spans};
use tui::layout::Rect;
use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

// TODO: use Iterator or Generator is more permenant
pub fn crop_line<'a>(mut line: Spans<'a>, mut offset: usize) -> Spans<'a> {
    for span in std::mem::replace(&mut line, Default::default()).0.into_iter() {
        if offset > 0 {
            if offset >= span.width() {
                offset -= span.width();
            } else {
                let mut word = String::new();

                for character in UnicodeSegmentation::graphemes(span.content.as_ref(), true) {
                    if offset > 0 {
                        if offset >= character.width() {
                            offset -= character.width();
                        } else {
                            word.push_str(&" ".repeat(character.width() - offset));
                            offset = 0;
                        }
                    } else {
                        word.push_str(character);
                    }
                }

                assert_eq!(offset, 0);
                line.0.push(Span { content: Cow::Owned(word), style: span.style });
            }
        } else {
            line.0.push(span)
        }
    }

    line
}

fn wrap_line<'a>(line: Vec<Span<'a>>, max_width: usize) -> Vec<Spans<'a>> {
    let mut lines = vec![];

    let mut spans = vec![];
    let mut position = 0;

    for span in line.into_iter() {
        if position + span.width() > max_width {
            let Span { content, style } = span;
            let mut last = String::new();
            for character in UnicodeSegmentation::graphemes(content.as_ref(), true) {
                if position + character.width() > max_width {
                    spans.push(Span { content: Cow::Owned(std::mem::replace(&mut last, Default::default())), style });
                    if max_width - position > 0 {
                        spans.push(Span::raw(" ".repeat(max_width - position)));
                    }
                    // lines not at end of a paragraph has no alignment and overflow
                    lines.push(Spans(std::mem::replace(&mut spans, Default::default())));

                    // what if this single character is bigger than max_width
                    if character.width() < max_width {
                        last.push_str(character);
                    } else {
                        last.push(' ');
                    }
                    position = last.width();
                } else {
                    last.push_str(character);
                    position += character.width();
                }
            }
            if last.len() > 0 {
                spans.push(Span { content: Cow::Owned(last), style });
            }
        } else {
            position += span.width();
            spans.push(span);
        }
    }

    if spans.len() > 0 {
        lines.push(Spans(spans));
    }

    lines
}

pub fn wrap_text<'a>(text: Text<'a>, max_width: usize, no_wrap: bool, offset: usize) -> Text<'a> {
    let mut lines_wrapped: Vec<Spans<'a>> = vec![];
    for line in text.lines.into_iter() {
        let wrap_overflow = line.width() > max_width;
        let mut extends: Vec<_> = wrap_line(line.0, max_width).into_iter().map(|line| crop_line(line, offset)).collect();
        if no_wrap && extends.len() > 0 {
            let mut fol = extends.remove(0);
            if wrap_overflow {
                if let Some(last_word)  = fol.0.last_mut() {
                    let mut cs: Vec<_> = last_word.content.as_ref().chars().collect();
                    if cs.len() > 0 {
                        let last_character = cs.pop().unwrap();
                        if let Some(width) = last_character.width() {
                            for _ in 0..width.saturating_sub(1) {
                                cs.push(' ');
                            }
                            cs.push('…');
                        } else {
                            cs.push(last_character);
                        }
                    }
                    last_word.content = Cow::Owned(cs.into_iter().collect());
                }
                lines_wrapped.push(fol);
            } else {
                lines_wrapped.push(fol);
            }
        } else {
            lines_wrapped.append(&mut extends);
        }
    }

    Text {
        lines: lines_wrapped
    }
}

fn rect_vsplit(rect: Rect, split: u16) -> (Rect, Rect) {
    let split = split.min(rect.height);
    (
        Rect {
            height: split,
            ..rect.clone()
        },
        Rect {
            y: rect.y.saturating_add(split),
            height: rect.height.saturating_sub(split),
            ..rect
        }
    )
}

fn rect_hsplit(rect: Rect, split: u16) -> (Rect, Rect) {
    let split = split.min(rect.width);
    (
        Rect {
            width: split,
            ..rect.clone()
        },
        Rect {
            x: rect.x.saturating_add(split),
            width: rect.width.saturating_sub(split),
            ..rect
        }
    )
}

fn rect_vmargin(rect: Rect, margin: u16) -> Rect {
    Rect {
        y: rect.y.saturating_add(margin),
        height: rect.height.saturating_sub(margin*2),
        ..rect
    }
}

fn rect_hmargin(rect: Rect, margin: u16) -> Rect {
    Rect {
        x: rect.x.saturating_add(margin),
        width: rect.width.saturating_sub(margin*2),
        ..rect
    }
}

fn rect_contain(rect: Rect, x: u16, y: u16) -> bool {
    return x >= rect.x && x < rect.right() && y >= rect.y && y < rect.bottom()
}

pub struct Split2D {
    pub top_left: Rect,
    pub top_right: Rect,
    pub bottom_left: Rect,
    pub bottom_right: Rect,
}

pub trait RectExt: Into<Rect> {
    fn contain(self, x: u16, y: u16) -> bool {
        rect_contain(self.into(), x, y)
    }

    fn split_2d(self, hsplit: u16, vsplit: u16) -> Split2D {
        let (top, bottom) = self.vsplit(vsplit);
        let (top_left, top_right) = top.hsplit(hsplit);
        let (bottom_left, bottom_right) = bottom.hsplit(hsplit);
        Split2D {top_left, top_right, bottom_left, bottom_right}
    }

    fn hsplits<const N: usize>(self, splits: [u16; N]) -> ([Rect; N], Rect) {
        let mut rect = self.into();
        let mut res = [Rect::default(); N];
        for (idx, split) in splits.into_iter().enumerate() {
            let (tmp_left, temp_right) = rect_hsplit(std::mem::replace(&mut rect, Default::default()), split);
            res[idx] = tmp_left;
            rect = temp_right;
        }
        (res, rect)
    }

    fn vsplits<const N: usize>(self, splits: [u16; N]) -> ([Rect; N], Rect) {
        let mut rect = self.into();
        let mut res = [Rect::default(); N];
        for (idx, split) in splits.into_iter().enumerate() {
            let (tmp_left, temp_right) = rect_vsplit(std::mem::replace(&mut rect, Default::default()), split);
            res[idx] = tmp_left;
            rect = temp_right;
        }
        (res, rect)
    }

    fn hsplit(self, split: u16) -> (Rect, Rect) {
        rect_hsplit(self.into(), split)
    }

    fn vsplit(self, split: u16) -> (Rect, Rect) {
        rect_vsplit(self.into(), split)
    }

    fn vmargin(self, margin: u16) -> Rect {
        rect_vmargin(self.into(), margin)
    }

    fn hmargin(self, margin: u16) -> Rect {
        rect_hmargin(self.into(), margin)
    }

    fn margin(self, margin: u16) -> Rect {
        rect_hmargin(rect_vmargin(self.into(), margin), margin)
    }
}

impl<R: Into<Rect>> RectExt for R {  }

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
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

