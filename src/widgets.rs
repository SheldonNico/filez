use tui::buffer::Buffer;
use tui::layout::{Direction, Rect, Corner};
use tui::style::{Style, Color};
use tui::symbols;
use tui::text::{Text, Span, Spans};
use tui::widgets::Widget;

use std::borrow::Cow;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// #[derive(Debug, Clone)]
// pub struct Slider {
//     start: f64,
//     end: f64,

//     color: Color,
//     slider_color: Color,
//     direction: Direction,
// }

// #[derive(Debug, Clone, Default)]
// pub struct SliderBuilder {
//     offset: usize,
//     total: usize,
// }

// impl SliderBuilder {
//     pub fn new(offset: usize, total: usize) -> Self {
//         Self { offset, total }
//     }

//     pub fn make(&self, area: Rect, direction: Direction) -> Slider {
//         match direction {
//             Direction::Vertical => {
//                 let len = area.height as f64 / (area.height as f64 + self.total as f64);
//                 let ratio = (self.offset as f64 / self.total as f64).max(0.0).min(1.0) * (1.0 - len);

//                 let mut end_width = f64::from(area.height) * (ratio + len);
//                 let mut start_width = f64::from(area.height) * ratio;
//                 if end_width > area.height as f64 {
//                     end_width = area.height as _;
//                     start_width = end_width - len * f64::from(area.height);
//                 }

//                 start_width = 0f64.max(start_width).min(area.height as f64);
//                 end_width = 0f64.max(end_width).min(area.height as f64);

//                 Slider {
//                     start: start_width, end: end_width, direction, .. Default::default()
//                 }
//             },
//             Direction::Horizontal => {
//                 let len = area.width as f64 / (area.width as f64 + self.total as f64);
//                 let ratio = (self.offset as f64 / self.total as f64).max(0.0).min(1.0) * (1.0 - len);

//                 let mut end_width = f64::from(area.width) * (ratio + len);
//                 let mut start_width = f64::from(area.width) * ratio;
//                 if end_width > area.width as f64 {
//                     end_width = area.width as _;
//                     start_width = end_width - len * f64::from(area.width);
//                 }

//                 start_width = 0f64.max(start_width).min(area.width as f64);
//                 end_width = 0f64.max(end_width).min(area.width as f64);

//                 Slider {
//                     start: start_width, end: end_width, direction, .. Default::default()
//                 }
//             }
//         }
//     }
// }

// impl Slider {
//     pub fn color(mut self, color: Color) -> Self {
//         self.color = color;
//         self
//     }

//     pub fn slider_color(mut self, color: Color) -> Self {
//         self.slider_color = color;
//         self
//     }

//     pub fn size(&self) -> (f64, f64) {
//         (self.start, self.end)
//     }
// }
//
// impl Widget for Slider {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         buf.set_style(area, Style::default().bg(self.color));
//         if area.height < 1 || area.width < 1 {
//             return;
//         }

//         let start = self.start.floor() as u16;
//         let end = self.end.floor() as u16;

//         match self.direction {
//             Direction::Vertical => {
//                 for x in area.left()..area.right() {
//                     for y in start..(end+1).min(area.height) {
//                         let mut symbol = " ";
//                         let mut fg = self.color;
//                         let mut bg = self.slider_color;

//                         if y == start {
//                             symbol = get_unicode_bar(1.0 - (self.start % 1.0));
//                             fg = self.slider_color;
//                             bg = self.color;
//                         }
//                         if y == end {
//                             symbol = get_unicode_bar(1.0 - (self.end % 1.0));
//                         }

//                         buf.get_mut(x, area.top() + y).set_symbol(symbol).set_fg(fg).set_bg(bg);
//                     }
//                 }
//             },
//             Direction::Horizontal => {
//                 for y in area.top()..area.bottom() {
//                     for x in start..(end+1).min(area.width) {
//                         let mut symbol = " ";
//                         let mut fg = self.color;
//                         let mut bg = self.slider_color;

