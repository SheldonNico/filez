use super::*;
use crate::fs::{Permissions, File, FileType, Metadata, LocalHost, Host};

use std::path::{Path, PathBuf};
use lazy_static::lazy_static;
use chrono::Datelike;
use number_prefix::NumberPrefix;

lazy_static! {
    static ref CURRENT_YEAR: i32 = chrono::Local::now().year();
}

#[derive(Default)]
pub struct ListView<T> {
    pub rows: Vec<T>,
    offset: usize,
    select: usize,

    rect: Rect,
}

impl<T> ListView<T> {
    fn select_left(&self) -> usize {
        (self.rect.height as usize).min(self.rows.len().saturating_sub(self.offset))
    }

    pub fn view(&self) -> (&[T], usize) {
        (&self.rows[self.offset..self.offset.saturating_add(self.select_left())], self.select)
    }

    pub fn suit(&mut self, rect: Rect) {
        self.rect = rect;
        self.adjust();
    }

    pub fn new() -> Self {
        Self {
            rows: Vec::new(), offset: 0, select: 0, rect: Rect::default()
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    pub fn adjust(&mut self) {
        self.adjust_scroll();
        self.adjust_select();
    }

    fn adjust_scroll(&mut self) {
        self.offset = self.offset.min(self.rows.len().saturating_sub(1));
    }

    fn adjust_select(&mut self) {
        let row_selectable = self.rows.len().saturating_sub(self.offset).min(self.rect.height as usize);
        self.select = self.select.min(row_selectable.saturating_sub(1));
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    pub fn go_select_up(&mut self) {
        if self.select > 0 {
            self.select -= 1;
        } else {
            self.go_scroll_up();
        }
    }

    pub fn go_select_down(&mut self) {
        if self.select + 1 == self.select_left() {
            self.go_scroll_down();
        } else {
            self.select = self.select.saturating_add(1);
            self.adjust();
        }
    }

    fn go_select(&mut self, (_, y): (u16, u16)) {
        if y >= self.rect.top() && y <= self.rect.bottom() {
            let to_select = y.saturating_sub(self.rect.y) as usize;
            if self.offset.saturating_add(to_select) < self.rows.len() {
                self.select = to_select;
                self.adjust();
            }
        }
    }

    pub fn go_scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
        self.adjust();
    }

    pub fn go_scroll_down(&mut self) {
        self.offset = self.offset.saturating_add(1);
        self.adjust();
    }

    pub fn go_scroll(&mut self, mut scroll: isize) {
        while scroll > 0 {
            self.go_scroll_down();
            scroll -= 1;
        }
        while scroll < 0 {
            self.go_scroll_up();
            scroll += 1;
        }
    }

    pub fn go_top(&mut self) {
        self.offset = 0;
        self.select = 0;
    }

    pub fn go_bottom(&mut self) {
        let bottom = self.rows.len().saturating_sub(self.rect.height as usize);
        if self.offset < bottom { self.offset = bottom; }
        self.select = (self.rect.height as usize).saturating_sub(1);
        self.adjust();
    }

    pub fn go_scroll_down_page(&mut self) {
        self.go_scroll(self.rect.height as isize / 2);
    }

    pub fn go_scroll_up_page(&mut self) {
        self.go_scroll(-(self.rect.height as isize / 2));
    }
}

pub struct ExplorerPanel<H> {
    host: H,
    list: ListView<RowFile>,

    dir: PathBuf,

    show_hidden: bool,
    sort_by: &'static str,
    sort_reverse: bool,
}

impl<H: Host> ExplorerPanel<H> {
    pub fn new<P: AsRef<Path>>(host: H, dir: P) -> io::Result<Self> {
        let mut slf = Self {
            host,
            list: ListView::new(),
            dir: dir.as_ref().to_path_buf(),

            show_hidden: false,
            sort_by: "name",
            sort_reverse: false,
        };

        slf._refresh()?;
        Ok(slf)
    }

    fn _refresh(&mut self) -> io::Result<()> {
        self.dir = self.dir.canonicalize()?;
        self.list.rows.clear();
        let mut rows = vec![];
        for file in self.host.read_dir(&self.dir)? {
            if !self.show_hidden {
                if let Some(stem) = file.path.file_stem() {
                    if let Some(stem) = stem.to_str() {
                        if stem.starts_with(".") {
                            continue;
                        }
                    }
                }
            }

            rows.push(RowFile { file, matched: None, mark: Mark::None });
        }

        if rows.len() > 0 {
            rows[1..].sort_by(|r1, r2| {
                let f1 = &r1.file;
                let f2 = &r2.file;

                let r = match self.sort_by {
                    "name"     => std::cmp::Ord::cmp(&(f1.file_type, &f1.name),             &(f2.file_type, &f2.name)),
                    "size"     => std::cmp::Ord::cmp(&(f1.file_type, f1.metadata.len),      &(f2.file_type, f2.metadata.len)),
                    "modified" => std::cmp::Ord::cmp(&(f1.file_type, f1.metadata.modified), &(f2.file_type, f2.metadata.modified)),
                    "type"     => std::cmp::Ord::cmp(&(f1.file_type, f1.metadata.len),      &(f2.file_type, f2.metadata.len)),
                    _          => unreachable!(),
                };
                if self.sort_reverse { r.reverse() } else { r }
            });
        }

        self.list.rows = rows; self.list.adjust();

        Ok(())
    }

    pub fn refresh(&mut self) {
        if let Err(e) = self._refresh() {
            log::error!("Fail to refresh into `{}`: {:?}", self.dir.display(), e);
        }
    }
}

impl<H: Host> Ui for ExplorerPanel<H> {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(rect);

        let r_table = chunks[0];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45),
                Constraint::Percentage(10),
                Constraint::Percentage(45),
            ])
            .split(chunks[2]);
        let r_dir = chunks[0]; let r_opt = chunks[2];

        self.list.suit(Rect { y: r_table.y.saturating_add(1), height: r_table.height.saturating_sub(1), .. r_table });
        let mut rows = vec![];

        const HEADER: [&'static str; 7] = ["", "name", "size", "modified", "permissions", "type", "action", ];
        let mut width: Vec<_> = HEADER.iter().map(|c| c.len() + 1).collect();
        let (rfs, select) = self.list.view();
        for (idx, rf) in rfs.iter().enumerate() {
            let row = rf.to_row(theme, idx == select);
            for (col, width) in row.iter().zip(width.iter_mut()) {
                *width = (*width).max(col.width())
            }

            rows.push(Row::new(row));
        }

        f.render_widget(
            Table::new(rows).header(
                Row::new(HEADER)).widths(&width.into_iter().map(|w| Constraint::Length(w as u16)).collect::<Vec<_>>()),
            r_table,
        );

        f.render_widget(
            Paragraph::new(Spans::from(vec![
                Span::raw(" "),
                Span::styled(format!("{}", self.dir.display()), Style::default().fg(Color::Red)),
            ])).alignment(Alignment::Left),
            r_dir,
        );

        f.render_widget(
            Paragraph::new(Spans::from(vec![
                Span::styled("Hidden", if self.show_hidden { Style::default().bg(Color::Red) } else { Style::default() } ),
                Span::raw(" "),
                Span::raw("Sort: name"),
                Span::raw(" "),
            ])).alignment(Alignment::Right),
            r_opt,
        );
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent { code: KeyCode::Up | KeyCode::Char('k'), modifiers: KeyModifiers::NONE }) => {
                self.list.go_select_up();
            },
            Event::Key(KeyEvent { code: KeyCode::Down | KeyCode::Char('j'), modifiers: KeyModifiers::NONE }) => {
                self.list.go_select_down();
            },
            Event::Key(KeyEvent { code: KeyCode::Char('.'), modifiers: KeyModifiers::NONE }) => {
                self.dir = PathBuf::from(".");
                self.refresh();
                self.list.go_select_down();
            },
            Event::Key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE }) => {
                let (rfs, select) = self.list.view();
                if select >= rfs.len() { return; }
                let file_type = rfs[select].file.file_type;
                let file_path = rfs[select].file.path.clone();

                match file_type {
                    FileType::Dir | FileType::DotDot => {
                        self.dir = file_path;
                        self.refresh();
                    },
                    _ => {}
                }
            },
            Event::Key(KeyEvent { code: KeyCode::Backspace, modifiers: KeyModifiers::NONE }) => {
                self.dir = self.dir.join("..").clone(); self.refresh();
            },
            Event::Key(KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL }) => {
                self.list.go_scroll_up_page();
            },
            Event::Key(KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL }) => {
                self.list.go_scroll_down_page();
            },
            Event::Key(KeyEvent { code: KeyCode::Char('x'), modifiers: KeyModifiers::NONE }) => {
                log::info!("Executing tasks...");
            },
            Event::Key(KeyEvent { code: KeyCode::Char('U'), modifiers: KeyModifiers::SHIFT }) => {
                let rf = &mut self.list.rows[self.list.offset.saturating_add(self.list.select)];
                rf.mark = Mark::Upload;
            },
            Event::Key(KeyEvent { code: KeyCode::Char('D'), modifiers: KeyModifiers::SHIFT }) => {
                let rf = &mut self.list.rows[self.list.offset.saturating_add(self.list.select)];
                rf.mark = Mark::Delete;
            },
            Event::Key(KeyEvent { code: KeyCode::Char('C'), modifiers: KeyModifiers::SHIFT }) => {
                let rf = &mut self.list.rows[self.list.offset.saturating_add(self.list.select)];
                rf.mark = Mark::None;
            },
            Event::ScrollDown => { self.list.go_scroll_down(); },
            Event::ScrollUp => { self.list.go_scroll_up(); },
            Event::Click(col, row) => {
                self.list.go_select((col, row));
            },
            Event::Keys_gg => { self.list.go_top(); }
            Event::Keys_G  => { self.list.go_bottom(); },
            Event::Search(search) => {
                for rf in self.list.rows.iter_mut() {
                    if let Some((idx, _)) = rf.file.name.match_indices(&search).next() {
                        rf.matched = Some((idx, idx+search.len()));
                    } else {
                        rf.matched = None;
                    }
                }
            },
            Event::Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL }) => {
                for rf in self.list.rows.iter_mut() {
                    rf.matched = None;
                }
            },

            // TODO: need better search movement
            Event::Key(KeyEvent { code: KeyCode::Char('n'), modifiers: KeyModifiers::NONE }) => {
                let offset_old = self.list.offset; let select_old = self.list.select;
                let mut restore = true;
                loop {
                    self.list.go_select_down();
                    let (view, idx) = self.list.view();
                    if idx == 0 || idx + 1 >= view.len(){ break; }
                    if view[idx].matched.is_some()  { restore = false; break; }
                }

                if restore {
                    self.list.offset = offset_old;
                    self.list.select = select_old;
                }
            },
            Event::Key(KeyEvent { code: KeyCode::Char('N'), modifiers: KeyModifiers::SHIFT }) => {
                let offset_old = self.list.offset; let select_old = self.list.select;
                let mut restore = true;
                loop {
                    self.list.go_select_up();
                    let (view, idx) = self.list.view();
                    if idx == 0 || idx + 1 >= view.len(){ break; }
                    if view[idx].matched.is_some()  { restore = false; break; }
                }

                if restore {
                    self.list.offset = offset_old;
                    self.list.select = select_old;
                }
            },
            _                => { },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Mark {
    None,
    Delete,
    Upload,
}

