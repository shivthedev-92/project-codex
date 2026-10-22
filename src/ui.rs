//! Terminal presentation: palette, panels, tables, and animations.
//!
//! Colour and animation only happen on an interactive terminal. Redirected
//! output, CI, `NO_COLOR`, and `TERM=dumb` get plain, stable text.

use std::io::{IsTerminal, Write, stdout};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use unicode_width::UnicodeWidthStr;

use crate::art::{INTRO_FRAMES, MAGIC_WAND_FRAMES, SPELLBOOK_LOGO};
use crate::safety::Hazard;

pub const SUBTITLE: &str = "Arcane Hardware Interface";

/// ANSI SGR codes.
pub mod palette {
    pub const ACCENT: &str = "1;96";
    pub const GOLD: &str = "1;33";
    pub const DANGER: &str = "1;31";
    pub const SUCCESS: &str = "1;32";
    pub const MUTED: &str = "38;5;249";
    pub const PANEL: &str = "94";
    pub const LABEL: &str = "1;97";
}

pub fn interactive() -> bool {
    static INTERACTIVE: OnceLock<bool> = OnceLock::new();
    *INTERACTIVE.get_or_init(|| {
        stdout().is_terminal() && std::env::var("TERM").map_or(true, |term| term != "dumb")
    })
}

fn color_enabled() -> bool {
    static COLOR: OnceLock<bool> = OnceLock::new();
    *COLOR.get_or_init(|| interactive() && std::env::var_os("NO_COLOR").is_none())
}

