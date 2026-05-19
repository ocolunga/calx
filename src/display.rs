use chrono::{Datelike, NaiveDate};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use std::collections::HashSet;

use crate::calendar::{
    WeekConfig, compute_biweek_number, compute_week_number, day_names, day_of_year,
    month_weeks, next_month, prev_month, view_label,
};

// ── ANSI color helpers ────────────────────────────────────────────────────────
// These wrap a string with ANSI escape codes.  We use them in the manual
// calendar renderer so we can apply color without comfy-table's cell padding.

fn ansi(code: &str, s: &str) -> String { format!("\x1b[{code}m{s}\x1b[0m") }
fn cyan(s: &str)         -> String { ansi("36",  s) }
fn yellow(s: &str)       -> String { ansi("33",  s) }
fn magenta(s: &str)      -> String { ansi("35",  s) }
fn bold_reverse(s: &str) -> String { ansi("1;7", s) }

// ── Info table ────────────────────────────────────────────────────────────────

/// Render today's calendar metrics as a bordered table, returned as a String.
/// comfy-table is appropriate here because the info panel needs box borders.
pub fn render_info_table(today: NaiveDate, cfg: WeekConfig) -> String {
    let week   = compute_week_number(today, cfg);
    let biweek = compute_biweek_number(week);
    let doy    = day_of_year(today);
    let label  = view_label(cfg);

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);

    table.set_header(vec![
        Cell::new(&label).add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new(""),
    ]);

    let rows: &[(&str, String)] = &[
        ("Week",        week.to_string()),
        ("Biweek",      biweek.to_string()),
        ("Year",        today.year().to_string()),
        ("Day",         today.format("%A").to_string()),
        ("Day of Year", doy.to_string()),
    ];

    for (prop, val) in rows {
        table.add_row(vec![
            Cell::new(*prop).fg(Color::Cyan),
            Cell::new(val.as_str()).fg(Color::Green),
        ]);
    }

    table.to_string()
}

// ── Month calendar (compact string renderer) ──────────────────────────────────
//
// comfy-table's NOTHING preset still adds one space of padding to each side of
// every cell, making the grid ~40% wider than needed.  We build the grid
// manually so each column is exactly as wide as its content.
//
// Column layout (show_week_numbers = true):
//   W##  <-- 4 chars: "W" + up-to-2-digit number, left-padded to 4
//   Mo Tu We Th Fr Sa Su  <-- 2 chars per day, space-separated = 20 chars
//    B##  <-- 4 chars: " B" + up-to-2-digit biweek, or 4 spaces when empty
//
// Total visible width per month: 4 + 20 + 4 = 28 chars.

/// Render one month's calendar grid as a compact String.
pub fn render_month_calendar(
    year: i32,
    month: u32,
    today: Option<NaiveDate>,
    show_week_numbers: bool,
    cfg: WeekConfig,
) -> String {
    let weeks = month_weeks(year, month, cfg);
    let names = day_names(cfg.first_day);
    let mut lines: Vec<String> = Vec::new();

    // ── Header row ────────────────────────────────────────────────────────────
    let mut hdr = String::new();
    if show_week_numbers {
        hdr.push_str("    "); // 4-char week-number column placeholder
    }
    for (i, name) in names.iter().enumerate() {
        if i > 0 { hdr.push(' '); }
        hdr.push_str(&cyan(name)); // 2 visible chars, bold cyan
    }
    if show_week_numbers {
        hdr.push_str("    "); // 4-char biweek column placeholder
    }
    lines.push(hdr);

    // ── Data rows ─────────────────────────────────────────────────────────────
    let mut seen_biweeks: HashSet<u32> = HashSet::new();

    for week in &weeks {
        // Use the first in-month day of this row to label the week.
        let week_num: Option<u32> = week
            .iter()
            .find(|d| d.month() == month)
            .map(|d| compute_week_number(*d, cfg));

        let mut line = String::new();

        // Week-number column: "W3 " / "W21" (left-padded to 3) + space.
        if show_week_numbers {
            let raw = week_num
                .map(|w| format!("W{w}"))
                .unwrap_or_default();
            // Colour the raw text, then pad the *visible* part to 3 chars.
            let coloured = if week_num.is_some() { yellow(&raw) } else { raw.clone() };
            let pad = 3usize.saturating_sub(raw.len());
            line.push_str(&coloured);
            line.push_str(&" ".repeat(pad + 1)); // +1 for the separator space
        }

        // Day cells: 2 visible chars each, separated by a single space.
        for (i, day) in week.iter().enumerate() {
            if i > 0 { line.push(' '); }
            if day.month() != month {
                line.push_str("  "); // out-of-month: two spaces
            } else if today == Some(*day) {
                line.push_str(&bold_reverse(&format!("{:>2}", day.day())));
            } else {
                line.push_str(&format!("{:>2}", day.day()));
            }
        }

        // Biweek column: " B9 " / " B12" (4 chars) or "    " when blank.
        if show_week_numbers {
            match week_num {
                Some(w) => {
                    let bw = compute_biweek_number(w);
                    if seen_biweeks.insert(bw) {
                        let raw = format!("B{bw}");
                        // Left-pad the coloured label inside a 3-char field.
                        let coloured = magenta(&raw);
                        let pad = 3usize.saturating_sub(raw.len());
                        line.push(' '); // separator before biweek
                        line.push_str(&coloured);
                        line.push_str(&" ".repeat(pad));
                    } else {
                        line.push_str("    "); // 4 spaces: same width as " B9 "
                    }
                }
                None => line.push_str("    "),
            }
        }

        lines.push(line);
    }

    lines.join("\n")
}