pub struct RowFile {
    file: File,
    matched: Option<(usize, usize)>,
    mark: Mark
}

impl Mark {
    fn symbol(&self) -> &'static str {
        match self {
            Self::Delete => "Delete",
            Self::Upload => "Upload",
            Self::None   => "",
        }
    }
}

impl RowFile {
    fn to_row<S: FeStyles>(&self, style: &S, select: bool) -> [Spans<'_>; 7] {
        let File { name, file_type, metadata, ext, .. } = &self.file;
        let file_type = *file_type;
        let &Metadata { len, modified, permissions, .. } = metadata;

        [
            Spans::from(if select { Span::styled(" ", style.cursor_select()) } else { Span::raw(" ") }),
            Spans::from(render_name(name, self.matched, style, file_type)),
            Spans::from(render_size(len, style, file_type)),
            Spans::from(render_datetime(modified, style)),
            Spans::from(render_permission(permissions, style, file_type)),
            Spans::from(render_type(file_type, ext.as_ref(), style)),
            Spans::from(Span::styled(self.mark.symbol(), style.mark())),
        ]
    }
}

pub fn render_type<'n, S: FeStyles>(file_type: FileType, ext: Option<&'n String>, style: &S) -> Span<'n> {
    if file_type == FileType::File {
        if let Some(ext) = ext {
            return Span::raw(ext);
        }
    }
    Span::raw("")
}

