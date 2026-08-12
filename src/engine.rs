use crossterm::{
    cursor, execute, queue,
    style::{self, Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::ops::{Index, IndexMut};
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A single cell in the rendering grid
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub visible: bool,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            fg: None,
            bg: None,
            visible: false,
        }
    }
}

/// Rendering grid containing rows of cells
pub struct Grid {
    pub cells: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn from_input(input: &str) -> Self {
        // Strip ANSI escape sequences from input
        let stripped = strip_ansi(input);
        let lines: Vec<&str> = stripped.lines().collect();
        let height = lines.len();

        // Calculate max grapheme width per row (Unicode grapheme clusters)
        let width = lines
            .iter()
            .map(|l| l.graphemes(true).count())
            .max()
            .unwrap_or(0);

        let mut cells = Vec::with_capacity(height);

        for line in &lines {
            let mut row = Vec::with_capacity(width);
            for g in line.graphemes(true) {
                let ch = g.chars().next().unwrap_or(' ');
                row.push(Cell::new(ch));
            }
            // Pad remaining row cells to match grid width
            while row.len() < width {
                row.push(Cell::new(' '));
            }
            cells.push(row);
        }

        Grid {
            cells,
            width,
            height,
        }
    }

    /// Check if all cells in grid are visible
    pub fn all_visible(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|c| c.visible))
    }

    /// Set all cells to visible
    pub fn set_all_visible(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                cell.visible = true;
                cell.fg = None;
                cell.bg = None;
            }
        }
    }

    /// Set all cells to invisible
    pub fn set_all_invisible(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                cell.visible = false;
                cell.fg = None;
                cell.bg = None;
            }
        }
    }

    /// Get all non-space character positions (y, x)
    pub fn char_positions(&self) -> Vec<(usize, usize)> {
        let mut pos = Vec::new();
        for (y, row) in self.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell.ch != ' ' {
                    pos.push((y, x));
                }
            }
        }
        pos
    }

    /// Get all character positions including spaces (y, x)
    pub fn all_positions(&self) -> Vec<(usize, usize)> {
        let mut pos = Vec::with_capacity(self.width * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                pos.push((y, x));
            }
        }
        pos
    }

    /// Render current grid frame as an ANSI string buffer (ideal for Ratatui / TUI framework integration)
    pub fn render_to_string(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height * 12);
        let mut last_fg: Option<Color> = None;
        let mut last_bg: Option<Color> = None;

        for (i, row) in self.cells.iter().enumerate() {
            for cell in row {
                if cell.visible {
                    if cell.fg != last_fg {
                        if let Some(fg) = cell.fg {
                            match fg {
                                Color::Rgb { r, g, b } => {
                                    out.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                                }
                                Color::Reset => {
                                    out.push_str("\x1b[39m");
                                }
                                _ => {}
                            }
                        } else {
                            out.push_str("\x1b[39m");
                        }
                        last_fg = cell.fg;
                    }
                    if cell.bg != last_bg {
                        if let Some(bg) = cell.bg {
                            match bg {
                                Color::Rgb { r, g, b } => {
                                    out.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
                                }
                                Color::Reset => {
                                    out.push_str("\x1b[49m");
                                }
                                _ => {}
                            }
                        } else {
                            out.push_str("\x1b[49m");
                        }
                        last_bg = cell.bg;
                    }
                    out.push(cell.ch);
                } else {
                    if last_fg.is_some() || last_bg.is_some() {
                        out.push_str("\x1b[0m");
                        last_fg = None;
                        last_bg = None;
                    }
                    out.push(' ');
                }
            }
            if i < self.cells.len() - 1 {
                out.push('\n');
            }
        }
        if last_fg.is_some() || last_bg.is_some() {
            out.push_str("\x1b[0m");
        }
        out
    }
}

/// Allow indexing grid directly via grid[y][x]
impl Index<usize> for Grid {
    type Output = Vec<Cell>;

    fn index(&self, y: usize) -> &Self::Output {
        &self.cells[y]
    }
}

/// Allow mutable indexing grid directly via grid[y][x]
impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, y: usize) -> &mut Self::Output {
        &mut self.cells[y]
    }
}