/// Add a centred "Month YYYY" title above the calendar grid.
pub fn render_month_with_title(
    year: i32,
    month: u32,
    today: Option<NaiveDate>,
    show_week_numbers: bool,
    cfg: WeekConfig,
) -> String {
    let title = format!("{} {year}", month_name(month));
    let cal = render_month_calendar(year, month, today, show_week_numbers, cfg);

    // Measure visible width (ANSI codes are invisible characters).
    let cal_width = cal.lines().map(|l| visible_len(l)).max().unwrap_or(title.len());
    let pad = (cal_width.saturating_sub(title.len())) / 2;
    let centered = format!("{:pad$}{title}", "");

    format!("{centered}\n{cal}")
}

// ── Layout ────────────────────────────────────────────────────────────────────

/// Join multiple rendered blocks side-by-side with `gap` spaces between them.
/// Uses `visible_len` for padding so ANSI colour codes don't distort alignment.
pub fn side_by_side(blocks: &[String], gap: usize) -> String {
    let split: Vec<Vec<&str>> = blocks.iter().map(|b| b.lines().collect()).collect();

    let widths: Vec<usize> = split
        .iter()
        .map(|ls| ls.iter().map(|l| visible_len(l)).max().unwrap_or(0))
        .collect();

    let height = split.iter().map(|ls| ls.len()).max().unwrap_or(0);
    let spacer = " ".repeat(gap);
    let mut result = String::new();

    for row in 0..height {
        for (col, lines) in split.iter().enumerate() {
            let line = lines.get(row).copied().unwrap_or("");
            result.push_str(line);
            if col < split.len() - 1 {
                let vis = visible_len(line);
                let pad = widths[col].saturating_sub(vis);
                result.push_str(&" ".repeat(pad));
                result.push_str(&spacer);
            }
        }
        result.push('\n');
    }

    result
}

// ── Multi-month views ─────────────────────────────────────────────────────────

pub fn render_three_months(
    center_year: i32,
    center_month: u32,
    today: Option<NaiveDate>,
    cfg: WeekConfig,
) -> String {
    let (py, pm) = prev_month(center_year, center_month);
    let (ny, nm) = next_month(center_year, center_month);
    let blocks = vec![
        render_month_with_title(py,          pm,           today, true, cfg),
        render_month_with_title(center_year, center_month, today, true, cfg),
        render_month_with_title(ny,          nm,           today, true, cfg),
    ];
    side_by_side(&blocks, 2)
}

pub fn render_year(year: i32, today: Option<NaiveDate>, cfg: WeekConfig) -> String {
    let title = format!("  {year}\n\n");
    let mut parts = vec![title];
    for row_idx in 0..4u32 {
        let blocks: Vec<String> = (0..3u32)
            .map(|col| {
                let month = row_idx * 3 + col + 1;
                render_month_with_title(year, month, today, true, cfg)
            })
            .collect();
        parts.push(side_by_side(&blocks, 2));
        parts.push("\n".to_string());
    }
    parts.join("")
}

// ── Default view ──────────────────────────────────────────────────────────────

pub fn display_default_view(today: NaiveDate, cfg: WeekConfig) {
    let info  = render_info_table(today, cfg);
    let month = render_month_with_title(today.year(), today.month(), Some(today), true, cfg);
    print!("{}", side_by_side(&[info, month], 4));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",   2 => "February", 3 => "March",
        4 => "April",     5 => "May",       6 => "June",
        7 => "July",      8 => "August",    9 => "September",
        10 => "October",  11 => "November", 12 => "December",
        _ => "Unknown",
    }
}

/// Count printable characters in a string, skipping ANSI escape sequences
/// (`\x1b[...m`) so colour codes don't distort visible-width calculations.
fn visible_len(s: &str) -> usize {
    let mut count = 0usize;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b'         { in_escape = true; }
        else if in_escape       { if ch == 'm' { in_escape = false; } }
        else                    { count += 1; }
    }
    count
}