pub fn render_datetime<S: FeStyles>(dt: Option<chrono::DateTime<chrono::Local>>, style: &S) -> Span<'static> {
    match dt {
        None => Span::raw(""),
        Some(dt) => if dt.year() == *CURRENT_YEAR {
            Span::styled(dt.format("%_d %b %H:%M").to_string(), style.datetime())
        } else {
            Span::styled(dt.format("%_d %b %Y").to_string(), style.datetime())
        },
    }
}

pub fn render_name<'n, S: FeStyles>(name: &'n str, matched: Option<(usize, usize)>, style: &S, file_type: FileType) -> Vec<Span<'n>> {
    let normal = match file_type {
        FileType::DotDot  => style.dot_dot(),
        FileType::Dir     => style.directory(),
        FileType::File    => style.file(),
        FileType::Other   => style.other(),
        FileType::SymLink => style.sym_link(),
    };

    if let Some((sidx, eidx)) = matched {
        vec![
            Span::styled(&name[..sidx], normal),
            Span::styled(&name[sidx..eidx], style.highlight()),
            Span::styled(&name[eidx..], normal),
        ]
    } else {
        vec![ Span::styled(name, normal) ]

    }

}

pub fn render_permission<S: FeStyles>(p: Permissions, style: &S, file_type: FileType) -> Vec<Span<'static>> {
    let bit = |b, c, s| if b { Span::styled(c, s) } else { Span::styled("-", style.dashed()) };
    let ft = match file_type {
        FileType::DotDot  => { return vec![]; },
        FileType::Dir     => Span::styled("d", style.directory()),
        FileType::File    => Span::styled(".", style.file()),
        FileType::SymLink => Span::styled("s", style.sym_link()),
        FileType::Other   => Span::styled("?", style.other()),
    };

    vec![
        ft,
        bit(p.user_read,     "r", style.user_read()),
        bit(p.user_write,    "w", style.user_write()),
        bit(p.user_execute,  "x", style.user_execute()),
        bit(p.group_read,    "r", style.group_read()),
        bit(p.group_write,   "w", style.group_write()),
        bit(p.group_execute, "x", style.group_execute()),
        bit(p.other_read,    "r", style.other_read()),
        bit(p.other_write,   "w", style.other_write()),
        bit(p.other_execute, "x", style.other_execute()),
    ]
}

pub fn render_size<S: FeStyles>(size: u64, style: &S, file_type: FileType) -> Span<'static> {
    match file_type {
        FileType::File => {
            let num = match NumberPrefix::binary(size as f64) {
                NumberPrefix::Standalone(_)       => size.to_string(),
                NumberPrefix::Prefixed(prefix, s) => format!("{:.0}{}", s, prefix.symbol())
            };
            Span::styled(num, style.size())
        },
        FileType::DotDot => Span::raw(""),
        _ => Span::styled("-", style.dashed()),
    }
}