//                         if x == start {
//                             symbol = get_unicode_block(self.start % 1.0);
//                         }
//                         if x == end {
//                             symbol = get_unicode_block(self.end % 1.0);
//                             fg = self.slider_color;
//                             bg = self.color;
//                         }

//                         buf.get_mut(area.left() + x, y).set_symbol(symbol).set_fg(fg).set_bg(bg);
//                     }
//                 }
//             },
//         }
//     }
// }

// impl Default for Slider {
//     fn default() -> Self {
//         Self {
//             start: 0.0,
//             end: 0.0,
//             slider_color: Color::White,
//             color: Color::Black,
//             direction: Direction::Vertical,
//         }
//     }
// }

pub struct Slider {
    offset: usize,
    total: usize,
    slider_color: Color,
    color: Color,
    direction: Direction,
    rev: bool,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            offset: 0,
            total: 0,
            slider_color: Color::Red,
            color: Color::Black,
            direction: Direction::Vertical,
            rev: false,
        }
    }
}

impl Slider {
    pub fn new(offset: usize, total: usize) -> Self {
        Self { offset, total, .. Default::default() }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn slider_color(mut self, color: Color) -> Self {
        self.slider_color = color;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn rev(mut self) -> Self {
        self.rev = !self.rev;
        self
    }

    pub fn positon(&self, area: Rect) -> (f64, f64) {
        let slider_win = match self.direction {
            Direction::Vertical => area.height as f64,
            Direction::Horizontal => area.width as f64,
        };
        let offset = self.offset as f64;
        let total = self.total as f64;

        if offset >= total || slider_win >= total { return (0.0, 0.0); }
        let slider_len = slider_win * (slider_win as f64 / total);

        let (_inner_start, inner_end) = (0.0, self.total as f64);
        let (_outer_start, outer_end) = (0.0, slider_win - slider_len);
        let rel_start = (outer_end * offset / inner_end).max(0.0).min(slider_win);
        let rel_end = (rel_start + slider_len).max(0.0).min(slider_win);
        if !self.rev {
            return (rel_start, rel_end)
        } else {
            return (slider_win - rel_end, slider_win - rel_start);
        }
    }
}

impl Widget for Slider {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(self.color));
        if area.height < 1 || area.width < 1 {
            return;
        }

        let (rel_start, rel_end) = self.positon(area);

        let start = rel_start.floor() as u16;
        let end = rel_end.floor() as u16;

        match self.direction {
            Direction::Vertical => {
                for x in area.left()..area.right() {
                    for y in start..(end+1).min(area.height) {
                        let mut symbol = get_unicode_bar(1.0);
                        let mut bg = self.color;
                        let mut fg = self.slider_color;

                        if y == start {
                            symbol = get_unicode_bar(1.0 - (rel_start % 1.0));
                        }
                        if y == end {
                            symbol = get_unicode_bar(1.0 - (rel_end % 1.0));
                            bg = self.slider_color;
                            fg = self.color;
                        }

                        buf.get_mut(x, area.top() + y).set_symbol(symbol).set_fg(fg).set_bg(bg);
                    }
                }
            },
            Direction::Horizontal => {
                for y in area.top()..area.bottom() {
                    for x in start..(end+1).min(area.width) {
                        let mut symbol = get_unicode_block(1.0);
                        let mut bg = self.color;
                        let mut fg = self.slider_color;

                        if x == start {
                            symbol = get_unicode_block(rel_start % 1.0);
                            bg = self.slider_color;
                            fg = self.color;
                        }
                        if x == end {
                            symbol = get_unicode_block(rel_end % 1.0);
                        }

                        buf.get_mut(area.left() + x, y).set_symbol(symbol).set_fg(fg).set_bg(bg);
                    }
                }
            }
        }
    }

}


pub fn get_unicode_bar<'a>(frac: f64) -> &'a str {
    match (frac * 8.0).round() as u16 {
        1 => symbols::bar::ONE_EIGHTH,
        2 => symbols::bar::ONE_QUARTER,
        3 => symbols::bar::THREE_EIGHTHS,
        4 => symbols::bar::HALF,
        5 => symbols::bar::FIVE_EIGHTHS,
        6 => symbols::bar::THREE_QUARTERS,
        7 => symbols::bar::SEVEN_EIGHTHS,
        8 => symbols::bar::FULL,
        _ => " ",
    }
}

