use chrono::{Datelike, NaiveDate};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use std::collections::HashSet;

use crate::calendar::{
    WeekConfig, compute_biweek_number, compute_week_number, day_names, day_of_year,
    month_weeks, next_month, prev_month, view_label,
};

// ── Info table ────────────────────────────────────────────────────────────────

/// Render today's calendar metrics as a bordered table, returned as a String.
pub fn render_info_table(today: NaiveDate, cfg: WeekConfig) -> String {
    let week = compute_week_number(today, cfg);
    let biweek = compute_biweek_number(week);
    let doy = day_of_year(today);
    let label = view_label(cfg);

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);

    // Title row spans both columns.
    table.set_header(vec![
        Cell::new(&label)
            .add_attribute(Attribute::Bold)
            .fg(Color::Blue),
        Cell::new(""),
    ]);

    let rows: &[(&str, String)] = &[
        ("Week", week.to_string()),
        ("Biweek", biweek.to_string()),
        ("Year", today.year().to_string()),
        ("Day", today.format("%A").to_string()),
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

// ── Month calendar ────────────────────────────────────────────────────────────

/// Render one month's calendar grid (borderless) as a String.
pub fn render_month_calendar(
    year: i32,
    month: u32,
    today: Option<NaiveDate>,
    show_week_numbers: bool,
    cfg: WeekConfig,
) -> String {
    let weeks = month_weeks(year, month, cfg);
    let names = day_names(cfg.first_day);

    let mut table = Table::new();
    table.load_preset(presets::NOTHING);

    // Header row: optional W# placeholder + day abbreviations + optional B# placeholder.
    let mut header: Vec<Cell> = Vec::new();
    if show_week_numbers {
        header.push(Cell::new("W#").fg(Color::DarkGrey));
    }
    for name in &names {
        header.push(
            Cell::new(*name)
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        );
    }
    if show_week_numbers {
        header.push(Cell::new("B#").fg(Color::DarkGrey));
    }
    table.set_header(header);

    // Track which biweeks have already been labelled so we don't repeat them.
    let mut seen_biweeks: HashSet<u32> = HashSet::new();

    for week in &weeks {
        let mut row: Vec<Cell> = Vec::new();

        // Use the first in-month day to compute the week's number.
        let week_num: Option<u32> = week
            .iter()
            .find(|d| d.month() == month)
            .map(|d| compute_week_number(*d, cfg));

        if show_week_numbers {
            let label = week_num
                .map(|w| format!("W{w}"))
                .unwrap_or_default();
            row.push(Cell::new(label).fg(Color::Yellow));
        }

        for day in week {
            if day.month() != month {
                // Out-of-month padding cell.
                row.push(Cell::new("  "));
            } else if today == Some(*day) {
                // Highlight today with reverse video.
                row.push(
                    Cell::new(format!("{:2}", day.day()))
                        .add_attribute(Attribute::Reverse)
                        .add_attribute(Attribute::Bold),
                );
            } else {
                row.push(Cell::new(format!("{:2}", day.day())));
            }
        }

        if show_week_numbers {
            let bw_cell = match week_num {
                Some(wn) => {
                    let bw = compute_biweek_number(wn);
                    if seen_biweeks.insert(bw) {
                        Cell::new(format!("B{bw}")).fg(Color::Magenta)
                    } else {
                        Cell::new("")
                    }
                }
                None => Cell::new(""),
            };
            row.push(bw_cell);
        }

        table.add_row(row);
    }

    table.to_string()
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

    // Measure the visible width of the calendar's first content line.
    // We strip ANSI escape codes before measuring so colours don't inflate
    // the count (visible width ≠ byte length when ANSI is present).
    let cal_width = cal
        .lines()
        .map(|l| visible_len(l))
        .max()
        .unwrap_or(title.len());

    let pad = (cal_width.saturating_sub(title.len())) / 2;
    let centered = format!("{:pad$}{title}", "");

    format!("{centered}\n{cal}")
}

// ── Layout ────────────────────────────────────────────────────────────────────

/// Join multiple rendered blocks side-by-side with `gap` spaces between them.
///
/// Because comfy-table emits ANSI escape codes for colours, `str::len()`
/// would overcount visible characters. We use `visible_len` (which strips
/// ANSI) for width calculations so column alignment stays correct.
pub fn side_by_side(blocks: &[String], gap: usize) -> String {
    // Split each block into lines.
    let split: Vec<Vec<&str>> = blocks.iter().map(|b| b.lines().collect()).collect();

    // Visible width of each block (max across all its lines).
    let widths: Vec<usize> = split
        .iter()
        .map(|lines| lines.iter().map(|l| visible_len(l)).max().unwrap_or(0))
        .collect();

    let height = split.iter().map(|ls| ls.len()).max().unwrap_or(0);
    let spacer = " ".repeat(gap);
    let mut result = String::new();

    for row in 0..height {
        for (col, lines) in split.iter().enumerate() {
            let line = lines.get(row).copied().unwrap_or("");
            result.push_str(line);
            if col < split.len() - 1 {
                // Pad with spaces to reach the block's full visible width,
                // then add the gap before the next block.
                let visible = visible_len(line);
                let pad = widths[col].saturating_sub(visible);
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
        render_month_with_title(py, pm, today, true, cfg),
        render_month_with_title(center_year, center_month, today, true, cfg),
        render_month_with_title(ny, nm, today, true, cfg),
    ];
    side_by_side(&blocks, 4)
}

pub fn render_year(year: i32, today: Option<NaiveDate>, cfg: WeekConfig) -> String {
    let title = format!("  {year}\n\n");
    let mut parts = vec![title];

    for row_idx in 0..4u32 {
        let blocks: Vec<String> = (0..3u32)
            .map(|col_idx| {
                let month = row_idx * 3 + col_idx + 1;
                render_month_with_title(year, month, today, true, cfg)
            })
            .collect();
        parts.push(side_by_side(&blocks, 4));
        parts.push("\n".to_string());
    }

    parts.join("")
}

// ── Default view ──────────────────────────────────────────────────────────────

/// Show the info panel and this month's calendar side by side.
pub fn display_default_view(today: NaiveDate, cfg: WeekConfig) {
    let info = render_info_table(today, cfg);
    let month = render_month_with_title(today.year(), today.month(), Some(today), true, cfg);
    print!("{}", side_by_side(&[info, month], 4));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

/// Count the visible (printed) characters in a string, ignoring ANSI
/// escape sequences like `\x1b[32m` (colour codes).
fn visible_len(s: &str) -> usize {
    let mut count = 0usize;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            count += 1;
        }
    }
    count
}