/// FSM-based ANSI stripper supporting CSI and OSC sequences
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some(&'[') => {
                    // CSI escape sequence \x1b[ ... [0-9;]*[mK...a-zA-Z]
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c.is_ascii() && (0x40..=0x7E).contains(&(c as u8)) {
                            break;
                        }
                    }
                }
                Some(&']') => {
                    // OSC escape sequence \x1b] ... (\x07 or \x1b\)
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some(&'(') | Some(&')') => {
                    chars.next();
                    chars.next();
                }
                _ => {}
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Render a single frame, repositioning cursor to `origin_row`.
/// Uses synchronized update DEC private mode 2026 (\x1b[?2026h / \x1b[?2026l)
/// to prevent terminal tearing and flickering in modern terminals (Kitty, Alacritty, WezTerm, Ghostty).
pub fn render_frame(
    grid: &Grid,
    out: &mut BufWriter<io::Stdout>,
    origin_row: u16,
    term_width: u16,
) {
    // Begin synchronized update (DEC private mode 2026)
    out.write_all(b"\x1b[?2026h").ok();

    queue!(out, cursor::MoveTo(0, origin_row)).ok();

    let mut last_fg: Option<Color> = Some(Color::Reset);
    let mut last_bg: Option<Color> = Some(Color::Reset);

    for (i, row) in grid.cells.iter().enumerate() {
        let mut col = 0u16;

        for cell in row {
            let mut s = [0u8; 4];
            let str_val = cell.ch.encode_utf8(&mut s);
            let w = UnicodeWidthStr::width(str_val).max(1);

            if cell.visible {
                if cell.fg != last_fg {
                    if let Some(fg) = cell.fg {
                        queue!(out, SetForegroundColor(fg)).ok();
                    } else if last_fg.is_some() {
                        queue!(out, ResetColor).ok();
                    }
                    last_fg = cell.fg;
                }
                if cell.bg != last_bg {
                    if let Some(bg) = cell.bg {
                        queue!(out, SetBackgroundColor(bg)).ok();
                    } else if last_bg.is_some() {
                        queue!(out, ResetColor).ok();
                    }
                    last_bg = cell.bg;
                }
                queue!(out, style::Print(cell.ch)).ok();
            } else {
                if (last_fg.is_some() && last_fg != Some(Color::Reset))
                    || (last_bg.is_some() && last_bg != Some(Color::Reset))
                {
                    queue!(out, ResetColor).ok();
                    last_fg = None;
                    last_bg = None;
                }
                queue!(out, style::Print(' ')).ok();
            }
            col += w as u16;
        }

        // Pad remainder of line with spaces to overwrite any stale content
        while col < term_width {
            queue!(out, style::Print(' ')).ok();
            col += 1;
        }
        if i < grid.cells.len() - 1 {
            queue!(out, style::Print('\n')).ok();
        }
    }

    if (last_fg.is_some() && last_fg != Some(Color::Reset))
        || (last_bg.is_some() && last_bg != Some(Color::Reset))
    {
        queue!(out, ResetColor).ok();
    }

    // End synchronized update
    out.write_all(b"\x1b[?2026l").ok();

    out.flush().ok();
}

/// Absolute deadline drift-free animation sleep loop
pub fn run_animation<F>(grid: &mut Grid, frame_rate: u32, mut tick: F)
where
    F: FnMut(&mut Grid, usize) -> bool, // returns true when done
{
    let mut stdout = BufWriter::with_capacity(64 * 1024, io::stdout());

    // Save cursor position — only query if both stdin and stdout are terminals
    // (DSR sends escape to stdout but reads response from stdin, blocks on pipes)
    let origin_row = if io::stdin().is_terminal() && io::stdout().is_terminal() {
        cursor::position().map(|(_, y)| y).unwrap_or(0)
    } else {
        0
    };

    execute!(stdout, cursor::Hide).ok();

    let term_width = terminal::size().map(|(w, _)| w).unwrap_or(80);
    let frame_duration = Duration::from_micros(1_000_000 / frame_rate as u64);
    let mut frame = 0;
    let mut next_frame_deadline = Instant::now();

    loop {
        let done = tick(grid, frame);
        render_frame(grid, &mut stdout, origin_row, term_width);

        if done {
            break;
        }

        frame += 1;
        next_frame_deadline += frame_duration;

        let now = Instant::now();
        if now < next_frame_deadline {
            std::thread::sleep(next_frame_deadline - now);
        }
    }

    // Final frame — ensure visibility
    for row in &mut grid.cells {
        for cell in row {
            cell.visible = true;
        }
    }
    render_frame(grid, &mut stdout, origin_row, term_width);

    // Move cursor below the grid
    queue!(stdout, cursor::MoveTo(0, origin_row + grid.height as u16)).ok();
    execute!(stdout, cursor::Show).ok();
}

#[cfg(test)]
#[path = "tests/engine.rs"]
mod tests;