pub fn get_unicode_block<'a>(frac: f64) -> &'a str {
    match (frac * 8.0).round() as u16 {
        1 => symbols::block::ONE_EIGHTH,
        2 => symbols::block::ONE_QUARTER,
        3 => symbols::block::THREE_EIGHTHS,
        4 => symbols::block::HALF,
        5 => symbols::block::FIVE_EIGHTHS,
        6 => symbols::block::THREE_QUARTERS,
        7 => symbols::block::SEVEN_EIGHTHS,
        8 => symbols::block::FULL,
        _ => " ",
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub const ELLIPSES: char = '…';

pub struct List2<'a> {
    rows: Vec<Text<'a>>,
    offset_x: usize,
    wrap: Option<usize>,
    truncator: Option<char>,
    start_corner: Corner,
}

impl<'a> List2<'a> {
    pub fn wrap(mut self, width: usize) -> Self {
        self.wrap = Some(width);
        self
    }

    pub fn offset_x(mut self, offset: usize) -> Self {
        self.offset_x = offset;
        self
    }

    pub fn truncator(mut self, symbol: char) -> Self {
        self.truncator = Some(symbol);
        self
    }

    pub fn start_corner(mut self, corner: Corner) -> Self {
        self.start_corner = corner;
        self
    }

    pub fn new<I: IntoIterator<Item = Text<'a>>>(
        rows: I, offset_y: usize, height: usize,
    ) -> Self {
        Self {
            rows: rows.into_iter().skip(offset_y).take(height).collect(),
            offset_x: 0,
            wrap: None,
            truncator: None,
            start_corner: Corner::TopLeft,
        }
    }

    pub fn get_row_heights(text: &Text, wrap: usize, is_wrap: bool) -> Vec<usize> {
        text.lines.iter().map(|line| {
            if is_wrap {
                line.width() / wrap + if line.width() % wrap > 0 { 1 } else { 0 }
            } else {
                1
            }
        }).collect()
    }
}

impl<'a> Widget for List2<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 1 || area.height < 1 { return; }

        let is_wrap = self.wrap.is_some();
        let wrap = self.wrap.unwrap_or(area.width as usize);
        if wrap < 1 { return; }

        let mut total_height: usize = 0;
        for item_text in self.rows.into_iter().take(area.height as usize) {
            let heights = Self::get_row_heights(&item_text, wrap as usize, is_wrap);

            let item_height = heights.iter().sum::<usize>();
            let mut line_position = 0usize;
            for (line_heights, spans) in heights.into_iter().zip(item_text.lines.into_iter()) {
                let mut cursor = (0, spans);
                for _ in 0..line_heights {
                    cursor = crop_left(cursor.1, (self.offset_x as usize).saturating_sub(cursor.0) );
                    let (x, y) = match self.start_corner {
                        Corner::BottomLeft => {
                            let line_abs_position = (total_height + item_height - line_position) as u16;
                            (area.left() + cursor.0 as u16, area.bottom().saturating_sub(line_abs_position))
                        },
                        _ => {
                            let line_abs_position = (total_height + line_position) as u16;
                            (area.left() + cursor.0 as u16, area.top().saturating_add(line_abs_position))
                        },
                    };
                    if x >= area.left() && x < area.right() && y < area.bottom() && y >= area.top() {
                        let room_wrap = (wrap - self.offset_x) as u16;
                        let room_rect = area.right().saturating_sub(x) as u16;
                        buf.set_spans(x, y, &cursor.1, room_wrap.min(room_rect));
                    }
                    cursor = crop_left(cursor.1, (wrap).saturating_sub(self.offset_x).saturating_sub(cursor.0) as usize);

                    line_position += 1;
                }
            }

            total_height += item_height;
        }
    }
}