pub fn paint(text: &str, style: &str) -> String {
    if color_enabled() && !text.is_empty() {
        format!("\x1b[{style}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn hazard_style(hazard: Hazard) -> &'static str {
    match hazard {
        Hazard::Safe => palette::SUCCESS,
        Hazard::Caution => palette::GOLD,
        Hazard::Danger => palette::DANGER,
    }
}

fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn pad(text: &str, to: usize) -> String {
    " ".repeat(to.saturating_sub(width(text)))
}

/// A line of text made of styled spans; widths are measured before painting.
type Line = Vec<(String, &'static str)>;

fn line_width(line: &Line) -> usize {
    line.iter().map(|(text, _)| width(text)).sum()
}

fn boxed(lines: &[Line], centered: bool) -> String {
    let inner = lines.iter().map(line_width).max().unwrap_or(0);
    let border = |text: &str| paint(text, palette::PANEL);
    let mut out = border(&format!("╭{}╮", "─".repeat(inner + 2)));
    for line in lines {
        let slack = inner - line_width(line);
        let (left, right) = if centered {
            (slack / 2, slack - slack / 2)
        } else {
            (0, slack)
        };
        let body: String = line
            .iter()
            .map(|(text, style)| paint(text, style))
            .collect();
        out.push('\n');
        out.push_str(&format!(
            "{} {}{body}{} {}",
            border("│"),
            " ".repeat(left),
            " ".repeat(right),
            border("│")
        ));
    }
    out.push('\n');
    out.push_str(&border(&format!("╰{}╯", "─".repeat(inner + 2))));
    out
}

/// `LABEL: message`, boxed. Extra message lines are indented beneath it.
pub fn panel(label: &str, message: &str, style: &'static str) -> String {
    let prefix = format!("{label}: ");
    let indent = "   ".to_string();
    let lines: Vec<Line> = message
        .lines()
        .enumerate()
        .map(|(index, text)| {
            let lead = if index == 0 {
                prefix.clone()
            } else {
                indent.clone()
            };
            vec![(lead, palette::LABEL), (text.to_string(), style)]
        })
        .collect();
    let lines = if lines.is_empty() {
        vec![vec![(prefix, palette::LABEL)]]
    } else {
        lines
    };
    boxed(&lines, false)
}

pub fn banner() -> String {
    // Pad the logo to one width so centering moves it as a block.
    let logo_width = SPELLBOOK_LOGO.lines().map(width).max().unwrap_or(0);
    let mut lines: Vec<Line> = SPELLBOOK_LOGO
        .lines()
        .map(|line| vec![(format!("{line}{}", pad(line, logo_width)), palette::ACCENT)])
        .collect();
    lines.push(vec![]);
    lines.push(vec![(SUBTITLE.to_string(), palette::GOLD)]);
    lines.push(vec![("✨ STATUS: READY".to_string(), palette::SUCCESS)]);
    boxed(&lines, true)
}

/// A bordered table. `styles[i]` colours column `i`.
pub fn table(
    title: &str,
    headers: &[&str],
    rows: &[Vec<String>],
    styles: &[&'static str],
) -> String {
    let columns = headers.len();
    let widths: Vec<usize> = (0..columns)
        .map(|column| {
            rows.iter()
                .filter_map(|row| row.get(column))
                .map(|cell| width(cell))
                .chain([width(headers[column])])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let border = |text: &str| paint(text, palette::PANEL);
    let rule = |left: &str, mid: &str, right: &str| {
        let segments: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        border(&format!("{left}{}{right}", segments.join(mid)))
    };
    let row_line = |cells: Vec<String>| {
        let mut line = border("│");
        for (column, cell) in cells.iter().enumerate() {
            let style = styles.get(column).copied().unwrap_or("");
            line.push_str(&format!(
                " {}{} ",
                paint(cell, style),
                pad(cell, widths[column])
            ));
            line.push_str(&border("│"));
        }
        line
    };

    let total = widths.iter().map(|w| w + 3).sum::<usize>() + 1;
    let mut out = String::new();
    if !title.is_empty() {
        out.push_str(&" ".repeat(total.saturating_sub(width(title)) / 2));
        out.push_str(&paint(title, palette::ACCENT));
        out.push('\n');
    }
    out.push_str(&rule("┌", "┬", "┐"));
    out.push('\n');
    let mut header = border("│");
    for (column, text) in headers.iter().enumerate() {
        header.push_str(&format!(
            " {}{} ",
            paint(text, palette::LABEL),
            pad(text, widths[column])
        ));
        header.push_str(&border("│"));
    }
    out.push_str(&header);
    out.push('\n');
    out.push_str(&rule("├", "┼", "┤"));
    for row in rows {
        let mut cells = row.clone();
        cells.resize(columns, String::new());
        out.push('\n');
        out.push_str(&row_line(cells));
    }
    out.push('\n');
    out.push_str(&rule("└", "┴", "┘"));
    out
}

/// Play the intro once, then erase it.
pub fn intro_animation() {
    if !interactive() {
        return;
    }
    let mut out = stdout();
    for _ in 0..2 {
        for (logo, status) in INTRO_FRAMES {
            let mut lines: Vec<Line> = logo
                .lines()
                .map(|l| vec![(l.to_string(), palette::ACCENT)])
                .collect();
            lines.push(vec![]);
            lines.extend(status.iter().map(|s| vec![(s.to_string(), palette::LABEL)]));
            let frame = boxed(&lines, false);
            let height = frame.lines().count();
            let _ = writeln!(out, "{frame}");
            let _ = out.flush();
            thread::sleep(Duration::from_millis(120));
            let _ = write!(out, "\x1b[{height}A\x1b[J");
        }
    }
    let _ = out.flush();
}

/// Animate a wand while `work` runs. Non-interactive output gets one line.
pub fn magic_wand<T>(message: &str, work: impl FnOnce() -> T) -> T {
    if !interactive() {
        println!("🪄 {message}…");
        return work();
    }
    let stopped = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            let mut out = stdout();
            for frame in MAGIC_WAND_FRAMES.iter().cycle() {
                if stopped.load(Ordering::Relaxed) {
                    break;
                }
                let _ = write!(
                    out,
                    "\r\x1b[2K{}{}{}",
                    paint(frame, palette::GOLD),
                    paint(message, palette::ACCENT),
                    paint("…", palette::MUTED)
                );
                let _ = out.flush();
                thread::sleep(Duration::from_millis(120));
            }
            let _ = write!(out, "\r\x1b[2K");
            let _ = out.flush();
        });
        let result = work();
        stopped.store(true, Ordering::Relaxed);
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_align_wide_glyphs() {
        // Tests run without a terminal, so output is uncoloured.
        let text = panel("👹 GOBLIN", "first line\nsecond", palette::DANGER);
        let widths: Vec<usize> = text.lines().map(width).collect();
        assert!(widths.windows(2).all(|pair| pair[0] == pair[1]), "{text}");
        assert!(text.contains("👹 GOBLIN: first line"));
    }

    #[test]
    fn tables_pad_every_row_to_the_same_width() {
        let rows = vec![vec!["📍 /map".into(), "scan".into()], vec!["x".into()]];
        let text = table("Grimoire", &["Spell", "Alias"], &rows, &[]);
        let widths: Vec<usize> = text.lines().skip(1).map(width).collect();
        assert!(widths.windows(2).all(|pair| pair[0] == pair[1]), "{text}");
    }
}