pub struct Table2<'a> {
    items: Vec<Vec<Text<'a>>>,
    widths: Vec<u16>,
    heights: Vec<u16>,
    offset_x: usize,
    column_spacing: u16
}

impl<'a> Table2<'a> {
    pub fn offset_x(mut self, offset: usize) -> Self {
        self.offset_x = offset;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    pub fn new<T: IntoIterator<Item = R>, R: IntoIterator<Item = Text<'a>>, H: IntoIterator<Item = Text<'a>>>(
        rows: T, header: H, offset_y: usize, height: usize
    ) -> Table2<'a> {
        let mut curr_height: usize = 0;
        let mut widths: Vec<u16> = vec![];
        let mut heights: Vec<u16> = vec![];

        let mut item = vec![];
        let mut row_height: usize = 0;
        for col in header.into_iter() {
            widths.push(col.width() as u16);
            row_height = row_height.max(col.height());
            item.push(col);
        }
        let mut items = vec![item];
        curr_height += row_height;
        heights.push(row_height as u16);

        for row in rows.into_iter().skip(offset_y) {
            if curr_height >= height { break; }
            row_height = 0;
            let mut item = vec![];
            for (cell, width) in row.into_iter().zip(widths.iter_mut()) {
                *width = (*width).max(cell.width() as u16);
                row_height = row_height.max(cell.height());
                item.push(cell);
            }
            items.push(item);
            curr_height += row_height;
            heights.push(row_height as u16);
        }

        Table2 {
            items,
            widths,
            heights,
            offset_x: 0,
            column_spacing: 1,
        }
    }
}

pub fn crop_left_span<'a>(Span { mut content, style }: Span<'a>, mut crop: usize) -> (usize, Span<'a>) {
    let original = std::mem::replace(&mut content, Default::default());
    for (idx, character) in UnicodeSegmentation::grapheme_indices(original.as_ref(), true) {
        if character.width() > crop {
            let split = if crop == 0 {
                idx
            } else {
                crop = character.width() - crop;
                idx + character.len()
            };

            content = match original {
                Cow::Owned(c) => Cow::Owned(c[split..].to_string()),
                Cow::Borrowed(c) => Cow::Borrowed(&c[split..])
            };
            break;
        } else {
            crop = crop.saturating_sub(character.width());
        }
    }

    (crop, Span { content, style })
}

pub fn crop_left<'a>(mut spans: Spans<'a>, mut crop: usize) -> (usize, Spans<'a>) {
    let mut cropped = false;
    for span in std::mem::replace(&mut spans.0, Default::default()) {
        if cropped {
            spans.0.push(span);
        } else {
            if crop <= span.content.width() {
                let (spaces, newspan) = crop_left_span(span, crop);
                crop = spaces;
                spans.0.push(newspan);
                cropped = true;
            } else {
                crop -= span.content.width();
            }
        }
    }

    (crop, spans)
}

impl<'a> Widget for Table2<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut y = 0;
        let offset_x = self.offset_x as u16;
        for (row, &height) in self.items.into_iter().zip(self.heights.iter()) {
            if y >= area.height { break; }

            let mut x: u16 = 0;
            for (cell, &width) in row.into_iter().zip(self.widths.iter()) {
                for (offset_y, line) in cell.lines.into_iter().enumerate() {
                    if y + offset_y as u16 >= area.height { break; }
                    if x >= offset_x && x - offset_x < area.width {
                        buf.set_spans(area.left() + x - offset_x, area.top() + y + offset_y as u16, &line, width);
                    } else if x < offset_x && width > offset_x - x {
                        let width_cropped = width.saturating_sub(offset_x - x);
                        let (pad, rest) = crop_left(line, (offset_x - x) as usize); let pad = pad as u16;
                        buf.set_spans(
                            area.left() + pad,
                            area.top() + y + offset_y as u16,
                            &rest,
                            width_cropped.saturating_sub(pad)
                        );
                    }
                }

                x += width;
                x += self.column_spacing;
            }
            y += height;
        }
    }
}

